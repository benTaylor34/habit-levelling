//! Habit CRUD and completion tracking against SQLite.
//!
//! These functions take a plain `&SqlitePool` and an explicit `today`, so they
//! can be tested against an in-memory database without Tauri or the clock.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{
    Completion, CompletionInput, Habit, HabitDetails, HabitInput, HabitSummary, SkillProgress,
};
use crate::{skills, streaks};

const DATE_FORMAT: &str = "%Y-%m-%d";
const HABIT_COLUMNS: &str = "id, name, description, skill, created_at, archived";

/// Trim the input and turn empty optional strings into NULL.
fn clean(input: HabitInput) -> AppResult<HabitInput> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("Habit name can't be empty".into()));
    }
    let opt = |s: Option<String>| s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    Ok(HabitInput {
        name,
        description: opt(input.description),
        skill: opt(input.skill),
    })
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Habit>> {
    let sql = format!("SELECT {HABIT_COLUMNS} FROM habits WHERE archived = 0 ORDER BY id");
    Ok(sqlx::query_as(&sql).fetch_all(pool).await?)
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Habit> {
    let sql = format!("SELECT {HABIT_COLUMNS} FROM habits WHERE id = ?");
    sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound(id))
}

pub async fn create(pool: &SqlitePool, input: HabitInput) -> AppResult<Habit> {
    let input = clean(input)?;
    let sql = format!(
        "INSERT INTO habits (name, description, skill) VALUES (?, ?, ?) RETURNING {HABIT_COLUMNS}"
    );
    Ok(sqlx::query_as(&sql)
        .bind(input.name)
        .bind(input.description)
        .bind(input.skill)
        .fetch_one(pool)
        .await?)
}

pub async fn update(pool: &SqlitePool, id: i64, input: HabitInput) -> AppResult<Habit> {
    let input = clean(input)?;
    let sql = format!(
        "UPDATE habits SET name = ?, description = ?, skill = ? WHERE id = ? RETURNING {HABIT_COLUMNS}"
    );
    sqlx::query_as(&sql)
        .bind(input.name)
        .bind(input.description)
        .bind(input.skill)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound(id))
}

/// Delete a habit and (via ON DELETE CASCADE) all of its completions.
pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM habits WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(id));
    }
    Ok(())
}

/// Log the habit as done on `date` with the time spent and difficulty.
/// Logging again on the same day replaces that day's entry (UNIQUE(habit_id, date)).
pub async fn complete(
    pool: &SqlitePool,
    id: i64,
    date: NaiveDate,
    log: CompletionInput,
) -> AppResult<()> {
    if log.minutes > skills::MAX_MINUTES {
        return Err(AppError::Invalid("Time spent can't be more than 24 hours".into()));
    }
    if !(skills::MIN_DIFFICULTY..=skills::MAX_DIFFICULTY).contains(&log.difficulty) {
        return Err(AppError::Invalid("Difficulty must be between 1 and 5".into()));
    }
    get(pool, id).await?; // clear NotFound error instead of an FK failure
    sqlx::query(
        "INSERT INTO habit_completions (habit_id, date, minutes, difficulty) VALUES (?, ?, ?, ?)
         ON CONFLICT (habit_id, date) DO UPDATE SET
             minutes = excluded.minutes, difficulty = excluded.difficulty",
    )
    .bind(id)
    .bind(date.format(DATE_FORMAT).to_string())
    .bind(log.minutes)
    .bind(log.difficulty)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the completion on `date`, if any.
pub async fn uncomplete(pool: &SqlitePool, id: i64, date: NaiveDate) -> AppResult<()> {
    sqlx::query("DELETE FROM habit_completions WHERE habit_id = ? AND date = ?")
        .bind(id)
        .bind(date.format(DATE_FORMAT).to_string())
        .execute(pool)
        .await?;
    Ok(())
}

fn parse_date(s: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATE_FORMAT)
        .map_err(|e| AppError::Invalid(format!("Bad date '{s}' in database: {e}")))
}

