//! Backup, export and restore.
//!
//! A backup is one human-readable JSON file holding everything the user has
//! entered: habits, daily logs and Luck rolls. Everything else (XP, levels,
//! streaks, decay) is derived from those, so restoring them restores it all.
//!
//! Restoring replaces all current data inside one transaction, so a bad
//! file can never leave the database half-restored. A safety backup of the
//! current data is written first, so a restore can always be undone.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use habit_core::stats::{self, Stat};

/// Identifies our files, so restoring some other JSON file fails clearly.
pub const APP_TAG: &str = "habit-levelling";
/// Bump when the file layout changes; older files must stay restorable.
pub const FORMAT_VERSION: u32 = 1;
/// How many automatic daily backups to keep.
pub const AUTO_BACKUPS_KEPT: usize = 14;
const AUTO_PREFIX: &str = "auto-";
const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupHabit {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub primary_stat: Option<Stat>,
    pub secondary_stat: Option<Stat>,
    pub created_at: String,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupCompletion {
    pub habit_id: i64,
    pub date: NaiveDate,
    pub minutes: u32,
    pub difficulty: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupLuckRoll {
    pub habit_id: i64,
    pub date: NaiveDate,
    pub streak: u32,
    pub xp: u32,
}

/// The whole backup file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub app: String,
    pub format_version: u32,
    /// Local time the backup was made, e.g. "2026-11-16 21:30:00".
    pub exported_at: String,
    pub habits: Vec<BackupHabit>,
    pub completions: Vec<BackupCompletion>,
    pub luck_rolls: Vec<BackupLuckRoll>,
}

/// What a backup contains, shown before restoring it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub exported_at: String,
    pub habits: usize,
    pub completions: usize,
    pub first_date: Option<NaiveDate>,
    pub last_date: Option<NaiveDate>,
}

