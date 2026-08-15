use monitor_domain::{MonitorTarget, TargetError};

#[test]
fn learner_can_create_and_read_an_https_monitor_target() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org")
        .expect("a named HTTPS target should be valid");

    assert_eq!(target.name(), "Rust");
    assert_eq!(target.url(), "https://www.rust-lang.org");
}

#[test]
fn learner_gets_a_specific_error_for_a_blank_target_name() {
    let error = MonitorTarget::new("   ", "https://www.rust-lang.org")
        .expect_err("a blank target name should be rejected");

    assert_eq!(error, TargetError::EmptyName);
}

#[test]
fn learner_gets_a_specific_error_when_the_url_has_no_scheme() {
    let error = MonitorTarget::new("Rust", "www.rust-lang.org")
        .expect_err("a URL without a scheme should be rejected");

    assert_eq!(error, TargetError::InvalidUrl);
}

#[test]
fn learner_gets_a_specific_error_for_a_non_http_scheme() {
    let error = MonitorTarget::new("Mirror", "ftp://example.com")
        .expect_err("the monitor only supports HTTP and HTTPS");

    assert_eq!(error, TargetError::UnsupportedScheme);
}

#[test]
fn learner_gets_a_specific_error_when_the_url_has_no_host() {
    for url in ["https://?query", "https://#fragment"] {
        let error = MonitorTarget::new("Missing host", url)
            .expect_err("a URL without a host should be rejected");

        assert_eq!(error, TargetError::InvalidUrl);
    }
}
