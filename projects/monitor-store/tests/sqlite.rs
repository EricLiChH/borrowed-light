use monitor_domain::{CheckFailureKind, CheckOutcome, CheckResult, MonitorTarget};
use monitor_store::{MonitorRepository, SqliteRepository};

#[tokio::test]
async fn learner_can_replace_memory_with_temporary_sqlite() {
    let repository = SqliteRepository::connect("sqlite::memory:")
        .await
        .expect("temporary database should open");
    let target =
        MonitorTarget::new("Rust", "https://www.rust-lang.org").expect("target should be valid");
    let stored = repository
        .add_target(target.clone())
        .await
        .expect("target should be stored");
    let result =
        CheckResult::unreachable_with(&target, CheckFailureKind::Timeout, "deadline exceeded");

    repository
        .save_result(stored.id(), result)
        .await
        .expect("result should be stored");

    let targets = repository
        .list_targets()
        .await
        .expect("targets should be listed");
    let latest = repository
        .latest_result(stored.id())
        .await
        .expect("result should be readable")
        .expect("latest result should exist");

    assert_eq!(targets, vec![stored]);
    assert!(matches!(
        latest.outcome(),
        CheckOutcome::Unreachable {
            kind: CheckFailureKind::Timeout,
            reason,
        } if reason == "deadline exceeded"
    ));
}
