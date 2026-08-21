use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::path::PathBuf;
use sqlx::migrate::Migrator;

pub async fn init_pool(db_path: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    // ensure the file exists / parent dir exists before connecting
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}", db_path.display()))
        .await?;

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_path = std::path::Path::new(&crate_dir).join("./migrations");
    let migrator = Migrator::new(migrations_path).await?;
    migrator.run(&pool).await?;

    Ok(pool)
}

#[tokio::main]
async fn main(){

}