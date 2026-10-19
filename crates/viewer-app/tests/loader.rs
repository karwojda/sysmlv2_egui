//! The background loader delivers Models and releases the backend on drop.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use viewer_app::loader::Loader;
use viewer_core::{BackendError, FakeBackend, Model, ModelBackend, QualifiedName};

fn wait_for_load(loader: &Loader, notified: &mpsc::Receiver<()>) -> viewer_app::loader::Loaded {
    notified
        .recv_timeout(Duration::from_secs(5))
        .expect("the loader notifies");
    loader
        .poll()
        .expect("a result is ready after the notification")
}

#[test]
fn delivers_the_loaded_model_for_the_folder_asked_for() {
    let model = Model::builder().package("P").build();
    let (notify, notified) = mpsc::channel();
    let loader = Loader::new(Box::new(FakeBackend::new(model)), move || {
        let _ = notify.send(());
    });

    loader.load(PathBuf::from("models/p"));
    let loaded = wait_for_load(&loader, &notified);

    assert_eq!(loaded.folder, PathBuf::from("models/p"));
    assert!(
        loaded
            .result
            .unwrap()
            .element(&QualifiedName::from("P"))
            .is_some()
    );
}

#[test]
fn delivers_backend_errors() {
    struct Failing;
    impl ModelBackend for Failing {
        fn load_with(
            &mut self,
            folder: &Path,
            _analyses: &viewer_core::backend::Analyses,
        ) -> Result<Model, BackendError> {
            Err(BackendError::FolderNotFound(folder.to_path_buf()))
        }
    }
    let (notify, notified) = mpsc::channel();
    let loader = Loader::new(Box::new(Failing), move || {
        let _ = notify.send(());
    });

    loader.load(PathBuf::from("missing"));

    assert!(matches!(
        wait_for_load(&loader, &notified).result,
        Err(BackendError::FolderNotFound(_))
    ));
}

#[test]
fn dropping_the_loader_drops_the_backend() {
    struct Tracked(Arc<AtomicBool>);
    impl ModelBackend for Tracked {
        fn load_with(
            &mut self,
            _: &Path,
            _analyses: &viewer_core::backend::Analyses,
        ) -> Result<Model, BackendError> {
            Ok(Model::default())
        }
    }
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let loader = Loader::new(Box::new(Tracked(dropped.clone())), || {});

    drop(loader);

    assert!(
        dropped.load(Ordering::SeqCst),
        "the backend, and so the sidecar, is released"
    );
}
