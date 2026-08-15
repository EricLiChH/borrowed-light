use monitor_domain::{CheckHistory, CheckResult, MonitorTarget};

fn record_then_borrow_latest(
    history: &mut CheckHistory,
    result: CheckResult,
) -> Option<&CheckResult> {
    let _ = (history, result);
    panic!("TODO: move result into history, then borrow the latest entry")
}

#[test]
#[ignore = "learning checkpoint: remove this attribute before solving"]
fn learner_moves_a_result_into_history_then_borrows_it_back() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org")
        .expect("the target should be valid");
    let result = CheckResult::reachable(&target, 200);
    let mut history = CheckHistory::new();

    let latest = record_then_borrow_latest(&mut history, result)
        .expect("recording should make a latest result available");

    assert_eq!(latest.target_name(), "Rust");
    assert_eq!(history.len(), 1);
}
