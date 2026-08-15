use monitor_domain::{CheckResult, MonitorTarget};
use monitor_store::{InMemoryRepository, MonitorRepository, SqliteRepository};

#[tokio::test]
async fn in_memory_rejects_a_result_for_a_different_target() {
    assert_mismatched_result_is_rejected(&InMemoryRepository::new()).await;
}

#[tokio::test]
async fn sqlite_rejects_a_result_for_a_different_target() {
    let repository = SqliteRepository::connect("sqlite::memory:")
        .await
        .expect("temporary database should open");
    assert_mismatched_result_is_rejected(&repository).await;
}

async fn assert_mismatched_result_is_rejected(repository: &dyn MonitorRepository) {
    let first =
        MonitorTarget::new("first", "https://first.example").expect("first target should be valid");
    let second = MonitorTarget::new("second", "https://second.example")
        .expect("second target should be valid");
    let stored = repository
        .add_target(first)
        .await
        .expect("target should be stored");
    let result = CheckResult::reachable(&second, 200);

    let error = repository
        .save_result(stored.id(), &result)
        .await
        .expect_err("mismatched result should be rejected");

    assert!(error.to_string().contains("does not belong"));
}
