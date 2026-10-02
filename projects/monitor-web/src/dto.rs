use monitor_domain::{CheckOutcome, CheckResult};
use monitor_store::StoredTarget;
use serde::{Deserialize, Serialize};

/// The request body for creating a target.
#[derive(Debug, Deserialize)]
pub(crate) struct CreateTarget {
    pub(crate) name: String,
    pub(crate) url: String,
}

/// A stored target as the API shows it.
#[derive(Debug, Serialize)]
pub(crate) struct TargetView {
    id: i64,
    name: String,
    url: String,
}

impl From<&StoredTarget> for TargetView {
    fn from(stored: &StoredTarget) -> Self {
        Self {
            id: stored.id(),
            name: stored.target().name().to_owned(),
            url: stored.target().url_str().to_owned(),
        }
    }
}

/// One check result as the API shows it.
///
/// `failure` is filled from `CheckFailureKind::as_str`, the same call the
/// database adapter uses. Before, this mapping was written out by hand in three
/// crates, so a new failure category could silently appear in the database and
/// be missing from the API.
#[derive(Debug, Serialize)]
pub(crate) struct CheckView {
    target_id: i64,
    name: String,
    url: String,
    reachable: bool,
    status: Option<u16>,
    failure: Option<&'static str>,
    reason: Option<String>,
}

impl CheckView {
    pub(crate) fn new(target_id: i64, result: &CheckResult) -> Self {
        let (reachable, status, failure, reason) = match result.outcome() {
            CheckOutcome::Reachable { status } => (true, Some(*status), None, None),
            CheckOutcome::Unreachable { kind, reason } => {
                (false, None, Some(kind.as_str()), Some(reason.clone()))
            }
        };

        Self {
            target_id,
            name: result.target_name().to_owned(),
            url: result.target_url().as_str().to_owned(),
            reachable,
            status,
            failure,
            reason,
        }
    }
}
