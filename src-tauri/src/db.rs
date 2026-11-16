use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};

/// Migrations are embedded into the binary at compile time, so the installed
/// app doesn't depend on the source tree being present.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Open (creating if needed) the SQLite database at `db_path` and run migrations.
pub async fn init_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true); // needed for ON DELETE CASCADE

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// `path` with `suffix` appended to the file name ("habits.db" -> "habits.db-wal").
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// If there's no database at `db_path` yet but one exists at `legacy_db`
/// (where an older version of the app kept it), copy it across. Copies rather
/// than moves, so the old folder stays as a backup. Returns whether it copied.
pub fn adopt_legacy_db(db_path: &Path, legacy_db: &Path) -> std::io::Result<bool> {
    if db_path.exists() || !legacy_db.exists() {
        return Ok(false);
    }
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // The -wal file holds recent writes not yet merged into the main file.
    // Copy it first and the main file last, so a half-finished copy is
    // retried on the next launch rather than leaving a database missing data.
    let wal = with_suffix(legacy_db, "-wal");
    if wal.exists() {
        std::fs::copy(&wal, with_suffix(db_path, "-wal"))?;
    }
    std::fs::copy(legacy_db, db_path)?;
    Ok(true)
}

/// Fresh in-memory database for tests. A single connection, because every
/// SQLite in-memory connection would otherwise be its own separate database.
#[cfg(test)]
pub async fn test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .in_memory(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn legacy_database_is_copied_once() {
        let root = std::env::temp_dir().join(format!(
            "habit_levelling_adopt_{}_{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let legacy = root.join("com.ben.habit_levelling").join("habits.db");
        let current = root.join("com.habitlevelling.app").join("habits.db");

        // Old app: one habit saved.
        let pool = init_pool(&legacy).await.unwrap();
        sqlx::query("INSERT INTO habits (name) VALUES ('Read')")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        assert!(adopt_legacy_db(&current, &legacy).unwrap());
        assert!(legacy.exists(), "old copy kept as a backup");
        let pool = init_pool(&current).await.unwrap();
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM habits")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(n, 1);
        pool.close().await;

        // Never overwrites a database that already exists.
        assert!(!adopt_legacy_db(&current, &legacy).unwrap());
        // Nothing to copy for a brand-new user.
        let fresh = root.join("other").join("habits.db");
        assert!(!adopt_legacy_db(&fresh, &root.join("missing.db")).unwrap());

        let _ = std::fs::remove_dir_all(&root);
    }
}