impl Backup {
    pub fn summary(&self) -> BackupSummary {
        BackupSummary {
            exported_at: self.exported_at.clone(),
            habits: self.habits.len(),
            completions: self.completions.len(),
            first_date: self.completions.iter().map(|c| c.date).min(),
            last_date: self.completions.iter().map(|c| c.date).max(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("backup always serialises")
    }
}

fn parse_date(s: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATE_FORMAT)
        .map_err(|e| AppError::Invalid(format!("Bad date '{s}' in database: {e}")))
}

/// Read everything the user has entered.
pub async fn export(pool: &SqlitePool) -> AppResult<Backup> {
    let habits: Vec<(i64, String, Option<String>, Option<Stat>, Option<Stat>, String, bool)> =
        sqlx::query_as(
            "SELECT id, name, description, primary_stat, secondary_stat, created_at, archived
             FROM habits ORDER BY id",
        )
        .fetch_all(pool)
        .await?;
    let completions: Vec<(i64, String, i64, i64)> = sqlx::query_as(
        "SELECT habit_id, date, minutes, difficulty FROM habit_completions ORDER BY habit_id, date",
    )
    .fetch_all(pool)
    .await?;
    let luck: Vec<(i64, String, i64, i64)> =
        sqlx::query_as("SELECT habit_id, date, streak, xp FROM luck_rolls ORDER BY habit_id, date")
            .fetch_all(pool)
            .await?;

    Ok(Backup {
        app: APP_TAG.into(),
        format_version: FORMAT_VERSION,
        exported_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        habits: habits
            .into_iter()
            .map(|(id, name, description, primary_stat, secondary_stat, created_at, archived)| {
                BackupHabit { id, name, description, primary_stat, secondary_stat, created_at, archived }
            })
            .collect(),
        completions: completions
            .into_iter()
            .map(|(habit_id, date, minutes, difficulty)| {
                Ok(BackupCompletion {
                    habit_id,
                    date: parse_date(&date)?,
                    minutes: minutes.max(0) as u32,
                    difficulty: difficulty.max(0) as u32,
                })
            })
            .collect::<AppResult<_>>()?,
        luck_rolls: luck
            .into_iter()
            .map(|(habit_id, date, streak, xp)| {
                Ok(BackupLuckRoll {
                    habit_id,
                    date: parse_date(&date)?,
                    streak: streak.max(0) as u32,
                    xp: xp.max(0) as u32,
                })
            })
            .collect::<AppResult<_>>()?,
    })
}

/// Parse and fully check a backup file's contents before anything is written.
pub fn parse(json: &str) -> AppResult<Backup> {
    let not_ours = || AppError::Invalid("This isn't a Habit Levelling backup file.".into());
    let value: serde_json::Value = serde_json::from_str(json).map_err(|_| not_ours())?;
    if value.get("app").and_then(|v| v.as_str()) != Some(APP_TAG) {
        return Err(not_ours());
    }
    let version = value.get("formatVersion").and_then(|v| v.as_u64()).unwrap_or(0);
    if version > u64::from(FORMAT_VERSION) {
        return Err(AppError::Invalid(
            "This backup was made by a newer version of Habit Levelling. Update the app to restore it."
                .into(),
        ));
    }
    let backup: Backup = serde_json::from_value(value)
        .map_err(|e| AppError::Invalid(format!("This backup file is damaged: {e}")))?;
    validate(&backup)?;
    Ok(backup)
}

fn validate(b: &Backup) -> AppResult<()> {
    let bad = |msg: String| Err(AppError::Invalid(format!("This backup file is damaged: {msg}")));
    let mut ids = HashSet::new();
    for h in &b.habits {
        if !ids.insert(h.id) {
            return bad(format!("habit {} appears twice", h.id));
        }
        if h.name.trim().is_empty() {
            return bad(format!("habit {} has no name", h.id));
        }
        for stat in [h.primary_stat, h.secondary_stat].into_iter().flatten() {
            if !Stat::TRAINABLE.contains(&stat) {
                return bad(format!("habit {} trains Luck", h.id));
            }
        }
        if h.secondary_stat.is_some() && (h.primary_stat.is_none() || h.primary_stat == h.secondary_stat) {
            return bad(format!("habit {} has an invalid secondary stat", h.id));
        }
    }
    let mut days = HashSet::new();
    for c in &b.completions {
        if !ids.contains(&c.habit_id) {
            return bad(format!("a log refers to missing habit {}", c.habit_id));
        }
        if !days.insert((c.habit_id, c.date)) {
            return bad(format!("habit {} is logged twice on {}", c.habit_id, c.date));
        }
        if c.minutes > stats::MAX_MINUTES
            || !(stats::MIN_DIFFICULTY..=stats::MAX_DIFFICULTY).contains(&c.difficulty)
        {
            return bad(format!("the log for habit {} on {} is out of range", c.habit_id, c.date));
        }
    }
    let mut rolls = HashSet::new();
    for r in &b.luck_rolls {
        if !ids.contains(&r.habit_id) || !rolls.insert((r.habit_id, r.date)) {
            return bad(format!("bad Luck roll for habit {} on {}", r.habit_id, r.date));
        }
    }
    Ok(())
}

/// Replace all data with the backup's contents, all or nothing.
pub async fn restore(pool: &SqlitePool, backup: &Backup) -> AppResult<()> {
    validate(backup)?;
    let mut tx = pool.begin().await?;
    // Children first, so foreign keys are never violated mid-way.
    sqlx::query("DELETE FROM luck_rolls").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM habit_completions").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM habits").execute(&mut *tx).await?;
    for h in &backup.habits {
        sqlx::query(
            "INSERT INTO habits (id, name, description, primary_stat, secondary_stat, created_at, archived)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(h.id)
        .bind(&h.name)
        .bind(&h.description)
        .bind(h.primary_stat)
        .bind(h.secondary_stat)
        .bind(&h.created_at)
        .bind(h.archived)
        .execute(&mut *tx)
        .await?;
    }
    for c in &backup.completions {
        sqlx::query(
            "INSERT INTO habit_completions (habit_id, date, minutes, difficulty) VALUES (?, ?, ?, ?)",
        )
        .bind(c.habit_id)
        .bind(c.date.format(DATE_FORMAT).to_string())
        .bind(c.minutes)
        .bind(c.difficulty)
        .execute(&mut *tx)
        .await?;
    }
    for r in &backup.luck_rolls {
        sqlx::query("INSERT INTO luck_rolls (habit_id, date, streak, xp) VALUES (?, ?, ?, ?)")
            .bind(r.habit_id)
            .bind(r.date.format(DATE_FORMAT).to_string())
            .bind(r.streak)
            .bind(r.xp)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// `dir/stem.json`, or `dir/stem-2.json`, `-3`… if that name is taken.
pub fn unique_path(dir: &Path, stem: &str) -> PathBuf {
    let first = dir.join(format!("{stem}.json"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{stem}-{n}.json")))
        .find(|p| !p.exists())
        .expect("some name is free")
}

/// Write a backup into `dir` under a fresh name; returns the file's path.
pub fn write(backup: &Backup, dir: &Path, stem: &str) -> AppResult<PathBuf> {
    std::fs::create_dir_all(dir).map_err(|e| AppError::Invalid(format!("Couldn't create {}: {e}", dir.display())))?;
    let path = unique_path(dir, stem);
    std::fs::write(&path, backup.to_json())
        .map_err(|e| AppError::Invalid(format!("Couldn't save the backup: {e}")))?;
    Ok(path)
}

/// Save today's automatic backup into `dir` (once a day) and delete all but
/// the newest `AUTO_BACKUPS_KEPT`. Returns the new file, if one was written.
pub async fn auto_backup(pool: &SqlitePool, dir: &Path, today: NaiveDate) -> AppResult<Option<PathBuf>> {
    let path = dir.join(format!("{AUTO_PREFIX}{}.json", today.format(DATE_FORMAT)));
    let written = if path.exists() {
        None
    } else {
        let backup = export(pool).await?;
        if backup.habits.is_empty() {
            return Ok(None); // nothing worth backing up yet
        }
        std::fs::create_dir_all(dir).map_err(|e| AppError::Invalid(e.to_string()))?;
        std::fs::write(&path, backup.to_json()).map_err(|e| AppError::Invalid(e.to_string()))?;
        Some(path)
    };
    prune_auto_backups(dir);
    Ok(written)
}

/// Automatic backups in `dir`, newest first (names sort by date).
pub fn auto_backups(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(AUTO_PREFIX) && n.ends_with(".json"))
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files.reverse();
    files
}

fn prune_auto_backups(dir: &Path) {
    for old in auto_backups(dir).into_iter().skip(AUTO_BACKUPS_KEPT) {
        let _ = std::fs::remove_file(old);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::habits::{self, complete_with_roll, create, summary};
    use habit_core::models::{CompletionInput, HabitInput};
    use habit_core::stats::Stat::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).unwrap()
    }

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "habit_levelling_{tag}_{}_{}",
            std::process::id(),
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    /// A database with two habits, a few logs and a Luck roll.
    async fn sample() -> SqlitePool {
        let pool = db::test_pool().await;
        let read = create(
            &pool,
            HabitInput {
                name: "Read".into(),
                description: Some("20 pages".into()),
                primary_stat: Some(Intelligence),
                secondary_stat: Some(Wisdom),
            },
        )
        .await
        .unwrap();
        create(
            &pool,
            HabitInput { name: "Run".into(), description: None, primary_stat: Some(Dexterity), secondary_stat: None },
        )
        .await
        .unwrap();
        // A 7-day streak with a winning roll on day 7 (a Luck milestone).
        for day in 1..=7 {
            let date = d(&format!("2026-11-0{day}"));
            let log = CompletionInput { minutes: 10 * day, difficulty: 2 };
            complete_with_roll(&pool, read.id, date, log, 0).await.unwrap();
        }
        pool
    }

    #[tokio::test]
    async fn export_then_restore_round_trips_everything() {
        let source = sample().await;
        let backup = export(&source).await.unwrap();
        assert_eq!((backup.habits.len(), backup.completions.len(), backup.luck_rolls.len()), (2, 7, 1));

        // Through JSON, into a different database that already has other data.
        let parsed = parse(&backup.to_json()).unwrap();
        let target = db::test_pool().await;
        create(&target, HabitInput { name: "Old".into(), description: None, primary_stat: None, secondary_stat: None })
            .await
            .unwrap();
        restore(&target, &parsed).await.unwrap();

        let mut again = export(&target).await.unwrap();
        again.exported_at = backup.exported_at.clone();
        assert_eq!(again, backup, "restored data matches exactly; 'Old' is gone");

        // Derived numbers come back too.
        let today = d("2026-11-07");
        let s = summary(&target, 1, today).await.unwrap();
        assert_eq!((s.current_streak, s.total_completions), (7, 7));
        let target_stats = habits::stat_progress(&target, today).await.unwrap();
        let source_stats = habits::stat_progress(&source, today).await.unwrap();
        assert_eq!(target_stats, source_stats, "XP, levels and Luck are identical");
    }

    #[tokio::test]
    async fn rejects_files_that_are_not_backups() {
        for json in ["not json", "{}", r#"{"app":"something-else"}"#, "[1,2,3]"] {
            assert!(matches!(parse(json), Err(AppError::Invalid(m)) if m.contains("isn't a Habit Levelling")));
        }
        let future = format!(r#"{{"app":"{APP_TAG}","formatVersion":99}}"#);
        assert!(matches!(parse(&future), Err(AppError::Invalid(m)) if m.contains("newer version")));
    }

    #[tokio::test]
    async fn rejects_damaged_backups_and_keeps_current_data() {
        let pool = sample().await;
        let good = export(&pool).await.unwrap();

        let mut orphan = good.clone();
        orphan.completions[0].habit_id = 99;
        let mut twice = good.clone();
        twice.completions.push(twice.completions[0].clone());
        let mut luck = good.clone();
        luck.habits[1].primary_stat = Some(Luck);
        let mut too_long = good.clone();
        too_long.completions[0].minutes = 5000;

        for bad in [orphan, twice, luck, too_long] {
            assert!(matches!(parse(&bad.to_json()), Err(AppError::Invalid(m)) if m.contains("damaged")));
            assert!(restore(&pool, &bad).await.is_err());
        }
        let mut now = export(&pool).await.unwrap();
        now.exported_at = good.exported_at.clone();
        assert_eq!(now, good, "nothing changed");
    }

    #[test]
    fn unique_paths_never_overwrite() {
        let dir = temp_dir("unique");
        std::fs::create_dir_all(&dir).unwrap();
        let a = unique_path(&dir, "backup");
        std::fs::write(&a, "x").unwrap();
        let b = unique_path(&dir, "backup");
        assert_eq!(b.file_name().unwrap(), "backup-2.json");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn automatic_backups_once_a_day_keeping_the_newest() {
        let dir = temp_dir("auto");
        let empty = db::test_pool().await;
        assert_eq!(auto_backup(&empty, &dir, d("2026-11-01")).await.unwrap(), None, "nothing to back up yet");

        let pool = sample().await;
        for day in 1..=20 {
            let date = NaiveDate::from_ymd_opt(2026, 10, day).unwrap();
            assert!(auto_backup(&pool, &dir, date).await.unwrap().is_some());
        }
        assert!(auto_backup(&pool, &dir, d("2026-10-20")).await.unwrap().is_none(), "once a day");

        let kept = auto_backups(&dir);
        assert_eq!(kept.len(), AUTO_BACKUPS_KEPT);
        assert!(kept[0].ends_with("auto-2026-10-20.json"), "newest first");
        assert!(kept.last().unwrap().ends_with("auto-2026-10-07.json"));
        // And each one restores.
        let json = std::fs::read_to_string(&kept[0]).unwrap();
        assert_eq!(parse(&json).unwrap().habits.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
