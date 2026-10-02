//! Plumbing shared by the two course CLIs.
//!
//! `monitor-sync` and `monitor` used to carry their own copy of the JSON
//! shapes, the client builder and the failure mapping. Three copies of a
//! mapping is three places to forget when a category is added, so both binaries
//! now depend on this crate instead.

pub mod client;
pub mod exit;
pub mod output;
