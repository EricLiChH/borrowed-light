use std::fmt;
use std::str::FromStr;

use url::Url;

use crate::MonitorTarget;

/// The observable outcome of one website check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The website returned an HTTP response.
    Reachable {
        /// The HTTP status code that came back.
        status: u16,
    },
    /// The website could not be checked.
    Unreachable {
        /// Stable category callers can match without parsing prose.
        kind: CheckFailureKind,
        /// Human-readable diagnostic detail.
        reason: String,
    },
}

impl CheckOutcome {
    /// Returns whether an HTTP response was observed.
    #[must_use]
    pub const fn is_reachable(&self) -> bool {
        matches!(self, Self::Reachable { .. })
    }

    /// Returns the HTTP status code, when one was observed.
    #[must_use]
    pub const fn status(&self) -> Option<u16> {
        match self {
            Self::Reachable { status } => Some(*status),
            Self::Unreachable { .. } => None,
        }
    }

    /// Returns the failure category, when the check failed.
    #[must_use]
    pub const fn failure_kind(&self) -> Option<CheckFailureKind> {
        match self {
            Self::Reachable { .. } => None,
            Self::Unreachable { kind, .. } => Some(*kind),
        }
    }
}

impl fmt::Display for CheckOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reachable { status } => write!(formatter, "reachable (HTTP {status})"),
            Self::Unreachable { kind, reason } => {
                write!(formatter, "unreachable ({kind}: {reason})")
            }
        }
    }
}

/// Stable categories for failed website checks.
///
/// This is the single source of truth for the spelling of each category on the
/// wire and in the database. Every layer that needs the stable name calls
/// [`CheckFailureKind::as_str`] or parses it back with [`str::parse`], so the
/// mapping cannot drift between the CLI, the web API and storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckFailureKind {
    /// The request exceeded the configured deadline.
    Timeout,
    /// A connection could not be established.
    Connect,
    /// Another request-layer failure occurred.
    Request,
}

impl CheckFailureKind {
    /// Every category, in a stable order, for exhaustive tests and listings.
    pub const ALL: [Self; 3] = [Self::Timeout, Self::Connect, Self::Request];

    /// Returns the stable lower-case name used on the wire and in storage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Connect => "connect",
            Self::Request => "request",
        }
    }
}

impl fmt::Display for CheckFailureKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CheckFailureKind {
    type Err = ParseFailureKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "timeout" => Ok(Self::Timeout),
            "connect" => Ok(Self::Connect),
            "request" => Ok(Self::Request),
            _other => Err(ParseFailureKindError),
        }
    }
}

/// A stored failure category was not one this version understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseFailureKindError;

impl fmt::Display for ParseFailureKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unknown check failure category")
    }
}

impl std::error::Error for ParseFailureKindError {}

/// An owned snapshot of one check.
///
/// A result carries the target it was produced for. Callers therefore never
/// keep a name and a URL in sync by hand, and storage can verify that a result
/// is saved against the target that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    target: MonitorTarget,
    outcome: CheckOutcome,
}

impl CheckResult {
    /// Records a reachable HTTP response while only borrowing the target.
    #[must_use]
    pub fn reachable(target: &MonitorTarget, status: u16) -> Self {
        Self {
            target: target.clone(),
            outcome: CheckOutcome::Reachable { status },
        }
    }

    /// Records why a website could not be checked while only borrowing the target.
    #[must_use]
    pub fn unreachable(target: &MonitorTarget, reason: impl Into<String>) -> Self {
        Self::unreachable_with(target, CheckFailureKind::Request, reason)
    }

    /// Records a classified website check failure while only borrowing the target.
    #[must_use]
    pub fn unreachable_with(
        target: &MonitorTarget,
        kind: CheckFailureKind,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            target: target.clone(),
            outcome: CheckOutcome::Unreachable {
                kind,
                reason: reason.into(),
            },
        }
    }

    /// Borrows the target this result belongs to.
    #[must_use]
    pub const fn target(&self) -> &MonitorTarget {
        &self.target
    }

    /// Returns the name stored in this snapshot.
    #[must_use]
    pub fn target_name(&self) -> &str {
        self.target.name()
    }

    /// Returns the URL stored in this snapshot.
    #[must_use]
    pub const fn target_url(&self) -> &Url {
        self.target.url()
    }

    /// Borrows the recorded outcome.
    #[must_use]
    pub const fn outcome(&self) -> &CheckOutcome {
        &self.outcome
    }

    /// Returns whether the recorded check observed an HTTP response.
    #[must_use]
    pub const fn is_reachable(&self) -> bool {
        self.outcome.is_reachable()
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.target, self.outcome)
    }
}

/// An in-memory sequence of check results, oldest first.
#[derive(Debug, Default, Clone)]
pub struct CheckHistory {
    results: Vec<CheckResult>,
}

impl CheckHistory {
    /// Creates empty history.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Transfers ownership of a result into history.
    pub fn record(&mut self, result: CheckResult) {
        self.results.push(result);
    }

    /// Borrows the most recently recorded result.
    #[must_use]
    pub fn latest(&self) -> Option<&CheckResult> {
        self.results.last()
    }

    /// Borrows every recorded result, oldest first.
    #[must_use]
    pub fn results(&self) -> &[CheckResult] {
        &self.results
    }

    /// Iterates over recorded results, oldest first.
    pub fn iter(&self) -> std::slice::Iter<'_, CheckResult> {
        self.results.iter()
    }

    /// Returns the number of recorded results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Returns whether no result has been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

impl<'a> IntoIterator for &'a CheckHistory {
    type Item = &'a CheckResult;
    type IntoIter = std::slice::Iter<'a, CheckResult>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.iter()
    }
}
