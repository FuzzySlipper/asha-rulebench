use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rulebench_protocol::{ContentImportAttemptDto, ContentWorkspaceDto, PROTOCOL_VERSION};

#[test]
fn authored_v3_survives_a_new_durable_host_process() {
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-v3-process-restart-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock follows the Unix epoch")
            .as_nanos()
    ));
    let address = available_loopback_address();
    let mut first = HostProcess::start(address, &directory);
    first.wait_until_ready();

    let payload = include_str!("../src/fixtures/authored-content-v3.json");
    let body = serde_json::to_vec(&serde_json::json!({
        "authoredPayload": payload,
        "replacementPolicy": "reject"
    }))
    .expect("import request serializes");
    let imported = request(address, "POST", "/api/rulebench/v1/content/import", &body);
    let imported: ContentImportAttemptDto =
        serde_json::from_slice(&imported).expect("first-process import response is JSON");
    assert!(imported.accepted, "{imported:?}");
    let reference = imported
        .outcome
        .expect("accepted import has a stored review")
        .review
        .pack
        .reference;
    first.stop();

    let second_address = available_loopback_address();
    let mut second = HostProcess::start(second_address, &directory);
    second.wait_until_ready();
    let workspace = request(second_address, "GET", "/api/rulebench/v1/content", &[]);
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace).expect("second-process workspace response is JSON");
    assert_eq!(workspace.packs.len(), 1);
    assert_eq!(workspace.packs[0].reference, reference);
    assert!(workspace.packs[0]
        .definitions
        .iter()
        .any(|definition| definition.kind == "action"));
    second.stop();
    fs::remove_dir_all(directory).expect("durable process fixture cleans up");
}

struct HostProcess {
    address: SocketAddr,
    child: Child,
}

impl HostProcess {
    fn start(address: SocketAddr, artifact_root: &Path) -> Self {
        let address_string = address.to_string();
        let artifact_root = artifact_root
            .to_str()
            .expect("temporary artifact path is UTF-8");
        let child = Command::new(env!("CARGO_BIN_EXE_rulebench_process_host"))
            .args([
                "--bind",
                address_string.as_str(),
                "--artifact-root",
                artifact_root,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("durable process host starts");
        Self { address, child }
    }

    fn wait_until_ready(&mut self) {
        for _ in 0..100 {
            if let Some(status) = self.child.try_wait().expect("host status is readable") {
                panic!("durable process host exited before readiness: {status}");
            }
            if TcpStream::connect_timeout(&self.address, Duration::from_millis(25)).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        panic!("durable process host did not bind {}", self.address);
    }

    fn stop(&mut self) {
        if self
            .child
            .try_wait()
            .expect("host status is readable")
            .is_none()
        {
            self.child.kill().expect("durable process host stops");
        }
        self.child.wait().expect("durable process host is reaped");
    }
}

impl Drop for HostProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn available_loopback_address() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral loopback port binds");
    listener
        .local_addr()
        .expect("ephemeral loopback address is available")
}

fn request(address: SocketAddr, method: &str, path: &str, body: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .expect("process host accepts a request");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("request read timeout configures");
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nx-rulebench-protocol-version: {PROTOCOL_VERSION}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .expect("request writes");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).expect("response reads");
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("response has an HTTP header");
    let header = std::str::from_utf8(&response[..header_end]).expect("response header is UTF-8");
    assert!(header.starts_with("HTTP/1.1 200"), "{header}");
    response[header_end..].to_vec()
}
