//! Console input (`ConsoleInputHandler`, `ConsoleInputPump`, `HeadlessConsoleInputException`).
//!
//! The console state is a cloneable [`ConsoleInputHandler`] handle (shared state behind an `Arc`); the owner of
//! `Core.Headless` and `Core.Closing` passes them in. Console commands run on the reader thread, so
//! they are `Send + Sync` closures with no access to the `World`. They must never call `read_line`.

use muo_primitives::dotnet::InvariantCase;
use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;

use indexmap::IndexMap;

/// `HeadlessConsoleInputException`: interactive input is required but stdin is not a TTY.
#[derive(Debug, thiserror::Error)]
#[error(
    "Interactive console input required but the server is headless (stdin is not a TTY). Pre-supply the required configuration/save data. Prompt: {context}."
)]
pub struct HeadlessConsoleInputError {
    pub context: String,
}

impl HeadlessConsoleInputError {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
        }
    }
}

/// A console command's function; the argument is the rest of the line.
pub type ConsoleAction = Arc<dyn Fn(&str) + Send + Sync>;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Default)]
struct Gate {
    prompt_pending: bool,
    prompt_result: Option<String>,
    /// The `AutoResetEvent`.
    delivered: bool,
}

type Lookup = Box<dyn Fn(&str) -> Option<ConsoleAction> + Send + Sync>;

/// `ConsoleInputPump`: owns the input stream; each line read goes either to a waiting `read_line` (a prompt) or
/// is dispatched as a command through the lookup. EOF ends the loop.
pub struct ConsoleInputPump {
    input: Mutex<Box<dyn BufRead + Send>>,
    lookup: Lookup,
    closing: Arc<AtomicBool>,
    gate: Mutex<Gate>,
    cv: Condvar,
    running: AtomicBool,
}

impl ConsoleInputPump {
    pub fn new(
        input: Box<dyn BufRead + Send>,
        lookup: impl Fn(&str) -> Option<ConsoleAction> + Send + Sync + 'static,
        closing: Arc<AtomicBool>,
    ) -> Self {
        Self {
            input: Mutex::new(input),
            lookup: Box::new(lookup),
            closing,
            gate: Mutex::new(Gate::default()),
            cv: Condvar::new(),
            running: AtomicBool::new(true),
        }
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// True while a `read_line` caller is registered and waiting for the next line.
    pub fn has_pending_prompt(&self) -> bool {
        lock(&self.gate).prompt_pending
    }

    pub fn run(&self) {
        let mut input = lock(&self.input);
        while self.running() && !self.closing.load(Ordering::SeqCst) {
            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => break, // EOF: never spin
                Ok(_) => {}
                Err(_) => {
                    tracing::warn!("Console commands have been disabled due to an error.");
                    break;
                }
            }
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            let is_command = {
                let mut g = lock(&self.gate);
                if g.prompt_pending {
                    g.prompt_result = Some(line.clone());
                    g.prompt_pending = false;
                    g.delivered = true;
                    self.cv.notify_all();
                    false
                } else {
                    true
                }
            };
            if !is_command {
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (name, rest) = trimmed.split_once(' ').unwrap_or((trimmed, ""));

            let name = name.lower_inv();
            // A panicking handler is caught and logged.
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(action) = (self.lookup)(&name) {
                    action(rest);
                }
            }));
            if r.is_err() {
                tracing::error!("Failed to execute console command: {}", line);
            }
        }
        self.running.store(false, Ordering::SeqCst);
        self.release_pending_prompt();
    }

    /// Blocks until the next console line is available, or until the pump stops (`None`). One caller at a time;
    /// never from the reader thread.
    pub fn read_line(&self) -> Option<String> {
        let mut g = lock(&self.gate);
        if !self.running() {
            return None;
        }
        g.prompt_result = None;
        g.prompt_pending = true;
        while !g.delivered {
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
        g.delivered = false;
        g.prompt_result.clone()
    }

    fn release_pending_prompt(&self) {
        let mut g = lock(&self.gate);
        if g.prompt_pending {
            g.prompt_result = None;
            g.prompt_pending = false;
            g.delivered = true;
            self.cv.notify_all();
        }
    }
}

