use std::num::NonZeroUsize;
use std::time::Duration;

use crate::Backoff;

/// Runtime limits for website checks.
///
/// Attempts and concurrency are [`NonZeroUsize`], so "zero attempts" is not a
/// value this type can hold. Validation therefore happens in the type system
/// instead of in a fallible constructor — there is no [`PolicyError`] to
/// handle, and no code path that has to prove the policy is usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckPolicy {
    total_timeout: Duration,
    max_attempts: NonZeroUsize,
    backoff: Backoff,
    concurrency: NonZeroUsize,
}

impl CheckPolicy {
    /// Starts a policy from the defaults used by the CLI and the web service.
    #[must_use]
    pub const fn builder() -> CheckPolicyBuilder {
        CheckPolicyBuilder {
            total_timeout: Duration::from_secs(5),
            max_attempts: NonZeroUsize::MIN.saturating_add(1),
            backoff: Backoff::exponential(Duration::from_millis(100)),
            concurrency: NonZeroUsize::MIN.saturating_add(7),
        }
    }

    /// Returns the deadline that covers every attempt and every wait.
    #[must_use]
    pub const fn total_timeout(&self) -> Duration {
        self.total_timeout
    }

    /// Returns how many times one target may be requested.
    #[must_use]
    pub const fn max_attempts(&self) -> NonZeroUsize {
        self.max_attempts
    }

    /// Returns the waiting strategy between attempts.
    #[must_use]
    pub const fn backoff(&self) -> Backoff {
        self.backoff
    }

    /// Returns how many checks a batch may run at the same time.
    #[must_use]
    pub const fn concurrency(&self) -> NonZeroUsize {
        self.concurrency
    }
}

impl Default for CheckPolicy {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Builds a [`CheckPolicy`] without a positional-argument guessing game.
///
/// `CheckPolicy::new(5s, 2, 100ms, 8)` is easy to write and easy to misread —
/// two of its four arguments are [`Duration`]s that mean opposite things. Named
/// setters make the call site self-documenting and default-friendly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckPolicyBuilder {
    total_timeout: Duration,
    max_attempts: NonZeroUsize,
    backoff: Backoff,
    concurrency: NonZeroUsize,
}

impl CheckPolicyBuilder {
    /// Sets the deadline covering all attempts and waits.
    #[must_use]
    pub const fn total_timeout(mut self, total_timeout: Duration) -> Self {
        self.total_timeout = total_timeout;
        self
    }

    /// Sets how many times one target may be requested.
    #[must_use]
    pub const fn max_attempts(mut self, max_attempts: NonZeroUsize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the waiting strategy between attempts.
    #[must_use]
    pub const fn backoff(mut self, backoff: Backoff) -> Self {
        self.backoff = backoff;
        self
    }

    /// Sets how many checks a batch may run at the same time.
    #[must_use]
    pub const fn concurrency(mut self, concurrency: NonZeroUsize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Finishes the policy. This cannot fail: every field is already valid.
    #[must_use]
    pub const fn build(self) -> CheckPolicy {
        CheckPolicy {
            total_timeout: self.total_timeout,
            max_attempts: self.max_attempts,
            backoff: self.backoff,
            concurrency: self.concurrency,
        }
    }
}
