use monitor_domain::{CheckResult, MonitorTarget};
use monitor_store::{InMemoryRepository, MonitorRepository, SqliteRepository, StoreError};

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

/// Every adapter has to honour the same contract, so the assertion is generic
/// over the seam instead of being copied into each adapter's test file.
///
/// The parameter used to be `&dyn MonitorRepository`. Native async trait
/// methods are not dyn compatible, and that is the trade this crate made on
/// purpose: no boxing per call, at the cost of writing `R: MonitorRepository`.
async fn assert_mismatched_result_is_rejected<R: MonitorRepository>(repository: &R) {
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

    assert!(matches!(error, StoreError::TargetMismatch { target_id } if target_id == stored.id()));
    assert!(error.to_string().contains("does not belong"));
}

#[tokio::test]
async fn saving_a_result_for_an_unknown_target_reports_not_found() {
    let repository = InMemoryRepository::new();
    let target = MonitorTarget::new("ghost", "https://ghost.example").expect("valid target");

    let error = repository
        .save_result(42, &CheckResult::reachable(&target, 200))
        .await
        .expect_err("an unknown identifier should be rejected");

    // "Not found" and "the database broke" are now different values, so the Web
    // layer can answer 404 instead of 500.
    assert!(matches!(error, StoreError::NotFound { target_id } if target_id == 42));
}