struct ConsoleCommand {
    commands: Vec<String>,
    description: String,
    function: ConsoleAction,
}

#[derive(Default)]
struct Inner {
    commands: Mutex<IndexMap<String, Arc<ConsoleCommand>>>,
    descriptions: Mutex<Option<Vec<String>>>,
    pump: Mutex<Option<Arc<ConsoleInputPump>>>,
}

/// `ConsoleInputHandler`.
#[derive(Clone)]
pub struct ConsoleInputHandler {
    inner: Arc<Inner>,
    headless: bool,
    closing: Arc<AtomicBool>,
}

impl ConsoleInputHandler {
    pub fn new(headless: bool, closing: Arc<AtomicBool>) -> Self {
        Self {
            inner: Arc::new(Inner::default()),
            headless,
            closing,
        }
    }

    pub fn register_command(
        &self,
        command: &str,
        description: &str,
        function: impl Fn(&str) + Send + Sync + 'static,
    ) {
        self.register_commands(&[command], description, function);
    }

    /// Functions run on the reader thread.
    pub fn register_commands(
        &self,
        commands: &[&str],
        description: &str,
        function: impl Fn(&str) + Send + Sync + 'static,
    ) {
        if commands.is_empty() {
            return;
        }
        let cc = Arc::new(ConsoleCommand {
            commands: commands.iter().map(|c| c.to_string()).collect(),
            description: description.to_string(),
            function: Arc::new(function),
        });
        {
            let mut map = lock(&self.inner.commands);
            for c in commands {
                map.insert(c.lower_inv(), cc.clone());
            }
        }
        *lock(&self.inner.descriptions) = None;
    }

    /// `UnregisterInputCommand` (the name isn't lowercased).
    pub fn unregister_input_command(&self, command: &str) -> bool {
        let removed = lock(&self.inner.commands).shift_remove(command).is_some();
        *lock(&self.inner.descriptions) = None;
        removed
    }

    pub fn get_input_command(&self, command: &str) -> Option<ConsoleAction> {
        lock(&self.inner.commands)
            .get(command)
            .map(|c| c.function.clone())
    }

    /// `Configure`: the default `help|?`.
    pub fn configure(&self) {
        let this = self.clone();
        self.register_commands(&["help", "?"], "Displays this help screen.", move |_| {
            this.display_help()
        });
    }

    /// `Initialize`: starts the 'Console Input Handler' background thread unless headless.
    pub fn initialize(&self) -> io::Result<()> {
        if self.headless {
            tracing::info!("Console input disabled (headless: stdin is not a TTY).");
            return Ok(());
        }
        let this = self.clone();
        let pump = Arc::new(ConsoleInputPump::new(
            Box::new(io::BufReader::new(io::stdin())),
            move |c| this.get_input_command(c),
            self.closing.clone(),
        ));
        *lock(&self.inner.pump) = Some(pump.clone());
        thread::Builder::new()
            .name("Console Input Handler".to_string())
            .spawn(move || pump.run())?;
        Ok(())
    }

