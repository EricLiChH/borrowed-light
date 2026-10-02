//! Persistence adapters for the course project.
//!
//! Two adapters implement one seam: [`InMemoryRepository`] for the CLI chapters
//! and fast tests, [`SqliteRepository`] for the Web chapters. Everything above
//! this crate depends on [`MonitorRepository`], never on the database.
//!
//! # Why the trait methods are written out
//!
//! Three spellings are possible for an async trait method in Rust, and only one
//! of them fits this project:
//!
//! 1. `async fn` in the trait. Its future is *not* known to be `Send`, and axum
//!    handlers must be `Send`. It also cannot be used through `dyn`.
//! 2. `#[async_trait]`. It works, but boxes every single call, so each query
//!    pays an allocation and a heap indirection.
//! 3. `-> impl Future<Output = ...> + Send` — return-position impl trait in a
//!    trait. Implementors still write a plain `async fn` and the compiler keeps
//!    the future unboxed.
//!
//! This crate uses the third spelling.

mod error;
mod memory;
mod sqlite;

use std::future::Future;

use monitor_domain::{CheckResult, MonitorTarget};

pub use error::StoreError;
pub use memory::InMemoryRepository;
pub use sqlite::SqliteRepository;

/// A monitor target paired with its repository identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredTarget {
    id: i64,
    target: MonitorTarget,
}

impl StoredTarget {
    /// Pairs a target with the identifier the adapter assigned to it.
    #[must_use]
    pub const fn new(id: i64, target: MonitorTarget) -> Self {
        Self { id, target }
    }

    /// Returns the repository identifier.
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Borrows the domain target.
    #[must_use]
    pub const fn target(&self) -> &MonitorTarget {
        &self.target
    }
}

/// The persistence seam used by the Web module and project tests.
pub trait MonitorRepository: Send + Sync {
    /// Stores one target and assigns a stable identifier.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the adapter cannot write the target.
    fn add_target(
        &self,
        target: MonitorTarget,
    ) -> impl Future<Output = Result<StoredTarget, StoreError>> + Send;

    /// Lists targets in insertion order.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the adapter cannot read targets, or when a
    /// stored row can no longer be turned back into a valid target.
    fn list_targets(&self) -> impl Future<Output = Result<Vec<StoredTarget>, StoreError>> + Send;

    /// Finds one target by its repository identifier.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the adapter cannot read the target.
    fn get_target(
        &self,
        target_id: i64,
    ) -> impl Future<Output = Result<Option<StoredTarget>, StoreError>> + Send;

    /// Stores a check result for an existing target.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::NotFound`] when the identifier does not exist and
    /// [`StoreError::TargetMismatch`] when the result was produced for a
    /// different target. Saving a result against the wrong target would corrupt
    /// the history, so adapters reject it rather than writing it.
    fn save_result(
        &self,
        target_id: i64,
        result: &CheckResult,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// Returns the newest result for a target, if one exists.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the adapter cannot read the history.
    fn latest_result(
        &self,
        target_id: i64,
    ) -> impl Future<Output = Result<Option<CheckResult>, StoreError>> + Send;
}
