//! Domain types for the website health monitor project.
//!
//! This crate owns the vocabulary of the course project: what a monitored
//! website is, what a check observed, and why a check failed. It performs no
//! I/O and depends on no runtime, so every other crate can speak these types.
//!
//! # Layering
//!
//! - [`MonitorTarget`] parses and validates user input once, at the boundary.
//! - [`CheckOutcome`] and [`CheckFailureKind`] classify what a check observed.
//! - [`CheckResult`] is an owned snapshot: a target plus its outcome.
//! - [`CheckHistory`] keeps recorded snapshots in order.

mod check;
mod target;

pub use check::{CheckFailureKind, CheckHistory, CheckOutcome, CheckResult, ParseFailureKindError};
pub use target::{MonitorTarget, TargetError};
