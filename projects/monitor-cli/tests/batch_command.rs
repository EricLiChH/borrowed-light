use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

#[test]
fn learner_can_check_a_json_target_list_with_bounded_concurrency() {
    let url = serve_twice();
    let config_path = std::env::temp_dir().join(format!(
        "borrowed-light-targets-{}.json",
        std::process::id()
    ));
    let config = serde_json::json!([
        {"name": "one", "url": url},
        {"name": "two", "url": url}
    ]);
    fs::write(&config_path, config.to_string()).expect("temporary config should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_monitor"))
        .args([
            "batch",
            config_path
                .to_str()
                .expect("temporary path should be UTF-8"),
            "--concurrency",
            "2",
            "--attempts",
            "1",
        ])
        .output()
        .expect("monitor binary should run");
    fs::remove_file(&config_path).expect("temporary config should be removed");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json.as_array().map(Vec::len), Some(2));
    assert_eq!(json[0]["reachable"], true);
    assert_eq!(json[1]["reachable"], true);
}

fn serve_twice() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("server should accept");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("server should reply");
        }
    });
    format!("http://{address}")
}
