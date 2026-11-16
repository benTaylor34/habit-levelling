//! Tauri commands: the only API the React frontend can call.
//! Each one is a thin wrapper that supplies "today" and delegates to `habits`.

use chrono::{Local, NaiveDate};
use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppResult;
use crate::models::{
    CompletionInput, Habit, HabitDetails, HabitInput, HabitSummary, LogResult, StatProgress,
    XpPreview,
};
use crate::stats;
use crate::habits;

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

/// XP a log would give the habit's primary and secondary stats, so the form
/// can preview it without duplicating the formula.
#[tauri::command]
pub fn preview_xp(minutes: u32, difficulty: u32) -> XpPreview {
    let primary = stats::completion_xp(minutes, difficulty);
    XpPreview {
        primary,
        secondary: stats::secondary_xp(primary),
    }
}

