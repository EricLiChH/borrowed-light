use monitor_domain::{MonitorTarget, TargetError};

#[test]
fn learner_can_create_and_read_an_https_monitor_target() {
    let target = MonitorTarget::new("Rust", "https://www.rust-lang.org")
        .expect("a named HTTPS target should be valid");

    assert_eq!(target.name(), "Rust");
    assert_eq!(target.url().scheme(), "https");
    assert_eq!(target.url().host_str(), Some("www.rust-lang.org"));
}

#[test]
fn learner_gets_a_trimmed_name_and_a_normalized_url() {
    // Validation and normalization happen once, at the boundary. The url crate
    // lowercases the scheme and host, and an empty path becomes `/`. That is
    // why a stored target is compared as `https://www.rust-lang.org/`.
    let target = MonitorTarget::new("  Rust  ", "  HTTPS://WWW.Rust-Lang.org  ")
        .expect("case differences are normalized, not rejected");

    assert_eq!(target.name(), "Rust");
    assert_eq!(target.url_str(), "https://www.rust-lang.org/");
}

#[test]
fn learner_can_monitor_ipv6_hosts_and_custom_ports() {
    // Hand-rolled string splitting usually gets these two wrong; a real URL
    // parser does not.
    let target = MonitorTarget::new("local", "http://[::1]:8080/health")
        .expect("an IPv6 host with a port should be valid");

    assert_eq!(target.url().port(), Some(8080));
    assert_eq!(target.url().path(), "/health");
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
fn learner_gets_a_specific_error_for_a_url_with_spaces() {
    let error = MonitorTarget::new("Broken", "http://exam ple.com")
        .expect_err("a space inside the host should be rejected");

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
    for url in [
        "https://?query",
        "https://#fragment",
        "https://:443",
        "https://user@",
    ] {
        let error = MonitorTarget::new("Missing host", url)
            .expect_err("a URL without a host should be rejected");

        assert_eq!(error, TargetError::InvalidUrl, "input was {url}");
    }
}
