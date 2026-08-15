//! Domain types for the website health monitor project.

/// A website selected for health checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorTarget {
    name: String,
    url: String,
}

impl MonitorTarget {
    /// Creates a monitor target.
    ///
    /// # Errors
    ///
    /// Returns [`TargetError`] when the name is blank, the URL is missing a
    /// scheme or host, or the scheme is not HTTP(S).
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Result<Self, TargetError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(TargetError::EmptyName);
        }

        let url = url.into();
        let Some((scheme, authority)) = url.split_once("://") else {
            return Err(TargetError::InvalidUrl);
        };
        if !matches!(scheme, "http" | "https") {
            return Err(TargetError::UnsupportedScheme);
        }

        let Some(host) = host_from_authority(authority) else {
            return Err(TargetError::InvalidUrl);
        };
        if host.trim().is_empty() || host.chars().any(char::is_whitespace) {
            return Err(TargetError::InvalidUrl);
        }

        Ok(Self { name, url })
    }

    /// Returns the learner-facing name without transferring ownership.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target URL without transferring ownership.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

fn host_from_authority(authority_and_rest: &str) -> Option<&str> {
    let authority = authority_and_rest.split(['/', '?', '#']).next()?;
    let host_and_port = authority.rsplit('@').next()?;

    if let Some(bracketed) = host_and_port.strip_prefix('[') {
        let (host, suffix) = bracketed.split_once(']')?;
        if !suffix.is_empty() && !suffix.starts_with(':') {
            return None;
        }
        Some(host)
    } else {
        Some(
            host_and_port
                .split_once(':')
                .map_or(host_and_port, |(host, _port)| host),
        )
    }
}

/// Why a monitor target could not be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetError {
    /// The learner-facing name contained only whitespace.
    EmptyName,
    /// The URL was missing a scheme or host.
    InvalidUrl,
    /// The URL did not use HTTP or HTTPS.
    UnsupportedScheme,
}

/// The observable outcome of one website check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The website returned an HTTP response.
    Reachable { status: u16 },
    /// The website could not be checked.
    Unreachable { reason: String },
}

/// An owned snapshot of one check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    target_name: String,
    target_url: String,
    outcome: CheckOutcome,
}

impl CheckResult {
    /// Records a reachable HTTP response while only borrowing the target.
    #[must_use]
    pub fn reachable(target: &MonitorTarget, status: u16) -> Self {
        Self {
            target_name: target.name().to_owned(),
            target_url: target.url().to_owned(),
            outcome: CheckOutcome::Reachable { status },
        }
    }

    /// Records why a website could not be checked while only borrowing the target.
    #[must_use]
    pub fn unreachable(target: &MonitorTarget, reason: impl Into<String>) -> Self {
        Self {
            target_name: target.name().to_owned(),
            target_url: target.url().to_owned(),
            outcome: CheckOutcome::Unreachable {
                reason: reason.into(),
            },
        }
    }

    /// Returns the target name stored in this snapshot.
    #[must_use]
    pub fn target_name(&self) -> &str {
        &self.target_name
    }

    /// Returns the target URL stored in this snapshot.
    #[must_use]
    pub fn target_url(&self) -> &str {
        &self.target_url
    }

    /// Borrows the recorded outcome.
    #[must_use]
    pub const fn outcome(&self) -> &CheckOutcome {
        &self.outcome
    }
}

/// An in-memory sequence of check results.
#[derive(Debug, Default)]
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
