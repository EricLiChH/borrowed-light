use std::process::Command;

#[test]
fn learner_can_validate_a_target_from_the_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_monitor"))
        .args(["target", "Rust", "https://www.rust-lang.org"])
        .output()
        .expect("monitor binary should run");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["name"], "Rust");
    assert_eq!(json["url"], "https://www.rust-lang.org");
}
