use std::time::Duration;

use reqwest::header::HeaderMap;

/// Set this environment variable to bypass system proxy discovery.
///
/// Tests talk to `127.0.0.1`, and a developer machine with a global proxy would
/// otherwise route those requests somewhere else.
pub const DISABLE_PROXY_ENV: &str = "MONITOR_DISABLE_PROXY";

/// Builds the async client used by `monitor`.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] when the TLS backend or the client
/// configuration cannot be initialised.
pub fn async_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder();
    if proxy_is_disabled() {
        builder = builder.no_proxy();
    }
    builder.build()
}

/// Builds the blocking client used by `monitor-sync`.
///
/// The blocking client gets an explicit timeout, because a sequential checker
/// has no outer deadline to fall back on.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] when the client configuration cannot be
/// initialised.
pub fn blocking_client(timeout: Duration) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut builder = reqwest::blocking::Client::builder().timeout(timeout);
    if proxy_is_disabled() {
        builder = builder.no_proxy();
    }
    builder.build()
}

/// Returns whether a response status is worth retrying, for callers that do
/// their own retry loop.
#[must_use]
pub fn retry_after_seconds(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn proxy_is_disabled() -> bool {
    std::env::var_os(DISABLE_PROXY_ENV).is_some()
}
