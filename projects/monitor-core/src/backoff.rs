use std::time::Duration;

/// How long to wait before the next attempt.
///
/// Backoff is described as data instead of being hard-coded in the retry loop,
/// which keeps it testable: [`Backoff::delay`] is a pure function of the retry
/// number and one caller-supplied random fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    initial: Duration,
    multiplier: u32,
    max: Duration,
    jitter: Jitter,
}

/// How much randomness to mix into each delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Jitter {
    /// Use the computed delay exactly. Required for deterministic tests.
    #[default]
    None,
    /// Pick uniformly from `0..=computed`. Recommended in production, because
    /// synchronized clients would otherwise retry in lockstep and re-create the
    /// spike that knocked the server over.
    Full,
}

impl Backoff {
    /// The default ceiling for exponential backoff: 30 seconds.
    pub const DEFAULT_MAX: Duration = Duration::from_secs(30);

    /// Never waits. Used by tests that assert on attempt counts only.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            initial: Duration::ZERO,
            multiplier: 1,
            max: Duration::ZERO,
            jitter: Jitter::None,
        }
    }

    /// Waits the same amount before every retry.
    #[must_use]
    pub const fn fixed(delay: Duration) -> Self {
        Self {
            initial: delay,
            multiplier: 1,
            max: delay,
            jitter: Jitter::None,
        }
    }

    /// Doubles the wait after every retry, up to [`Backoff::DEFAULT_MAX`], with
    /// full jitter.
    #[must_use]
    pub const fn exponential(initial: Duration) -> Self {
        Self {
            initial,
            multiplier: 2,
            max: Self::DEFAULT_MAX,
            jitter: Jitter::Full,
        }
    }

    /// Sets how fast the delay grows. A multiplier of `1` means constant delay.
    #[must_use]
    pub const fn with_multiplier(mut self, multiplier: u32) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Caps the computed delay.
    #[must_use]
    pub const fn with_max(mut self, max: Duration) -> Self {
        self.max = max;
        self
    }

    /// Chooses how much randomness to mix in.
    #[must_use]
    pub const fn with_jitter(mut self, jitter: Jitter) -> Self {
        self.jitter = jitter;
        self
    }

    /// Returns whether this strategy ever waits.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.initial.is_zero()
    }

    /// Returns the deterministic delay before retry number `retry_index`
    /// (1-based), ignoring jitter.
    #[must_use]
    pub fn base_delay(&self, retry_index: u32) -> Duration {
        if self.initial.is_zero() {
            return Duration::ZERO;
        }
        let exponent = retry_index.saturating_sub(1);
        let factor = self.multiplier.max(1).saturating_pow(exponent);
        self.initial.saturating_mul(factor).min(self.max)
    }

    /// Returns the delay to actually sleep, given a random fraction in `[0, 1)`.
    ///
    /// The fraction is an argument rather than a hidden source of randomness so
    /// that tests can pin it: `delay(2, 0.0)` is always the lower bound and
    /// `delay(2, 0.5)` is always exactly half of the computed delay.
    #[must_use]
    pub fn delay(&self, retry_index: u32, jitter_fraction: f64) -> Duration {
        let base = self.base_delay(retry_index);
        match self.jitter {
            Jitter::None => base,
            Jitter::Full => {
                let fraction = jitter_fraction.clamp(0.0, 1.0);
                base.mul_f64(fraction)
            }
        }
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::exponential(Duration::from_millis(100))
    }
}
