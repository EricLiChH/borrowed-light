use monitor_domain::{CheckFailureKind, CheckHistory, CheckOutcome, CheckResult, MonitorTarget};

#[test]
fn learner_can_move_a_successful_result_into_history_and_borrow_it_back() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org").unwrap();
    let result = CheckResult::reachable(&target, 200);
    let mut history = CheckHistory::new();

    history.record(result);

    let latest = history.latest().expect("the recorded result should exist");
    assert_eq!(latest.target_name(), "Rust");
    assert_eq!(latest.target_url().as_str(), "https://www.rust-lang.org/");
    assert_eq!(latest.outcome(), &CheckOutcome::Reachable { status: 200 });
    assert!(latest.is_reachable());
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
    assert_eq!(
        result.outcome().failure_kind(),
        Some(CheckFailureKind::Request)
    );
    assert!(!result.is_reachable());
    assert_eq!(target.url_str(), "https://example.com/");
}

#[test]
fn a_result_remembers_which_target_produced_it() {
    // Storage relies on this to reject a result saved against the wrong target.
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org").unwrap();
    let result = CheckResult::reachable(&target, 204);

    assert_eq!(result.target(), &target);
}

#[test]
fn history_iterates_oldest_first_and_can_be_reused_by_reference() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org").unwrap();
    let mut history = CheckHistory::new();
    history.record(CheckResult::reachable(&target, 200));
    history.record(CheckResult::unreachable(&target, "boom"));

    let statuses = history
        .iter()
        .map(|result| result.outcome().status())
        .collect::<Vec<_>>();

    assert_eq!(statuses, vec![Some(200), None]);
    assert_eq!(history.results().len(), 2);
    assert_eq!((&history).into_iter().count(), 2);
}
