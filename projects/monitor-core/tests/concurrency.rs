mod common;

use std::sync::atomic::Ordering;
use std::time::Duration;

use monitor_core::{Backoff, CheckPolicy, HealthChecker};
use monitor_domain::MonitorTarget;

#[tokio::test]
async fn learner_can_bound_batch_concurrency() {
    let probe = common::ConcurrencyProbe::start(4).await;
    let targets = (1..=4)
        .map(|number| {
            MonitorTarget::new(format!("local-{number}"), probe.url.clone())
                .expect("local URL should be valid")
        })
        .collect::<Vec<_>>();
    let policy = CheckPolicy::builder()
        .total_timeout(Duration::from_secs(1))
        .max_attempts(common::nonzero(1))
        .backoff(Backoff::none())
        .concurrency(common::nonzero(2))
        .build();
    let checker = HealthChecker::new(common::client(), policy);

    let batch = tokio::spawn(async move { checker.check_all(&targets).await });
    probe.wait_for_accepted(2).await;
    for _ in 0..4 {
        tokio::task::yield_now().await;
    }

    assert_eq!(probe.accepted.load(Ordering::SeqCst), 2);
    probe.release_all();

    let results = batch.await.expect("batch task should finish");
    assert_eq!(results.len(), 4);
    assert!(probe.maximum.load(Ordering::SeqCst) <= 2);
}

#[tokio::test(start_paused = true)]
async fn batch_results_keep_input_order_when_responses_finish_out_of_order() {
    let slow_url = common::serve_after(Duration::from_secs(10)).await;
    let fast_url = common::serve_after(Duration::ZERO).await;
    let targets = vec![
        MonitorTarget::new("slow-first", slow_url).expect("local URL should be valid"),
        MonitorTarget::new("fast-second", fast_url).expect("local URL should be valid"),
    ];
    let policy = CheckPolicy::builder()
        .total_timeout(Duration::from_secs(20))
        .max_attempts(common::nonzero(1))
        .backoff(Backoff::none())
        .concurrency(common::nonzero(2))
        .build();
    let checker = HealthChecker::new(common::client(), policy);

    let results = checker.check_all(&targets).await;

    let names = results
        .iter()
        .map(monitor_domain::CheckResult::target_name)
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["slow-first", "fast-second"]);
}
