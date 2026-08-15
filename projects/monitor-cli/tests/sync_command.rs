use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

#[test]
fn learner_starts_with_a_sequential_blocking_check() {
    let url = serve_once();
    let output = Command::new(env!("CARGO_BIN_EXE_monitor-sync"))
        .env("MONITOR_DISABLE_PROXY", "1")
        .args(["check", "local", &url])
        .output()
        .expect("sync monitor binary should run");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["reachable"], true);
    assert_eq!(json["status"], 204);
}

fn serve_once() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("server should accept");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
            .expect("server should reply");
    });
    format!("http://{address}")
}
