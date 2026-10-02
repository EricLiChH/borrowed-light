use std::process::{Command, Output};

fn monitor(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_monitor"))
        .env("MONITOR_DISABLE_PROXY", "1")
        .args(arguments)
        .output()
        .expect("monitor binary should run")
}

#[test]
fn a_failed_check_exits_with_code_one_and_still_prints_json() {
    // Port 1 on loopback refuses connections immediately, so this needs no
    // timeout and touches no network.
    let output = monitor(&["check", "dead", "http://127.0.0.1:1", "--attempts", "1"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "a failed check is not a crash"
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should still be JSON");
    assert_eq!(json["reachable"], false);
    assert_eq!(json["name"], "dead");
    assert!(json["failure"].is_string());
}

#[test]
fn an_invalid_target_exits_with_code_two() {
    let output = monitor(&["check", "bad", "not a url"]);

    assert_eq!(
        output.status.code(),
        Some(2),
        "usage errors are exit code 2"
    );
    assert!(output.stdout.is_empty(), "nothing is printed on stdout");
}

#[test]
fn zero_attempts_is_rejected_by_argument_parsing() {
    // Attempts is a NonZeroUsize, so "0" cannot even be parsed into the policy.
    let output = monitor(&["check", "x", "http://127.0.0.1:1", "--attempts", "0"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_batch_reports_one_exit_code_for_the_whole_run() {
    let config = serde_json::json!([
        { "name": "dead", "url": "http://127.0.0.1:1" }
    ]);
    let path =
        std::env::temp_dir().join(format!("borrowed-light-exit-{}.json", std::process::id()));
    std::fs::write(&path, config.to_string()).expect("temporary config should be written");

    let output = monitor(&[
        "batch",
        path.to_str().expect("temporary path should be UTF-8"),
        "--attempts",
        "1",
    ]);
    std::fs::remove_file(&path).expect("temporary config should be removed");

    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json.as_array().map(Vec::len), Some(1));
    assert_eq!(json[0]["reachable"], false);
}
