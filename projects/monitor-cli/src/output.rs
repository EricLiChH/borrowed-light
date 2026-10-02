use monitor_domain::{CheckOutcome, CheckResult, MonitorTarget};
use serde::Serialize;

/// A target as the CLI prints it.
#[derive(Debug, Serialize)]
pub struct TargetOutput<'a> {
    /// The learner-facing name.
    pub name: &'a str,
    /// The normalized URL.
    pub url: &'a str,
}

impl<'a> From<&'a MonitorTarget> for TargetOutput<'a> {
    fn from(target: &'a MonitorTarget) -> Self {
        Self {
            name: target.name(),
            url: target.url_str(),
        }
    }
}

/// A check result as the CLI prints it.
///
/// `failure` is filled from `CheckFailureKind::as_str`, the same call the
/// database adapter and the HTTP API use.
#[derive(Debug, Serialize)]
pub struct CheckOutput<'a> {
    /// The learner-facing name.
    pub name: &'a str,
    /// The normalized URL.
    pub url: &'a str,
    /// Whether the site answered at all.
    pub reachable: bool,
    /// The HTTP status, when one came back.
    pub status: Option<u16>,
    /// The stable failure category, when the check failed.
    pub failure: Option<&'static str>,
    /// The human-readable diagnostic detail.
    pub reason: Option<&'a str>,
}

impl<'a> From<&'a CheckResult> for CheckOutput<'a> {
    fn from(result: &'a CheckResult) -> Self {
        let (reachable, status, failure, reason) = match result.outcome() {
            CheckOutcome::Reachable { status } => (true, Some(*status), None, None),
            CheckOutcome::Unreachable { kind, reason } => {
                (false, None, Some(kind.as_str()), Some(reason.as_str()))
            }
        };

        Self {
            name: result.target_name(),
            url: result.target_url().as_str(),
            reachable,
            status,
            failure,
            reason,
        }
    }
}

/// Prints a value as pretty JSON on stdout.
///
/// # Errors
///
/// Returns the serialization error when the value cannot be encoded.
pub fn print_json(value: &impl Serialize) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
