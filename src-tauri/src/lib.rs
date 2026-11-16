mod commands;
mod db;
mod error;
mod habits;
mod luck;
mod models;
mod stats;
mod streaks;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // e.g. %APPDATA%\com.ben.habit_levelling\habits.db on Windows
            let db_path = app.path().app_data_dir()?.join("habits.db");
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
            commands::preview_xp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
