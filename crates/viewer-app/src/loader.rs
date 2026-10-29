//! Loads Models on a worker thread that owns the backend, so the window
//! stays responsive while the service parses. Dropping the loader stops the
//! worker, which drops the backend and with it the sidecar.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use viewer_core::backend::Analyses;
use viewer_core::{BackendError, Model, ModelBackend};

/// What to load: a Model folder, and which analyses to run.
pub type Request = (PathBuf, Analyses);

pub struct Loaded {
    pub folder: PathBuf,
    pub result: Result<Model, BackendError>,
    /// What loaded it, as the backend names itself after the load.
    pub loaded_by: Option<String>,
}

pub struct Loader {
    requests: Option<Sender<Request>>,
    results: Receiver<Loaded>,
    worker: Option<JoinHandle<()>>,
}

impl Loader {
    /// Starts the worker. `notify` runs after each load, from the worker
    /// thread, so the UI can wake up and collect the result.
    pub fn new(
        mut backend: Box<dyn ModelBackend + Send>,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let (requests, incoming) = mpsc::channel::<Request>();
        let (outgoing, results) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("model-loader".to_owned())
            .spawn(move || {
                for (folder, analyses) in incoming {
                    let result = backend.load_with(&folder, &analyses);
                    let loaded_by = backend.loaded_by();
                    let loaded = Loaded {
                        folder,
                        result,
                        loaded_by,
                    };
                    if outgoing.send(loaded).is_err() {
                        break;
                    }
                    notify();
                }
            })
            .expect("the loader thread starts");
        Self {
            requests: Some(requests),
            results,
            worker: Some(worker),
        }
    }

    /// Loads `folder`, running analyses as its settings say.
    pub fn load(&self, folder: PathBuf) {
        self.load_with(folder, Analyses::AsSet);
    }

    /// Loads `folder`, running the analyses `analyses` asks for.
    pub fn load_with(&self, folder: PathBuf, analyses: Analyses) {
        if let Some(requests) = &self.requests {
            let _ = requests.send((folder, analyses));
        }
    }

    /// A handle other threads can use to ask for loads.
    pub fn requester(&self) -> Option<Sender<Request>> {
        self.requests.clone()
    }

    /// The next finished load, if any.
    pub fn poll(&self) -> Option<Loaded> {
        self.results.try_recv().ok()
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.requests.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
