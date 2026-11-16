mod activity;
mod commands;
mod db;
mod decay;
mod error;
mod habits;
mod history;
mod luck;
mod models;
mod recommend;
mod stats;
mod streaks;

use tauri::Manager;

/// The app's identifier before it was renamed (it decides the data folder).
const LEGACY_IDENTIFIER: &str = "com.ben.habit_levelling";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // e.g. %APPDATA%\com.habitlevelling.app\habits.db on Windows
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("habits.db");
            // Early builds used the identifier "com.ben.habit_levelling"; bring
            // that data across the first time this version starts.
            if let Some(parent) = data_dir.parent() {
                let legacy = parent.join(LEGACY_IDENTIFIER).join("habits.db");
                if let Err(e) = db::adopt_legacy_db(&db_path, &legacy) {
                    eprintln!("Couldn't copy data from {}: {e}", legacy.display());
                }
            }
            let pool = tauri::async_runtime::block_on(db::init_pool(&db_path))?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today,
            commands::list_habits,
            commands::get_habit_details,
            commands::create_habit,
            commands::update_habit,
            commands::delete_habit,
            commands::complete_habit_today,
            commands::undo_habit_today,
            commands::get_stats,
            commands::get_stat_history,
            commands::get_activity,
            commands::preview_xp,
            commands::recommend_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
