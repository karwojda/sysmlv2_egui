//! Reaching sysml-grpc: a configured binary, a running service, and clear
//! failures. Tests that need the real service are opt-in (`-- --ignored`).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use viewer_core::{BackendError, ModelBackend, QualifiedName};
use viewer_opensysml::{OpenSysmlBackend, ServiceConfig};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn failure(config: ServiceConfig) -> String {
    match OpenSysmlBackend::with_config(config).load(&fixture("vehicles")) {
        Err(BackendError::Unavailable(message)) => message,
        other => panic!("expected the service to be unavailable, got {other:?}"),
    }
}

#[test]
fn a_configured_binary_that_does_not_exist_fails_naming_it() {
    let message = failure(ServiceConfig {
        binary: Some("/nowhere/sysml-grpc".into()),
        service: None,
    });

    assert!(
        message.contains("/nowhere/sysml-grpc") && message.contains("does not exist"),
        "{message}"
    );
}

#[test]
fn an_unreachable_service_fails_naming_its_address() {
    let started = Instant::now();

    let message = failure(ServiceConfig {
        binary: None,
        service: Some("127.0.0.1:1".into()),
    });

    assert!(message.contains("127.0.0.1:1"), "{message}");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "fails promptly"
    );
}

#[test]
fn a_service_address_that_is_not_host_and_port_fails_clearly() {
    for address in ["nonsense", "host:notaport", ":50051"] {
        let message = failure(ServiceConfig {
            binary: None,
            service: Some(address.into()),
        });
        assert!(message.contains("is not host:port"), "{address}: {message}");
    }
}

/// The binary the client caches, or one named by OPENSYSML_GRPC_BINARY.
fn sysml_grpc() -> PathBuf {
    std::env::var_os("OPENSYSML_GRPC_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap()).join(".opensysml/bin/sysml-grpc")
        })
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_running_service_is_used_and_left_running() {
    let mut service = Command::new(sysml_grpc())
        .args([
            "-port",
            "0",
            "-health-port",
            "0",
            "-report-address",
            "-log-level",
            "error",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut address = String::new();
    BufReader::new(service.stdout.take().unwrap())
        .read_line(&mut address)
        .unwrap();
    let address = address.trim().trim_start_matches("http://").to_owned();

    let mut backend = OpenSysmlBackend::with_config(ServiceConfig {
        binary: None,
        service: Some(address.clone()),
    });
    let model = backend.load(&fixture("vehicles")).unwrap();
    drop(backend);

    assert!(
        model
            .element(&QualifiedName::from("Vehicles::Car"))
            .is_some()
    );
    assert!(
        service.try_wait().unwrap().is_none(),
        "the service at {address} keeps running"
    );
    service.kill().unwrap();
}

/// Re-run as a child (see below): start the private service, report its
/// pid, then die without any chance to clean up.
#[test]
#[ignore = "run as a child process by the test below"]
fn child_starts_the_private_service_then_is_killed() {
    if std::env::var_os("VIEWER_SERVICE_CHILD").is_none() {
        return;
    }
    let connection = opensysml::Connection::private().unwrap();
    println!("SIDECAR {}", connection.private_service_pid().unwrap());
    std::thread::sleep(Duration::from_secs(60));
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_killed_viewer_leaves_no_service_behind() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "child_starts_the_private_service_then_is_killed",
            "--ignored",
            "--nocapture",
        ])
        .env("VIEWER_SERVICE_CHILD", "1")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .find_map(|line| {
            line.strip_prefix("SIDECAR ")
                .map(|pid| pid.trim().to_owned())
        })
        .expect("the child reports its service");
    assert!(
        Path::new(&format!("/proc/{pid}")).exists(),
        "the service is running"
    );

    child.kill().unwrap(); // SIGKILL: no destructors, no cleanup
    child.wait().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    while Path::new(&format!("/proc/{pid}")).exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "service {pid} outlived the killed viewer"
    );
}
