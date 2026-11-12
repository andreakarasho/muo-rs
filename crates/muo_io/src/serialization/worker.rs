//! The save worker.
//!
//! The game thread composes each persistence's bytes (`EntityFileWriter`), and one background thread writes
//! files, in the order they were queued, while the game goes on. `wait_idle` is the pause/handshake.

use crate::error::{IoError, Result};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() -> Result<()> + Send>;

#[derive(Default)]
struct State {
    pending: usize,
    errors: Vec<IoError>,
}

pub struct SaveWorker {
    tx: Option<Sender<Job>>,
    handle: Option<JoinHandle<()>>,
    state: Arc<(Mutex<State>, Condvar)>,
}

impl SaveWorker {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Job>();
        let state: Arc<(Mutex<State>, Condvar)> = Arc::default();
        let worker_state = Arc::clone(&state);
        let handle = std::thread::spawn(move || {
            for job in rx {
                let result = job();
                let (lock, cv) = &*worker_state;
                let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
                if let Err(e) = result {
                    s.errors.push(e);
                }
                s.pending -= 1;
                cv.notify_all();
            }
        });
        Self {
            tx: Some(tx),
            handle: Some(handle),
            state,
        }
    }

    /// Queues `job`; jobs run one at a time in submission order.
    pub fn submit(&self, job: impl FnOnce() -> Result<()> + Send + 'static) {
        let (lock, _) = &*self.state;
        lock.lock().unwrap_or_else(|e| e.into_inner()).pending += 1;
        if let Some(tx) = &self.tx {
            // The thread only ends when `tx` is dropped, so the send cannot fail while `self` lives
            let _ = tx.send(Box::new(job));
        }
    }

    /// Blocks until everything submitted so far has run.
    pub fn wait_idle(&self) {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        while s.pending > 0 {
            s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Errors of finished jobs (a failed save must not publish: the caller checks this before moving the save on).
    pub fn take_errors(&self) -> Vec<IoError> {
        let (lock, _) = &*self.state;
        std::mem::take(&mut lock.lock().unwrap_or_else(|e| e.into_inner()).errors)
    }
}

impl Default for SaveWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SaveWorker {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn runs_in_order_and_collects_errors() {
        let w = SaveWorker::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        for i in 0..5 {
            let log = Arc::clone(&log);
            w.submit(move || {
                log.lock().unwrap().push(i);
                Ok(())
            });
        }
        w.submit(|| Err(IoError::InvalidData("boom".into())));
        w.wait_idle();
        assert_eq!(*log.lock().unwrap(), [0, 1, 2, 3, 4]);
        assert_eq!(w.take_errors().len(), 1);
        assert!(w.take_errors().is_empty());

        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        w.submit(move || {
            n2.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        drop(w); // joins
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }
}
