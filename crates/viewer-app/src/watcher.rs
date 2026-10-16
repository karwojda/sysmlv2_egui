//! Polls the open Model folder and asks for a reload when its Model files
//! have changed and settled. Dropping the watcher stops its thread.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use viewer_core::watch::{ChangeDetector, fingerprint};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub struct Watcher {
    folder: Arc<Mutex<Option<PathBuf>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Watcher {
    /// Starts polling; `on_change` runs on the watcher's thread with the
    /// folder to reload.
    pub fn new(on_change: impl Fn(PathBuf) + Send + 'static) -> Self {
        let folder: Arc<Mutex<Option<PathBuf>>> = Arc::default();
        let stop: Arc<AtomicBool> = Arc::default();
        let (shared, stopping) = (folder.clone(), stop.clone());
        let thread = std::thread::Builder::new()
            .name("model-watcher".to_owned())
            .spawn(move || {
                let mut watched: Option<(PathBuf, ChangeDetector)> = None;
                while !stopping.load(Ordering::SeqCst) {
                    std::thread::sleep(POLL_INTERVAL);
                    let Some(folder) = shared
                        .lock()
                        .expect("the folder lock is never poisoned")
                        .clone()
                    else {
                        continue;
                    };
                    let Ok(current) = fingerprint(&folder) else {
                        continue;
                    };
                    match &mut watched {
                        Some((path, detector)) if *path == folder => {
                            if detector.observe(current) {
                                on_change(folder);
                            }
                        }
                        _ => watched = Some((folder, ChangeDetector::new(current))),
                    }
                }
            })
            .expect("the watcher thread starts");
        Self {
            folder,
            stop,
            thread: Some(thread),
        }
    }

    /// Watches `folder` from now on, taking its current files as loaded.
    pub fn watch(&self, folder: PathBuf) {
        *self
            .folder
            .lock()
            .expect("the folder lock is never poisoned") = Some(folder);
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
