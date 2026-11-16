//! Data types shared between the database layer and the Tauri commands.
//! Everything is serialised camelCase so the TypeScript side reads naturally.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::stats::Stat;

/// A row of the `habits` table.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    /// Stat that gets the full XP of each completion.
    pub primary_stat: Option<Stat>,
    /// Stat that gets half the XP of each completion.
    pub secondary_stat: Option<Stat>,
    pub created_at: String,
    pub archived: bool,
}

/// User input for creating or editing a habit.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitInput {
    pub name: String,
    pub description: Option<String>,
    pub primary_stat: Option<Stat>,
    pub secondary_stat: Option<Stat>,
}

/// What the user enters when logging a completion.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionInput {
    /// Time spent that day, in minutes (0 if not tracked).
    pub minutes: u32,
    /// How hard it felt, 1 (easy) to 5 (brutal).
    pub difficulty: u32,
}

/// One logged day for a habit, with the XP it earned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    /// Local calendar date, serialised as "YYYY-MM-DD".
    pub date: NaiveDate,
    pub minutes: u32,
    pub difficulty: u32,
    /// The habit's streak on this day (1 = first day of a streak).
    pub streak: u32,
    /// XP the primary stat earned, including the streak bonus.
    pub xp: u32,
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
    pub total_minutes: u32,
    pub total_xp: u32,
    /// Today's log, if the habit has been completed today.
    pub today_log: Option<Completion>,
    /// Completion flags for the last 7 days, oldest first (last = today).
    pub recent_days: Vec<bool>,
}

/// Summary plus the full completion history (for the details page/calendar).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitDetails {
    pub summary: HabitSummary,
    /// Every logged day, oldest first.
    pub completions: Vec<Completion>,
}

/// Level, XP bar and streaks for one stat, derived from the completions of
/// every habit that trains it (as primary or secondary).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatProgress {
    pub stat: Stat,
    /// Total XP ever earned in this stat.
    pub xp: u32,
    pub level: u32,
    /// XP earned since the last level up (the XP bar resets to 0 each level).
    pub xp_into_level: u32,
    /// XP needed to go from this level to the next (0 at max level).
    pub xp_for_next_level: u32,
    /// Consecutive days on which any habit training this stat was done.
    pub current_streak: u32,
    pub longest_streak: u32,
}

/// Luck XP won when a streak milestone came up lucky.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckWin {
    pub xp: u32,
    /// The streak length that triggered the roll.
    pub streak: u32,
}

/// Result of logging a habit for today.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResult {
    pub summary: HabitSummary,
    /// Set only when this log hit a streak milestone *and* the roll won.
    pub luck: Option<LuckWin>,
}

/// XP a log would give each of a habit's stats (for the form's preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XpPreview {
    pub primary: u32,
    pub secondary: u32,
    /// The streak this log would be part of.
    pub streak: u32,
    /// Streak multiplier as a percentage (100 = no bonus).
    pub streak_percent: u32,
}

/// One stat's day-by-day progress over a date range.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatSeries {
    pub stat: Stat,
    /// Cumulative XP at the end of each day, lined up with `StatHistory::dates`.
    pub xp: Vec<u32>,
    /// Level as a decimal at the end of each day (3.5 = halfway to level 4).
    pub level: Vec<f64>,
    pub start_level: u32,
    pub end_level: u32,
    /// XP earned within the range.
    pub xp_gained: u32,
}

/// Stat progress over time for the Analysis page.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatHistory {
    /// Every day in the range, oldest first.
    pub dates: Vec<NaiveDate>,
    /// One series per trainable stat (Luck is secret and not charted).
    pub series: Vec<StatSeries>,
}
