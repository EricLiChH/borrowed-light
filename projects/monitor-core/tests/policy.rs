use std::time::Duration;

use monitor_core::{Backoff, CheckPolicy, Jitter};

#[test]
fn exponential_backoff_grows_until_it_reaches_the_ceiling() {
    let backoff = Backoff::exponential(Duration::from_millis(100)).with_jitter(Jitter::None);

    assert_eq!(backoff.base_delay(1), Duration::from_millis(100));
    assert_eq!(backoff.base_delay(2), Duration::from_millis(200));
    assert_eq!(backoff.base_delay(3), Duration::from_millis(400));
    assert_eq!(backoff.base_delay(20), Backoff::DEFAULT_MAX);
}

#[test]
fn fixed_backoff_never_grows() {
    let backoff = Backoff::fixed(Duration::from_millis(250));

    assert_eq!(backoff.base_delay(1), Duration::from_millis(250));
    assert_eq!(backoff.base_delay(9), Duration::from_millis(250));
}

#[test]
fn full_jitter_scales_the_computed_delay_and_clamps_the_input() {
    let backoff = Backoff::exponential(Duration::from_millis(100))
        .with_max(Duration::from_secs(1))
        .with_jitter(Jitter::Full);

    assert_eq!(backoff.delay(3, 0.0), Duration::ZERO);
    assert_eq!(backoff.delay(3, 0.5), Duration::from_millis(200));
    assert_eq!(backoff.delay(3, 1.0), Duration::from_millis(400));
    assert_eq!(backoff.delay(3, 42.0), Duration::from_millis(400));
}

#[test]
fn backoff_can_be_turned_off_entirely() {
    let backoff = Backoff::none();

    assert!(backoff.is_none());
    assert_eq!(backoff.delay(4, 0.5), Duration::ZERO);
}

#[test]
fn a_policy_can_never_hold_zero_attempts_or_zero_concurrency() {
    // NonZeroUsize makes the old PolicyError unrepresentable: there is nothing
    // left for a constructor to reject.
    let policy = CheckPolicy::builder().build();

    assert_eq!(policy.max_attempts().get(), 2);
    assert_eq!(policy.concurrency().get(), 8);
    assert_eq!(policy.total_timeout(), Duration::from_secs(5));
    assert_eq!(CheckPolicy::default(), policy);
}

#[test]
fn the_builder_can_override_every_field() {
    let policy = CheckPolicy::builder()
        .total_timeout(Duration::from_millis(1500))
        .max_attempts(std::num::NonZeroUsize::MIN)
        .backoff(Backoff::none())
        .concurrency(std::num::NonZeroUsize::MIN)
        .build();

    assert_eq!(policy.total_timeout(), Duration::from_millis(1500));
    assert_eq!(policy.max_attempts().get(), 1);
    assert!(policy.backoff().is_none());
    assert_eq!(policy.concurrency().get(), 1);
}
