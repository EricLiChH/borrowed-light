#[derive(Clone, Copy)]
enum Failure {
    Connect,
    Timeout,
    Redirect,
    Builder,
}

fn should_retry(failure: Failure, attempt: usize, max_attempts: usize) -> bool {
    attempt < max_attempts && matches!(failure, Failure::Connect | Failure::Timeout)
}

fn main() {}

#[test]
fn only_transient_failures_are_retried() {
    assert!(should_retry(Failure::Connect, 1, 3));
    assert!(should_retry(Failure::Timeout, 2, 3));
    assert!(!should_retry(Failure::Redirect, 1, 3));
    assert!(!should_retry(Failure::Builder, 1, 3));
    assert!(!should_retry(Failure::Connect, 3, 3));
}
