//! Persistence adapters for the course project.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use async_trait::async_trait;
use monitor_domain::{CheckFailureKind, CheckOutcome, CheckResult, MonitorTarget};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use tokio::sync::RwLock;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// A monitor target paired with its repository identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredTarget {
    id: i64,
    target: MonitorTarget,
}

impl StoredTarget {
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
#[async_trait]
pub trait MonitorRepository: Send + Sync {
    /// Stores one target and assigns a stable identifier.
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError>;

    /// Lists targets in insertion order.
    async fn list_targets(&self) -> Result<Vec<StoredTarget>, StoreError>;

    /// Finds one target by its repository identifier.
    async fn get_target(&self, target_id: i64) -> Result<Option<StoredTarget>, StoreError>;

    /// Stores a check result for an existing target.
    async fn save_result(&self, target_id: i64, result: CheckResult) -> Result<(), StoreError>;

    /// Returns the newest result for a target, if one exists.
    async fn latest_result(&self, target_id: i64) -> Result<Option<CheckResult>, StoreError>;
}

/// In-memory adapter used before the `SQLite` chapter and in fast tests.
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    state: RwLock<MemoryState>,
}

impl InMemoryRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: RwLock::new(MemoryState::default()),
        }
    }
}

#[async_trait]
impl MonitorRepository for InMemoryRepository {
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError> {
        let mut state = self.state.write().await;
        state.next_id += 1;
        let stored = StoredTarget {
            id: state.next_id,
            target,
        };
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
            .find(|target| target.id == target_id)
            .cloned())
    }

    async fn save_result(&self, target_id: i64, result: CheckResult) -> Result<(), StoreError> {
        let mut state = self.state.write().await;
        if !state.targets.iter().any(|target| target.id == target_id) {
            return Err(StoreError::new(format!("target {target_id} was not found")));
        }
        state.results.entry(target_id).or_default().push(result);
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

/// `SQLite` adapter used by the final CLI and Web applications.
#[derive(Debug, Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    /// Opens a `SQLite` database and creates the course schema when needed.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the connection or schema setup fails.
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(StoreError::from_display)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(StoreError::from_display)?;
        MIGRATOR
            .run(&pool)
            .await
            .map_err(StoreError::from_display)?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl MonitorRepository for SqliteRepository {
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError> {
        let result = sqlx::query("INSERT INTO targets (name, url) VALUES (?, ?)")
            .bind(target.name())
            .bind(target.url())
            .execute(&self.pool)
            .await
            .map_err(StoreError::from_display)?;
        Ok(StoredTarget {
            id: result.last_insert_rowid(),
            target,
        })
    }

    async fn list_targets(&self) -> Result<Vec<StoredTarget>, StoreError> {
        let rows = sqlx::query("SELECT id, name, url FROM targets ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from_display)?;
        rows.into_iter()
            .map(|row| {
                let id = row.try_get("id").map_err(StoreError::from_display)?;
                let name = row
                    .try_get::<String, _>("name")
                    .map_err(StoreError::from_display)?;
                let url = row
                    .try_get::<String, _>("url")
                    .map_err(StoreError::from_display)?;
                let target = MonitorTarget::new(name, url).map_err(StoreError::from_display)?;
                Ok(StoredTarget { id, target })
            })
            .collect()
    }

    async fn get_target(&self, target_id: i64) -> Result<Option<StoredTarget>, StoreError> {
        let row = sqlx::query("SELECT id, name, url FROM targets WHERE id = ?")
            .bind(target_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from_display)?;
        row.map(|row| {
            let id = row.try_get("id").map_err(StoreError::from_display)?;
            let name = row
                .try_get::<String, _>("name")
                .map_err(StoreError::from_display)?;
            let url = row
                .try_get::<String, _>("url")
                .map_err(StoreError::from_display)?;
            let target = MonitorTarget::new(name, url).map_err(StoreError::from_display)?;
            Ok(StoredTarget { id, target })
        })
        .transpose()
    }

    async fn save_result(&self, target_id: i64, result: CheckResult) -> Result<(), StoreError> {
        let (status, kind, reason) = outcome_columns(result.outcome());
        sqlx::query(
            "INSERT INTO check_results (target_id, status, failure_kind, reason)
             VALUES (?, ?, ?, ?)",
        )
        .bind(target_id)
        .bind(status)
        .bind(kind)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(StoreError::from_display)?;
        Ok(())
    }

    async fn latest_result(&self, target_id: i64) -> Result<Option<CheckResult>, StoreError> {
        let row = sqlx::query(
            "SELECT targets.name, targets.url, check_results.status,
                    check_results.failure_kind, check_results.reason
             FROM check_results
             JOIN targets ON targets.id = check_results.target_id
             WHERE check_results.target_id = ?
             ORDER BY check_results.id DESC
             LIMIT 1",
        )
        .bind(target_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from_display)?;

        row.as_ref().map(row_to_result).transpose()
    }
}

fn outcome_columns(outcome: &CheckOutcome) -> (Option<i64>, Option<&'static str>, Option<&str>) {
    match outcome {
        CheckOutcome::Reachable { status } => (Some(i64::from(*status)), None, None),
        CheckOutcome::Unreachable { kind, reason } => {
            (None, Some(failure_kind_name(*kind)), Some(reason))
        }
    }
}

fn row_to_result(row: &sqlx::sqlite::SqliteRow) -> Result<CheckResult, StoreError> {
    let name = row
        .try_get::<String, _>("name")
        .map_err(StoreError::from_display)?;
    let url = row
        .try_get::<String, _>("url")
        .map_err(StoreError::from_display)?;
    let target = MonitorTarget::new(name, url).map_err(StoreError::from_display)?;
    let status = row
        .try_get::<Option<i64>, _>("status")
        .map_err(StoreError::from_display)?;
    if let Some(status) = status {
        let status = u16::try_from(status).map_err(StoreError::from_display)?;
        return Ok(CheckResult::reachable(&target, status));
    }

    let kind = row
        .try_get::<Option<String>, _>("failure_kind")
        .map_err(StoreError::from_display)?
        .as_deref()
        .and_then(parse_failure_kind)
        .ok_or_else(|| StoreError::new("stored result has no failure category"))?;
    let reason = row
        .try_get::<Option<String>, _>("reason")
        .map_err(StoreError::from_display)?
        .ok_or_else(|| StoreError::new("stored result has no failure reason"))?;
    Ok(CheckResult::unreachable_with(&target, kind, reason))
}

const fn failure_kind_name(kind: CheckFailureKind) -> &'static str {
    match kind {
        CheckFailureKind::Timeout => "timeout",
        CheckFailureKind::Connect => "connect",
        CheckFailureKind::Request => "request",
    }
}

fn parse_failure_kind(value: &str) -> Option<CheckFailureKind> {
    match value {
        "timeout" => Some(CheckFailureKind::Timeout),
        "connect" => Some(CheckFailureKind::Connect),
        "request" => Some(CheckFailureKind::Request),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct MemoryState {
    next_id: i64,
    targets: Vec<StoredTarget>,
    results: HashMap<i64, Vec<CheckResult>>,
}

/// A persistence operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    message: String,
}

impl StoreError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn from_display(error: impl fmt::Display) -> Self {
        Self::new(error.to_string())
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StoreError {}
