//! Tauri commands: the only API the React frontend can call.
//! Each one is a thin wrapper that supplies "today" and delegates to `habits`.

use chrono::{Local, NaiveDate};
use sqlx::SqlitePool;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use crate::error::AppResult;
use habit_core::models::{
    CompletionInput, Habit, HabitDetails, HabitInput, HabitSummary, LogResult, StatHistory,
    StatProgress, XpPreview,
};
use habit_core::activity::Activity;
use crate::backup::{self, BackupSummary};
use crate::error::AppError;
use crate::habits;
use habit_core::recommend::{self, StatRecommendation};

/// "Today" is the user's *local* calendar date, not a UTC timestamp.
fn today() -> NaiveDate {
    Local::now().date_naive()
}

#[tauri::command]
pub fn get_today() -> NaiveDate {
    today()
}

#[tauri::command]
pub async fn list_habits(pool: State<'_, SqlitePool>) -> AppResult<Vec<HabitSummary>> {
    habits::summaries(&pool, today()).await
}

#[tauri::command]
pub async fn get_habit_details(pool: State<'_, SqlitePool>, id: i64) -> AppResult<HabitDetails> {
    habits::details(&pool, id, today()).await
}

#[tauri::command]
pub async fn create_habit(pool: State<'_, SqlitePool>, input: HabitInput) -> AppResult<Habit> {
    habits::create(&pool, input).await
}

#[tauri::command]
pub async fn update_habit(
    pool: State<'_, SqlitePool>,
    id: i64,
    input: HabitInput,
) -> AppResult<Habit> {
    habits::update(&pool, id, input).await
}

#[tauri::command]
pub async fn delete_habit(pool: State<'_, SqlitePool>, id: i64) -> AppResult<()> {
    habits::delete(&pool, id).await
}

/// Log today for a habit. May also win Luck XP if the streak hit a milestone.
#[tauri::command]
pub async fn complete_habit_today(
    pool: State<'_, SqlitePool>,
    id: i64,
    log: CompletionInput,
) -> AppResult<LogResult> {
    let today = today();
    let luck = habits::complete(&pool, id, today, log).await?;
    Ok(LogResult {
        summary: habits::summary(&pool, id, today).await?,
        luck,
    })
}

#[tauri::command]
pub async fn undo_habit_today(pool: State<'_, SqlitePool>, id: i64) -> AppResult<HabitSummary> {
    let today = today();
    habits::uncomplete(&pool, id, today).await?;
    habits::summary(&pool, id, today).await
}

#[tauri::command]
pub async fn get_stats(pool: State<'_, SqlitePool>) -> AppResult<Vec<StatProgress>> {
    habits::stat_progress(&pool, today()).await
}

/// XP a log for today would give the habit's primary and secondary stats
/// (including the streak bonus), so the form can preview it without
/// duplicating the formula.
#[tauri::command]
pub async fn preview_xp(
    pool: State<'_, SqlitePool>,
    id: i64,
    minutes: u32,
    difficulty: u32,
) -> AppResult<XpPreview> {
    habits::preview_xp(&pool, id, today(), minutes, difficulty).await
}

/// Suggested primary/secondary stats for a habit's name and description.
#[tauri::command]
pub fn recommend_stats(name: String, description: Option<String>) -> StatRecommendation {
    recommend::recommend(&name, description.as_deref())
}

/// Stat XP and levels per day for the last `days` days, or all time if `None`.
#[tauri::command]
pub async fn get_stat_history(
    pool: State<'_, SqlitePool>,
    days: Option<u32>,
) -> AppResult<StatHistory> {
    habits::stat_history(&pool, today(), days).await
}

/// A year of daily activity (habits logged per day) for the heatmap.
#[tauri::command]
pub async fn get_activity(pool: State<'_, SqlitePool>) -> AppResult<Activity> {
    habits::activity(&pool, today()).await
}

/// Where automatic and safety backups live: `<app data>/backups`.
pub fn backups_dir(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app_data_dir(app)?.join("backups"))
}

fn app_data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(format!("Couldn't find the app data folder: {e}")))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedBackup {
    /// Suggested file name, e.g. "habit-levelling-backup-2026-11-16.json".
    pub file_name: String,
    /// The backup itself, for the page to save wherever the user picks.
    pub json: String,
}

/// Build a backup of everything. The page saves it through the system "Save"
/// dialog, which works the same on desktop and Android.
#[tauri::command]
pub async fn export_backup(pool: State<'_, SqlitePool>) -> AppResult<ExportedBackup> {
    let data = backup::export(&pool).await?;
    Ok(ExportedBackup {
        file_name: format!("habit-levelling-backup-{}.json", today().format("%Y-%m-%d")),
        json: data.to_json(),
    })
}

/// Check a backup file's contents and say what's in it, without restoring.
#[tauri::command]
pub fn inspect_backup(json: String) -> AppResult<BackupSummary> {
    Ok(backup::parse(&json)?.summary())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub summary: BackupSummary,
    /// Copy of the data as it was just before restoring, to undo with.
    pub safety_backup: String,
}

/// Replace all data with a backup's contents, saving the current data first.
#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    json: String,
) -> AppResult<RestoreResult> {
    let incoming = backup::parse(&json)?;
    let current = backup::export(&pool).await?;
    let stem = format!("before-restore-{}", today().format("%Y-%m-%d"));
    let safety = backup::write(&current, &backups_dir(&app)?, &stem)?;
    backup::restore(&pool, &incoming).await?;
    Ok(RestoreResult {
        summary: incoming.summary(),
        safety_backup: safety.display().to_string(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// The live database file.
    pub data_file: String,
    pub backups_dir: String,
    /// Number of automatic daily backups currently kept.
    pub auto_backups: usize,
    /// Full path of the newest automatic backup, e.g. ".../backups/auto-2026-11-16.json".
    pub latest_auto: Option<String>,
    pub auto_backups_kept: usize,
}

#[tauri::command]
pub fn get_backup_info(app: AppHandle) -> AppResult<BackupInfo> {
    let dir = backups_dir(&app)?;
    let autos = backup::auto_backups(&dir);
    Ok(BackupInfo {
        data_file: app_data_dir(&app)?.join("habits.db").display().to_string(),
        backups_dir: dir.display().to_string(),
        auto_backups: autos.len(),
        latest_auto: autos.first().map(|p| p.display().to_string()),
        auto_backups_kept: backup::AUTO_BACKUPS_KEPT,
    })
}
