use std::collections::HashMap;

use monitor_domain::{CheckResult, MonitorTarget};
use tokio::sync::RwLock;

use crate::{MonitorRepository, StoreError, StoredTarget};

/// In-memory adapter used before the `SQLite` chapter and in fast tests.
///
/// The lock is [`tokio::sync::RwLock`] rather than the standard-library one: a
/// synchronous guard is not `Send`, so holding it across an `await` would make
/// every method future non-`Send` and axum could not run it.
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    state: RwLock<MemoryState>,
}

impl InMemoryRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
struct MemoryState {
    next_id: i64,
    targets: Vec<StoredTarget>,
    results: HashMap<i64, Vec<CheckResult>>,
}

impl MonitorRepository for InMemoryRepository {
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError> {
        let mut state = self.state.write().await;
        state.next_id += 1;
        let stored = StoredTarget::new(state.next_id, target);
        state.targets.push(stored.clone());
        Ok(stored)
    }

    async fn list_targets(&self) -> Result<Vec<StoredTarget>, StoreError> {
        Ok(self.state.read().await.targets.clone())
    }

    async fn get_target(&self, target_id: i64) -> Result<Option<StoredTarget>, StoreError> {
        Ok(self
            .state
            .read()
            .await
            .targets
            .iter()
            .find(|stored| stored.id() == target_id)
            .cloned())
    }

    async fn save_result(&self, target_id: i64, result: &CheckResult) -> Result<(), StoreError> {
        let mut state = self.state.write().await;
        let stored = state
            .targets
            .iter()
            .find(|stored| stored.id() == target_id)
            .ok_or(StoreError::NotFound { target_id })?;

        if stored.target() != result.target() {
            return Err(StoreError::TargetMismatch { target_id });
        }

        state
            .results
            .entry(target_id)
            .or_default()
            .push(result.clone());
        Ok(())
    }

    async fn latest_result(&self, target_id: i64) -> Result<Option<CheckResult>, StoreError> {
        Ok(self
            .state
            .read()
            .await
            .results
            .get(&target_id)
            .and_then(|results| results.last())
            .cloned())
    }
}
