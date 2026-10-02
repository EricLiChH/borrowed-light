use std::path::PathBuf;

use monitor_domain::MonitorTarget;
use monitor_store::{MonitorRepository, SqliteRepository};

/// A file-backed database in the temporary directory, removed on drop.
///
/// The in-memory tests exercise one connection; this one exercises the pool
/// that the Web service actually uses, including the side files that
/// write-ahead logging creates.
struct TempDatabase {
    path: PathBuf,
}

impl TempDatabase {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("borrowed-light-{name}-{}.db", std::process::id()));
        let database = Self { path };
        database.remove_files();
        database
    }

    fn url(&self) -> String {
        format!("sqlite://{}", self.path.display())
    }

    fn remove_files(&self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        self.remove_files();
    }
}

#[tokio::test]
async fn a_file_database_keeps_targets_across_connections() {
    let database = TempDatabase::new("store");

    let stored = {
        let repository = SqliteRepository::connect(&database.url())
            .await
            .expect("file database should open");
        repository
            .add_target(MonitorTarget::new("Rust", "https://www.rust-lang.org").expect("valid"))
            .await
            .expect("target should be stored")
    };

    let reopened = SqliteRepository::connect(&database.url())
        .await
        .expect("file database should reopen");
    let targets = reopened
        .list_targets()
        .await
        .expect("targets should be listed");

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].id(), stored.id());
    assert_eq!(targets[0].target().name(), "Rust");
}
