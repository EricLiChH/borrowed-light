use monitor_domain::{CheckFailureKind, CheckHistory, CheckOutcome, CheckResult, MonitorTarget};

#[test]
fn learner_can_move_a_successful_result_into_history_and_borrow_it_back() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org").unwrap();
    let result = CheckResult::reachable(&target, 200);
    let mut history = CheckHistory::new();

    history.record(result);

    let latest = history.latest().expect("the recorded result should exist");
    assert_eq!(latest.target_name(), "Rust");
    assert_eq!(latest.target_url(), "https://www.rust-lang.org");
    assert_eq!(latest.outcome(), &CheckOutcome::Reachable { status: 200 });
    assert_eq!(history.len(), 1);
    assert_eq!(
        target.name(),
        "Rust",
        "creating a result only borrows the target"
    );
}

#[test]
fn learner_can_record_a_failed_check_without_consuming_the_target() {
    let target = MonitorTarget::new("Example", "https://example.com").unwrap();
    let result = CheckResult::unreachable(&target, "request timed out");

    assert_eq!(
        result.outcome(),
        &CheckOutcome::Unreachable {
            kind: CheckFailureKind::Request,
            reason: "request timed out".to_owned(),
        }
    );
    assert_eq!(target.url(), "https://example.com");
}
