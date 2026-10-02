use std::str::FromStr;
use std::time::Duration;

use monitor_domain::{CheckFailureKind, CheckOutcome, CheckResult, MonitorTarget};
use sqlx::Row;
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use crate::{MonitorRepository, StoreError, StoredTarget};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Connections a file-backed database keeps open.
const FILE_POOL_SIZE: u32 = 5;

/// How long a writer waits for the database lock before giving up.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

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
    /// Returns [`StoreError`] when the URL cannot be parsed, the connection
    /// cannot be opened, or a migration fails.
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(BUSY_TIMEOUT);

        // An in-memory database lives inside one connection. Giving it a pool of
        // two would hand each caller a different, empty database, which is the
        // trap the in-memory recipe describes. A file-backed database can use a
        // real pool, and write-ahead logging lets readers and the writer work at
        // the same time.
        let in_memory = options.get_filename().to_string_lossy() == ":memory:";
        let options = if in_memory {
            options
        } else {
            options.journal_mode(SqliteJournalMode::Wal)
        };
        let max_connections = if in_memory { 1 } else { FILE_POOL_SIZE };

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        MIGRATOR
            .run(&pool)
            .await
            .map_err(|error| StoreError::Migration {
                detail: error.to_string(),
            })?;

        Ok(Self { pool })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TargetRow {
    id: i64,
    name: String,
    url: String,
}

impl TryFrom<TargetRow> for StoredTarget {
    type Error = StoreError;

    fn try_from(row: TargetRow) -> Result<Self, Self::Error> {
        let target = MonitorTarget::new(row.name, row.url).map_err(StoreError::corrupt)?;
        Ok(Self::new(row.id, target))
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ResultRow {
    name: String,
    url: String,
    status: Option<i64>,
    failure_kind: Option<String>,
    reason: Option<String>,
}

fn result_from_row(row: ResultRow) -> Result<CheckResult, StoreError> {
    let target = MonitorTarget::new(row.name, row.url).map_err(StoreError::corrupt)?;

    let Some(status) = row.status else {
        let kind = row
            .failure_kind
            .ok_or_else(|| StoreError::corrupt("stored result has no failure category"))?
            .parse::<CheckFailureKind>()
            .map_err(StoreError::corrupt)?;
        let reason = row
            .reason
            .ok_or_else(|| StoreError::corrupt("stored result has no failure reason"))?;
        return Ok(CheckResult::unreachable_with(&target, kind, reason));
    };

    let status = u16::try_from(status).map_err(StoreError::corrupt)?;
    Ok(CheckResult::reachable(&target, status))
}

/// Splits an outcome into the four columns the schema stores it in.
fn outcome_columns(outcome: &CheckOutcome) -> (Option<i64>, Option<&'static str>, Option<&str>) {
    match outcome {
        CheckOutcome::Reachable { status } => (Some(i64::from(*status)), None, None),
        CheckOutcome::Unreachable { kind, reason } => (None, Some(kind.as_str()), Some(reason)),
    }
}

impl MonitorRepository for SqliteRepository {
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError> {
        // RETURNING hands the assigned id back in the same round trip, so there
        // is no window in which another writer could slip in between.
        let row = sqlx::query("INSERT INTO targets (name, url) VALUES (?, ?) RETURNING id")
            .bind(target.name())
            .bind(target.url_str())
            .fetch_one(&self.pool)
            .await?;
        let id = row.try_get::<i64, _>("id")?;
        Ok(StoredTarget::new(id, target))
    }

    async fn list_targets(&self) -> Result<Vec<StoredTarget>, StoreError> {
        let rows = sqlx::query_as::<_, TargetRow>("SELECT id, name, url FROM targets ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(StoredTarget::try_from).collect()
    }

    async fn get_target(&self, target_id: i64) -> Result<Option<StoredTarget>, StoreError> {
        let row = sqlx::query_as::<_, TargetRow>("SELECT id, name, url FROM targets WHERE id = ?")
            .bind(target_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(StoredTarget::try_from).transpose()
    }

    async fn save_result(&self, target_id: i64, result: &CheckResult) -> Result<(), StoreError> {
        // Verifying the target and inserting the result happen in one
        // transaction. Two separate statements would leave a window in which the
        // target could be deleted between the check and the write.
        let mut transaction = self.pool.begin().await?;

        let row = sqlx::query_as::<_, TargetRow>("SELECT id, name, url FROM targets WHERE id = ?")
            .bind(target_id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or(StoreError::NotFound { target_id })?;
        let stored = StoredTarget::try_from(row)?;
        if stored.target() != result.target() {
            return Err(StoreError::TargetMismatch { target_id });
        }

        let (status, kind, reason) = outcome_columns(result.outcome());
        sqlx::query(
            "INSERT INTO check_results (target_id, status, failure_kind, reason)
             VALUES (?, ?, ?, ?)",
        )
        .bind(target_id)
        .bind(status)
        .bind(kind)
        .bind(reason)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn latest_result(&self, target_id: i64) -> Result<Option<CheckResult>, StoreError> {
        let row = sqlx::query_as::<_, ResultRow>(
            "SELECT targets.name AS name, targets.url AS url,
                    check_results.status AS status,
                    check_results.failure_kind AS failure_kind,
                    check_results.reason AS reason
             FROM check_results
             JOIN targets ON targets.id = check_results.target_id
             WHERE check_results.target_id = ?
             ORDER BY check_results.id DESC
             LIMIT 1",
        )
        .bind(target_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(result_from_row).transpose()
    }
}
