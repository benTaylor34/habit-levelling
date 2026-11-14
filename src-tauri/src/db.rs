use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;

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