fn to_completion(date: &str, minutes: i64, difficulty: i64) -> AppResult<Completion> {
    let (minutes, difficulty) = (minutes.max(0) as u32, difficulty.max(0) as u32);
    Ok(Completion {
        date: parse_date(date)?,
        minutes,
        difficulty,
        xp: skills::completion_xp(minutes, difficulty),
    })
}

/// Every logged day for a habit, oldest first.
pub async fn completions(pool: &SqlitePool, id: i64) -> AppResult<Vec<Completion>> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT date, minutes, difficulty FROM habit_completions WHERE habit_id = ? ORDER BY date",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|(date, minutes, difficulty)| to_completion(date, *minutes, *difficulty))
        .collect()
}

fn dates_of(completions: &[Completion]) -> Vec<NaiveDate> {
    completions.iter().map(|c| c.date).collect()
}

fn summarize(habit: Habit, completions: &[Completion], today: NaiveDate) -> HabitSummary {
    let dates = dates_of(completions);
    HabitSummary {
        current_streak: streaks::current_streak(&dates, today),
        longest_streak: streaks::longest_streak(&dates),
        completed_today: dates.contains(&today),
        total_completions: completions.len() as u32,
        total_minutes: completions.iter().map(|c| c.minutes).sum(),
        total_xp: completions.iter().map(|c| c.xp).sum(),
        today_log: completions.iter().find(|c| c.date == today).cloned(),
        recent_days: streaks::recent_days(&dates, today, 7),
        habit,
    }
}

pub async fn summary(pool: &SqlitePool, id: i64, today: NaiveDate) -> AppResult<HabitSummary> {
    let habit = get(pool, id).await?;
    let completions = completions(pool, id).await?;
    Ok(summarize(habit, &completions, today))
}

pub async fn summaries(pool: &SqlitePool, today: NaiveDate) -> AppResult<Vec<HabitSummary>> {
    let mut out = Vec::new();
    for habit in list(pool).await? {
        let completions = completions(pool, habit.id).await?;
        out.push(summarize(habit, &completions, today));
    }
    Ok(out)
}

pub async fn details(pool: &SqlitePool, id: i64, today: NaiveDate) -> AppResult<HabitDetails> {
    let habit = get(pool, id).await?;
    let completions = completions(pool, id).await?;
    Ok(HabitDetails {
        summary: summarize(habit, &completions, today),
        completions,
    })
}

