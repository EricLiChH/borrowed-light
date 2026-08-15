use monitor_domain::{CheckOutcome, CheckResult, MonitorTarget};
use monitor_store::{InMemoryRepository, MonitorRepository};

#[tokio::test]
async fn learner_can_add_and_list_targets_in_memory() {
    let repository = InMemoryRepository::new();
    let target =
        MonitorTarget::new("Rust", "https://www.rust-lang.org").expect("target should be valid");

    let stored = repository
        .add_target(target)
        .await
        .expect("target should be stored");
    let targets = repository
        .list_targets()
        .await
        .expect("targets should be listed");

    assert_eq!(stored.id(), 1);
    assert_eq!(stored.target().name(), "Rust");
    assert_eq!(targets, vec![stored]);
}

#[tokio::test]
async fn learner_can_record_and_read_the_latest_result_in_memory() {
    let repository = InMemoryRepository::new();
    let target =
        MonitorTarget::new("Rust", "https://www.rust-lang.org").expect("target should be valid");
    let stored = repository
        .add_target(target.clone())
        .await
        .expect("target should be stored");

    repository
        .save_result(stored.id(), &CheckResult::reachable(&target, 204))
        .await
        .expect("result should be stored");
    let latest = repository
        .latest_result(stored.id())
        .await
        .expect("result should be readable")
        .expect("latest result should exist");

    assert_eq!(latest.outcome(), &CheckOutcome::Reachable { status: 204 });
}
