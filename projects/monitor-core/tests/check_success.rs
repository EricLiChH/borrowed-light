mod common;

use std::time::Duration;

use monitor_core::{Backoff, CheckPolicy, HealthChecker};
use monitor_domain::{CheckOutcome, MonitorTarget};

#[tokio::test]
async fn learner_can_check_a_local_website_without_using_the_public_internet() {
    let url = common::serve_once("HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n").await;
    let target = MonitorTarget::new("local", url).expect("local URL should be valid");
    let policy = CheckPolicy::builder()
        .total_timeout(Duration::from_secs(1))
        .max_attempts(common::nonzero(1))
        .backoff(Backoff::none())
        .concurrency(common::nonzero(1))
        .build();
    let checker = HealthChecker::new(common::client(), policy);

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 204 });
}