/// XP and streaks per skill, from every completion of the skill's habits.
/// A skill's streak counts days on which *any* of its habits was done.
/// Skills with habits but no completions yet show up with 0 XP.
pub async fn skill_progress(pool: &SqlitePool, today: NaiveDate) -> AppResult<Vec<SkillProgress>> {
    let rows: Vec<(String, Option<String>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT h.skill, c.date, c.minutes, c.difficulty
         FROM habits h
         LEFT JOIN habit_completions c ON c.habit_id = h.id
         WHERE h.skill IS NOT NULL AND h.archived = 0",
    )
    .fetch_all(pool)
    .await?;

    let mut by_skill: BTreeMap<String, Vec<Completion>> = BTreeMap::new();
    for (skill, date, minutes, difficulty) in rows {
        let entry = by_skill.entry(skill).or_default();
        if let Some(date) = date {
            entry.push(to_completion(&date, minutes.unwrap_or(0), difficulty.unwrap_or(1))?);
        }
    }

    let mut out: Vec<SkillProgress> = by_skill
        .into_iter()
        .map(|(skill, completions)| {
            let dates = dates_of(&completions);
            let mut p = skills::progress(skill, completions.iter().map(|c| c.xp).sum());
            p.current_streak = streaks::current_streak(&dates, today);
            p.longest_streak = streaks::longest_streak(&dates);
            p
        })
        .collect();
    out.sort_by(|a, b| b.xp.cmp(&a.xp).then_with(|| a.skill.cmp(&b.skill)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).unwrap()
    }

    /// A plain completion: no time tracked, easiest difficulty (10 XP).
    const PLAIN: CompletionInput = CompletionInput { minutes: 0, difficulty: 1 };

    fn log(minutes: u32, difficulty: u32) -> CompletionInput {
        CompletionInput { minutes, difficulty }
    }

    fn input(name: &str, skill: Option<&str>) -> HabitInput {
        HabitInput {
            name: name.into(),
            description: None,
            skill: skill.map(Into::into),
        }
    }

    #[tokio::test]
    async fn create_list_edit_delete() {
        let pool = db::test_pool().await;

        let h = create(&pool, input("  Exercise ", Some("Fitness"))).await.unwrap();
        assert_eq!(h.name, "Exercise");
        assert_eq!(h.skill.as_deref(), Some("Fitness"));
        assert!(!h.archived);

        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 1);

        let edited = update(
            &pool,
            h.id,
            HabitInput {
                name: "Run".into(),
                description: Some("5k".into()),
                skill: Some("".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(edited.name, "Run");
        assert_eq!(edited.description.as_deref(), Some("5k"));
        assert_eq!(edited.skill, None, "empty skill becomes NULL");

        delete(&pool, h.id).await.unwrap();
        assert!(list(&pool).await.unwrap().is_empty());
        assert!(matches!(delete(&pool, h.id).await, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn empty_name_is_rejected() {
        let pool = db::test_pool().await;
        assert!(matches!(
            create(&pool, input("   ", None)).await,
            Err(AppError::Invalid(_))
        ));
    }

    #[tokio::test]
    async fn logging_twice_a_day_counts_once_and_undo_works() {
        let pool = db::test_pool().await;
        let today = d("2026-11-14");
        let h = create(&pool, input("Read", None)).await.unwrap();

        complete(&pool, h.id, today, PLAIN).await.unwrap();
        complete(&pool, h.id, today, PLAIN).await.unwrap();
        complete(&pool, h.id, today, PLAIN).await.unwrap();
        let s = summary(&pool, h.id, today).await.unwrap();
        assert_eq!(s.total_completions, 1);
        assert!(s.completed_today);
        assert_eq!(s.current_streak, 1);

        uncomplete(&pool, h.id, today).await.unwrap();
        let s = summary(&pool, h.id, today).await.unwrap();
        assert_eq!(s.total_completions, 0);
        assert!(!s.completed_today);
    }

    #[tokio::test]
    async fn streaks_from_history() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Exercise", None)).await.unwrap();
        for day in ["2026-11-10", "2026-11-11", "2026-11-13", "2026-11-14"] {
            complete(&pool, h.id, d(day), PLAIN).await.unwrap();
        }
        let det = details(&pool, h.id, d("2026-11-14")).await.unwrap();
        assert_eq!(det.summary.current_streak, 2);
        assert_eq!(det.summary.longest_streak, 2);
        assert_eq!(det.completions.len(), 4);
        assert_eq!(det.completions[0].date, d("2026-11-10"));
    }

    #[tokio::test]
    async fn deleting_habit_removes_completions() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Exercise", None)).await.unwrap();
        complete(&pool, h.id, d("2026-11-14"), PLAIN).await.unwrap();
        delete(&pool, h.id).await.unwrap();
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM habit_completions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn completing_missing_habit_is_not_found() {
        let pool = db::test_pool().await;
        assert!(matches!(
            complete(&pool, 99, d("2026-11-14"), PLAIN).await,
            Err(AppError::NotFound(99))
        ));
    }

    #[tokio::test]
    async fn logging_again_replaces_the_days_entry() {
        let pool = db::test_pool().await;
        let today = d("2026-11-14");
        let h = create(&pool, input("Read", None)).await.unwrap();

        complete(&pool, h.id, today, log(20, 1)).await.unwrap();
        complete(&pool, h.id, today, log(45, 3)).await.unwrap();
        let s = summary(&pool, h.id, today).await.unwrap();
        assert_eq!(s.total_completions, 1);
        assert_eq!(s.total_minutes, 45);
        assert_eq!(s.total_xp, skills::completion_xp(45, 3));
        let today_log = s.today_log.unwrap();
        assert_eq!((today_log.minutes, today_log.difficulty), (45, 3));
    }

    #[tokio::test]
    async fn invalid_logs_are_rejected() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", None)).await.unwrap();
        let today = d("2026-11-14");
        for bad in [log(0, 0), log(0, 6), log(24 * 60 + 1, 2)] {
            assert!(matches!(
                complete(&pool, h.id, today, bad).await,
                Err(AppError::Invalid(_))
            ));
        }
        assert_eq!(summary(&pool, h.id, today).await.unwrap().total_completions, 0);
    }

    #[tokio::test]
    async fn skill_xp_comes_from_time_and_difficulty() {
        let pool = db::test_pool().await;
        let run = create(&pool, input("Run", Some("Fitness"))).await.unwrap();
        let gym = create(&pool, input("Gym", Some("Fitness"))).await.unwrap();
        create(&pool, input("Read", Some("Knowledge"))).await.unwrap();
        create(&pool, input("Untagged", None)).await.unwrap();

        complete(&pool, run.id, d("2026-11-13"), log(30, 2)).await.unwrap(); // 60
        complete(&pool, run.id, d("2026-11-14"), PLAIN).await.unwrap(); // 10
        complete(&pool, gym.id, d("2026-11-14"), log(60, 3)).await.unwrap(); // 140

        let skills = skill_progress(&pool, d("2026-11-14")).await.unwrap();
        let pairs: Vec<_> = skills.iter().map(|s| (s.skill.as_str(), s.xp)).collect();
        assert_eq!(pairs, vec![("Fitness", 210), ("Knowledge", 0)]);
        assert_eq!(skills[0].level, 2);
    }

    #[tokio::test]
    async fn skill_streak_counts_days_any_habit_was_done() {
        let pool = db::test_pool().await;
        let run = create(&pool, input("Run", Some("Fitness"))).await.unwrap();
        let gym = create(&pool, input("Gym", Some("Fitness"))).await.unwrap();
        let today = d("2026-11-14");

        // Run and gym alternate, so neither habit has a long streak on its
        // own, but Fitness was trained every day from the 11th to the 14th.
        complete(&pool, run.id, d("2026-11-08"), PLAIN).await.unwrap();
        complete(&pool, run.id, d("2026-11-11"), PLAIN).await.unwrap();
        complete(&pool, gym.id, d("2026-11-12"), PLAIN).await.unwrap();
        complete(&pool, run.id, d("2026-11-13"), PLAIN).await.unwrap();
        complete(&pool, gym.id, d("2026-11-13"), PLAIN).await.unwrap();
        complete(&pool, gym.id, d("2026-11-14"), PLAIN).await.unwrap();

        let fitness = &skill_progress(&pool, today).await.unwrap()[0];
        assert_eq!(fitness.current_streak, 4);
        assert_eq!(fitness.longest_streak, 4);
        assert_eq!(summary(&pool, run.id, today).await.unwrap().current_streak, 1);
        assert_eq!(summary(&pool, gym.id, today).await.unwrap().current_streak, 3);
    }

    /// Definition of done: data survives closing and reopening the app.
    #[tokio::test]
    async fn data_persists_across_restarts() {
        let path = std::env::temp_dir().join(format!(
            "habit_levelling_test_{}_{}.db",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let today = d("2026-11-14");

        let pool = db::init_pool(&path).await.unwrap();
        let h = create(&pool, input("Exercise", Some("Fitness"))).await.unwrap();
        complete(&pool, h.id, d("2026-11-13"), PLAIN).await.unwrap();
        complete(&pool, h.id, today, PLAIN).await.unwrap();
        pool.close().await;

        // "Reopen the app": new pool on the same file, migrations re-run harmlessly.
        let pool = db::init_pool(&path).await.unwrap();
        let all = summaries(&pool, today).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].habit.name, "Exercise");
        assert_eq!(all[0].current_streak, 2);
        assert!(all[0].completed_today);
        pool.close().await;

        let _ = std::fs::remove_file(&path);
    }
}
