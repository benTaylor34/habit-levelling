fn main() {
    // Re-embed migrations when they change (sqlx::migrate! reads them at compile time).
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}
