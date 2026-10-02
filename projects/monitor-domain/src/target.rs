use std::fmt;

use url::Url;

/// A website selected for health checks.
///
/// Both fields are validated once, in [`MonitorTarget::new`], so the rest of
/// the program never has to re-check them:
///
/// - the name is trimmed and known to be non-empty;
/// - the URL is a parsed [`Url`] whose scheme is HTTP or HTTPS and whose host
///   is non-empty.
///
/// Parsing with [`Url`] also *normalizes* the input: the scheme is lowercased
/// and an empty path becomes `/`. So `https://www.rust-lang.org` is stored as
/// `https://www.rust-lang.org/`. That is deliberate — the standard URL form is
/// what gets persisted, compared and printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorTarget {
    name: String,
    url: Url,
}

impl MonitorTarget {
    /// Creates a monitor target from a learner-facing name and a URL.
    ///
    /// # Errors
    ///
    /// Returns [`TargetError::EmptyName`] when the name is blank,
    /// [`TargetError::UnsupportedScheme`] when the URL parses but does not use
    /// HTTP or HTTPS, and [`TargetError::InvalidUrl`] when the URL cannot be
    /// parsed as an absolute HTTP(S) URL with a host.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Result<Self, TargetError> {
        let name = name.into();
        let name = name.trim();
        if name.is_empty() {
            return Err(TargetError::EmptyName);
        }

        let raw_url = url.into();
        let url = Url::parse(raw_url.trim()).map_err(|_error| TargetError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(TargetError::UnsupportedScheme);
        }
        if url.host_str().is_none_or(str::is_empty) {
            return Err(TargetError::InvalidUrl);
        }

        Ok(Self {
            name: name.to_owned(),
            url,
        })
    }

    /// Returns the trimmed, learner-facing name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parsed URL.
    #[must_use]
    pub const fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the URL in its serialized, normalized form.
    #[must_use]
    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }
}

impl fmt::Display for MonitorTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.name, self.url)
    }
}

/// Why a monitor target could not be created.
///
/// The three variants are the three decisions a caller can act on: fix the
/// name, fix the URL, or stop pointing the monitor at a non-HTTP service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetError {
    /// The learner-facing name contained only whitespace.
    EmptyName,
    /// The URL was missing a scheme, a host, or was otherwise unparsable.
    InvalidUrl,
    /// The URL did not use HTTP or HTTPS.
    UnsupportedScheme,
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyName => "target name must not be blank",
            Self::InvalidUrl => "target URL must be an absolute http(s) URL with a host",
            Self::UnsupportedScheme => "target URL must use http or https",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TargetError {}
