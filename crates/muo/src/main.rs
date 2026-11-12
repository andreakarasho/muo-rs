//! The server executable and the process-level parts of `Core.Setup`: the banner, working directory, logging, Ctrl+C, exit code.

use muo_server::World;
use muo_server::core_loop::{ServerHooks, default_prompt, run_with, setup_with};

mod log;

fn main() {
    log::init();
    // Core.BaseDirectory: where the executable is. Configuration/, Data/, Saves/ resolve from here.
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_default();
    if !base.as_os_str().is_empty() {
        let _ = std::env::set_current_dir(&base);
    }

    println!("muo Version {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Copyright 2019-2026 ModernUO Development Team\nThis program comes with ABSOLUTELY NO WARRANTY;\nThis is free software, and you are welcome to redistribute it under certain conditions."
    );

    let mut w = World::new_unloaded();
    // Console_CancelKeyPressed -> Core.Kill(): the loop checks the flag through `signals` each tick.
    let signals = w.core.signals.clone();
    let ctx = w.core.loop_context.clone();
    if let Err(e) = ctrlc::set_handler(move || {
        tracing::info!("Detected CTRL+C pressed.");
        signals.kill(false);
        ctx.wake();
    }) {
        tracing::error!("Could not install the Ctrl+C handler: {e}");
    }

    // ConsoleInputHandler.Configure (`help|?`); the `save`, `shutdown` and `restart` commands read the handle in
    // their own Configure. The reader thread starts once the world is up (Initialize).
    let headless = {
        use std::io::IsTerminal;
        !std::io::stdin().is_terminal()
    };
    let console = muo_server::console::ConsoleInputHandler::new(
        headless,
        w.core.signals.closing.clone(), // `Core.Closing`, which the pump loop checks
    );
    console.configure();
    w.resources
        .get_mut::<muo_content::console::ConsoleHandle>()
        .0 = Some(console.clone());

    let mut prompt = default_prompt();
    let mut net = None;
    let r = setup_with(&mut w, base, muo::register_content, &mut prompt, |w| {
        net = muo::start_network(w);
    });
    match r {
        Ok(()) => {
            if let Err(e) = console.initialize() {
                tracing::error!("Could not start the console input thread: {e}");
            }
            let (net, ping, closing) = match net {
                Some((n, p, c)) => (Some(n), Some(p), Some(c)),
                None => (None, None, None),
            };
            let mut hooks = muo::Hooks {
                server: ServerHooks,
                net,
                ping,
                closing,
            };
            raise_timer_resolution();
            std::process::exit(run_with(&mut w, &mut hooks))
        }
        Err(e) => {
            // CurrentDomain_UnhandledException: print and exit.
            eprintln!(
                "Error:
{e}"
            );
            std::process::exit(1);
        }
    }
}

/// Windows wakes sleeps on a ~15.6ms clock by default, so the loop's 2ms idle waits always come back late and the
/// health check suspends sleeping for good (a spinning core). 1ms resolution lasts until the process exits.
#[cfg(windows)]
fn raise_timer_resolution() {
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn timeBeginPeriod(period: u32) -> u32;
    }
    // SAFETY: plain Win32 call with no pointers; Windows undoes it when the process exits.
    if unsafe { timeBeginPeriod(1) } != 0 {
        tracing::warn!(
            "Could not raise the Windows timer resolution; idle waits may come back late"
        );
    }
}

#[cfg(not(windows))]
fn raise_timer_resolution() {}
