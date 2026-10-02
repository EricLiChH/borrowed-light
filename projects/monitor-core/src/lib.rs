//! Asynchronous website checking for the course project.
//!
//! The checker answers one question — *what did this website do?* — and never
//! fails. Every failure mode, including running out of time, becomes a
//! [`CheckResult`] the caller can record and display.
//!
//! # The three moving parts
//!
//! - [`CheckPolicy`] is the *budget*: total deadline, attempts, backoff,
//!   concurrency.
//! - [`Backoff`] is the *waiting strategy* between attempts, as a pure
//!   function of the attempt number plus one random fraction.
//! - [`HealthChecker`] applies the policy to a reusable [`reqwest::Client`],
//!   so connection pooling and TLS setup are shared by every check.

mod backoff;
mod checker;
mod policy;

pub use backoff::{Backoff, Jitter};
pub use checker::{HealthChecker, classify_transport_error};
pub use policy::{CheckPolicy, CheckPolicyBuilder};
