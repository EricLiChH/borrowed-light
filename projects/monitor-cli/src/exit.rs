//! Process exit codes shared by both binaries.
//!
//! `curl`, `grep` and friends all distinguish "I ran and the answer was no" from
//! "I could not run". Doing the same makes the monitor usable from a shell
//! script or a CI job without parsing its output.

/// The command completed and every check observed a response.
pub const SUCCESS: i32 = 0;

/// The command ran, but at least one check failed.
pub const CHECK_FAILED: i32 = 1;

/// The command could not run: bad arguments, an unreadable file, an invalid URL.
pub const USAGE: i32 = 2;