    fn help_descriptions(&self) -> Vec<String> {
        if let Some(d) = lock(&self.inner.descriptions).as_ref() {
            return d.clone();
        }

        // the distinct commands
        let mut distinct: Vec<Arc<ConsoleCommand>> = Vec::new();
        for c in lock(&self.inner.commands).values() {
            if !distinct.iter().any(|d| Arc::ptr_eq(d, c)) {
                distinct.push(c.clone());
            }
        }

        let mut tuples: Vec<(String, String)> = distinct
            .iter()
            .map(|c| (c.commands.join("|"), c.description.clone()))
            .collect();
        let longest = tuples
            .iter()
            .map(|t| t.0.encode_utf16().count())
            .max()
            .unwrap_or(0);
        tuples.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));

        let d: Vec<String> = tuples
            .iter()
            .map(|(aliases, desc)| {
                let pad = (longest + 1).saturating_sub(aliases.encode_utf16().count());
                format!("{aliases}{} - {desc}", " ".repeat(pad))
            })
            .collect();
        *lock(&self.inner.descriptions) = Some(d.clone());
        d
    }

    fn display_help(&self) {
        let d = self.help_descriptions();
        if d.is_empty() {
            println!("No console commands registered.");
            return;
        }
        println!("Available Commands:");
        for line in d {
            println!("{line}");
        }
    }

    /// `ReadLine`: `Err` when headless; before `initialize` or after the loop ended, reads stdin directly.
    /// `Ok(None)` is end of input.
    pub fn read_line(&self) -> Result<Option<String>, HeadlessConsoleInputError> {
        if self.headless {
            return Err(HeadlessConsoleInputError::new(
                "ConsoleInputHandler.ReadLine",
            ));
        }
        let pump = lock(&self.inner.pump).clone();
        match pump {
            Some(p) if p.running() => Ok(p.read_line()),
            _ => {
                let mut s = String::new();
                match io::stdin().read_line(&mut s) {
                    Ok(0) | Err(_) => Ok(None),
                    Ok(_) => {
                        let t = s.trim_end_matches(['\n', '\r']).len();
                        s.truncate(t);
                        Ok(Some(s))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::*;

    fn pump(
        text: &str,
        lookup: impl Fn(&str) -> Option<ConsoleAction> + Send + Sync + 'static,
    ) -> Arc<ConsoleInputPump> {
        Arc::new(ConsoleInputPump::new(
            Box::new(Cursor::new(text.to_string().into_bytes())),
            lookup,
            Arc::new(AtomicBool::new(false)),
        ))
    }

    #[test]
    fn eof_ends_loop() {
        let p = pump("", |_| None);
        p.run();
        assert!(!p.running());
    }

    #[test]
    fn recognized_line_dispatches_command() {
        let (tx, rx) = mpsc::channel();
        let tx = Mutex::new(tx);
        let action: ConsoleAction = Arc::new(move |a| {
            tx.lock().unwrap().send(a.to_string()).unwrap();
        });
        let p = pump("ping hello\n", move |c| {
            (c == "ping").then(|| action.clone())
        });
        p.run();
        assert_eq!(rx.try_recv().unwrap(), "hello");
    }

    /// A reader fed through a channel, so the prompt registers before the line arrives.
    struct Chan(mpsc::Receiver<String>);

    impl io::Read for Chan {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl BufRead for Chan {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _: usize) {}
        fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
            match self.0.recv() {
                Ok(l) => {
                    buf.push_str(&l);
                    Ok(l.len())
                }
                Err(_) => Ok(0),
            }
        }
    }

    #[test]
    fn line_goes_to_waiting_prompt_not_command() {
        let (cmd_tx, cmd_rx) = mpsc::channel::<String>();
        let cmd_tx = Mutex::new(cmd_tx);
        let action: ConsoleAction = Arc::new(move |a| {
            cmd_tx.lock().unwrap().send(a.to_string()).unwrap();
        });
        let (tx, rx) = mpsc::channel();
        let p = Arc::new(ConsoleInputPump::new(
            Box::new(Chan(rx)),
            move |c| (c == "ping").then(|| action.clone()),
            Arc::new(AtomicBool::new(false)),
        ));
        let runner = {
            let p = p.clone();
            thread::spawn(move || p.run())
        };
        let reader = {
            let p = p.clone();
            thread::spawn(move || p.read_line())
        };
        let t = Instant::now();
        while !p.has_pending_prompt() {
            assert!(t.elapsed() < Duration::from_secs(5));
            thread::yield_now();
        }
        tx.send("ping answer\n".to_string()).unwrap();
        assert_eq!(reader.join().unwrap().as_deref(), Some("ping answer"));
        tx.send("ping next\n".to_string()).unwrap();
        assert_eq!(cmd_rx.recv_timeout(Duration::from_secs(5)).unwrap(), "next");
        drop(tx);
        runner.join().unwrap();
        assert_eq!(p.read_line(), None);
    }

    #[test]
    fn help_table_is_sorted_and_padded() {
        let h = ConsoleInputHandler::new(true, Arc::new(AtomicBool::new(false)));
        h.configure();
        h.register_command("save", "Saves.", |_| {});
        assert_eq!(
            h.help_descriptions(),
            vec![
                "help|?  - Displays this help screen.".to_string(),
                "save    - Saves.".to_string()
            ]
        );
        assert!(h.read_line().is_err());
        assert!(h.get_input_command("?").is_some());
    }

    // ConsoleInputPumpTests: the prompt/EOF rendezvous tests. `Counting` counts read_line entries
    // (the reader's call count).
    struct Counting {
        rx: mpsc::Receiver<String>,
        reads: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl io::Read for Counting {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl BufRead for Counting {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _: usize) {}
        fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            match self.rx.recv() {
                Ok(l) => {
                    buf.push_str(&l);
                    Ok(l.len())
                }
                Err(_) => Ok(0), // sender dropped: EOF
            }
        }
    }

    fn wait_until(condition: impl Fn() -> bool, message: &str) {
        let t = Instant::now();
        while !condition() {
            assert!(t.elapsed() < Duration::from_secs(5), "{message}");
            thread::yield_now();
        }
    }

    fn counting_pump(
        lookup: impl Fn(&str) -> Option<ConsoleAction> + Send + Sync + 'static,
    ) -> (
        Arc<ConsoleInputPump>,
        mpsc::Sender<String>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let (tx, rx) = mpsc::channel();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let p = Arc::new(ConsoleInputPump::new(
            Box::new(Counting {
                rx,
                reads: reads.clone(),
            }),
            lookup,
            Arc::new(AtomicBool::new(false)),
        ));
        (p, tx, reads)
    }

    #[test]
    fn pending_prompt_receives_next_line() {
        let (p, tx, _) = counting_pump(|_| None);
        let run = {
            let p = p.clone();
            thread::spawn(move || p.run())
        };
        let prompt = {
            let p = p.clone();
            thread::spawn(move || p.read_line())
        };
        wait_until(|| p.has_pending_prompt(), "prompt did not register");
        tx.send("the answer\n".to_string()).unwrap();

        assert_eq!(prompt.join().unwrap().as_deref(), Some("the answer"));

        drop(tx);
        run.join().unwrap();
    }

    #[test]
    fn eof_while_prompt_pending_completes_prompt_with_null() {
        let (p, tx, _) = counting_pump(|_| None);
        let run = {
            let p = p.clone();
            thread::spawn(move || p.run())
        };
        let prompt = {
            let p = p.clone();
            thread::spawn(move || p.read_line())
        };
        wait_until(|| p.has_pending_prompt(), "prompt did not register");
        drop(tx); // EOF while the prompt is waiting

        assert_eq!(prompt.join().unwrap(), None);
        run.join().unwrap();
        assert!(!p.running());
    }

    #[test]
    fn throwing_lookup_does_not_hang_pending_prompt() {
        let (p, tx, reads) = counting_pump(|_| panic!("boom"));
        let run = {
            let p = p.clone();
            thread::spawn(move || p.run())
        };

        // wait until the reader loop is blocked on its first read, then feed a bad command
        wait_until(|| reads.load(Ordering::SeqCst) >= 1, "reader did not start");
        tx.send("badcommand\n".to_string()).unwrap();

        // the loop has consumed that command (throwing lookup swallowed) and blocks on the next line
        wait_until(
            || reads.load(Ordering::SeqCst) >= 2,
            "bad command was not consumed",
        );

        let prompt = {
            let p = p.clone();
            thread::spawn(move || p.read_line())
        };
        wait_until(|| p.has_pending_prompt(), "prompt did not register");
        drop(tx); // EOF while the prompt is waiting

        assert_eq!(prompt.join().unwrap(), None);
        run.join().unwrap();
        assert!(!p.running());
    }
}
