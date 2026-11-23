//! Habit Levelling's game rules, independent of any platform.
//!
//! Everything here is pure: functions take history in and give results out,
//! with no database, files, clock or UI. That keeps the rules testable and
//! lets every front end (desktop, Android, a future web or server version)
//! share exactly the same behaviour.
//!
//! - [`stats`]: the six stats (plus Luck), XP per completion and levels.
//! - [`habit_levels`]: each habit's own level, from time spent only.
//! - [`streaks`]: current, longest and per-day streaks.
//! - [`decay`]: idle stats losing XP, then levels.
//! - [`luck`]: the secret Luck stat's streak-milestone rolls.
//! - [`history`]: day-by-day stat progress for charts.
//! - [`activity`]: a year of daily activity for the heatmap.
//! - [`recommend`]: suggesting stats from a habit's name and description.
//! - [`models`]: the shared data types, serialised for the UI.

pub mod activity;
pub mod decay;
pub mod habit_levels;
pub mod history;
pub mod luck;
pub mod models;
pub mod quests;
pub mod recommend;
pub mod stats;
pub mod streaks;
