//! Data types shared between the database layer and the Tauri commands.
//! Everything is serialised camelCase so the TypeScript side reads naturally.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A row of the `habits` table.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub skill: Option<String>,
    pub created_at: String,
    pub archived: bool,
}

/// User input for creating or editing a habit.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitInput {
    pub name: String,
    pub description: Option<String>,
    pub skill: Option<String>,
}

/// A habit plus everything derived from its completion history.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitSummary {
    #[serde(flatten)]
    pub habit: Habit,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub completed_today: bool,
    pub total_completions: u32,
    /// Completion flags for the last 7 days, oldest first (last = today).
    pub recent_days: Vec<bool>,
}

/// Summary plus the full completion history (for the details page/calendar).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitDetails {
    pub summary: HabitSummary,
    /// Completed dates, ascending, serialised as "YYYY-MM-DD".
    pub completions: Vec<NaiveDate>,
}

/// XP earned in one skill, derived from completions.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillProgress {
    pub skill: String,
    pub xp: u32,
    pub level: u32,
    /// XP earned inside the current level.
    pub xp_into_level: u32,
    /// XP needed to go from this level to the next.
    pub xp_for_next_level: u32,
}
