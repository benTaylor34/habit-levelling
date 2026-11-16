//! Habit CRUD and completion tracking against SQLite.
//!
//! These functions take a plain `&SqlitePool` and an explicit `today`, so they
//! can be tested against an in-memory database without Tauri or the clock.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{
    Completion, CompletionInput, Habit, HabitDetails, HabitInput, HabitSummary, LuckWin,
    StatHistory, StatProgress, XpPreview,
};
use crate::history::{self, StatGain};
use crate::stats::{self, Stat};
use crate::{luck, streaks};

const DATE_FORMAT: &str = "%Y-%m-%d";
const HABIT_COLUMNS: &str =
    "id, name, description, primary_stat, secondary_stat, created_at, archived";

/// Trim the input, turn empty optional strings into NULL and check the stats.
fn clean(input: HabitInput) -> AppResult<HabitInput> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("Habit name can't be empty".into()));
    }
    for stat in [input.primary_stat, input.secondary_stat].into_iter().flatten() {
        if !Stat::TRAINABLE.contains(&stat) {
            return Err(AppError::Invalid("Luck can't be trained directly".into()));
        }
    }
    match (input.primary_stat, input.secondary_stat) {
        (None, Some(_)) => {
            return Err(AppError::Invalid("Pick a primary stat before a secondary one".into()))
        }
        (Some(a), Some(b)) if a == b => {
            return Err(AppError::Invalid(
                "Primary and secondary stats must be different".into(),
            ))
        }
        _ => {}
    }
    let opt = |s: Option<String>| s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    Ok(HabitInput {
        name,
        description: opt(input.description),
        ..input
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
        "INSERT INTO habits (name, description, primary_stat, secondary_stat)
         VALUES (?, ?, ?, ?) RETURNING {HABIT_COLUMNS}"
    );
    Ok(sqlx::query_as(&sql)
        .bind(input.name)
        .bind(input.description)
        .bind(input.primary_stat)
        .bind(input.secondary_stat)
        .fetch_one(pool)
        .await?)
}

pub async fn update(pool: &SqlitePool, id: i64, input: HabitInput) -> AppResult<Habit> {
    let input = clean(input)?;
    let sql = format!(
        "UPDATE habits SET name = ?, description = ?, primary_stat = ?, secondary_stat = ?
         WHERE id = ? RETURNING {HABIT_COLUMNS}"
    );
    sqlx::query_as(&sql)
        .bind(input.name)
        .bind(input.description)
        .bind(input.primary_stat)
        .bind(input.secondary_stat)
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

/// Log the habit as done on `date` with the time spent and difficulty, rolling
/// for Luck if this completes a streak milestone.
pub async fn complete(
    pool: &SqlitePool,
    id: i64,
    date: NaiveDate,
    log: CompletionInput,
) -> AppResult<Option<LuckWin>> {
    complete_with_roll(pool, id, date, log, luck::roll_d100()).await
}

/// `complete` with an explicit d100 roll (0–99), so tests can control luck.
/// Logging again on the same day replaces that day's entry (UNIQUE(habit_id, date))
/// and never re-rolls: each habit gets one Luck roll per milestone day.
pub async fn complete_with_roll(
    pool: &SqlitePool,
    id: i64,
    date: NaiveDate,
    log: CompletionInput,
    roll: u32,
) -> AppResult<Option<LuckWin>> {
    if log.minutes > stats::MAX_MINUTES {
        return Err(AppError::Invalid("Time spent can't be more than 24 hours".into()));
    }
    if !(stats::MIN_DIFFICULTY..=stats::MAX_DIFFICULTY).contains(&log.difficulty) {
        return Err(AppError::Invalid("Difficulty must be between 1 and 5".into()));
    }
    get(pool, id).await?; // clear NotFound error instead of an FK failure
    let date_str = date.format(DATE_FORMAT).to_string();
    sqlx::query(
        "INSERT INTO habit_completions (habit_id, date, minutes, difficulty) VALUES (?, ?, ?, ?)
         ON CONFLICT (habit_id, date) DO UPDATE SET
             minutes = excluded.minutes, difficulty = excluded.difficulty",
    )
    .bind(id)
    .bind(&date_str)
    .bind(log.minutes)
    .bind(log.difficulty)
    .execute(pool)
    .await?;

    let streak = streaks::current_streak(&dates_of(&completions(pool, id).await?), date);
    let Some(milestone) = luck::milestone(streak) else {
        return Ok(None);
    };
    let xp = luck::award(milestone, roll);
    let inserted = sqlx::query(
        "INSERT OR IGNORE INTO luck_rolls (habit_id, date, streak, xp) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&date_str)
    .bind(streak)
    .bind(xp)
    .execute(pool)
    .await?
    .rows_affected();
    Ok((inserted == 1 && xp > 0).then_some(LuckWin { xp, streak }))
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

/// Every logged day for a habit, oldest first, with each day's streak and XP.
pub async fn completions(pool: &SqlitePool, id: i64) -> AppResult<Vec<Completion>> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT date, minutes, difficulty FROM habit_completions WHERE habit_id = ? ORDER BY date",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let dates = rows
        .iter()
        .map(|(date, _, _)| parse_date(date))
        .collect::<AppResult<Vec<_>>>()?;
    let streaks = streaks::streak_lengths(&dates);
    Ok(rows
        .iter()
        .zip(dates)
        .zip(streaks)
        .map(|(((_, minutes, difficulty), date), streak)| {
            let (minutes, difficulty) = ((*minutes).max(0) as u32, (*difficulty).max(0) as u32);
            Completion {
                date,
                minutes,
                difficulty,
                streak,
                xp: stats::completion_xp(minutes, difficulty, streak),
            }
        })
        .collect())
}

/// XP a log for `date` would earn, using the streak it would extend.
pub async fn preview_xp(
    pool: &SqlitePool,
    id: i64,
    date: NaiveDate,
    minutes: u32,
    difficulty: u32,
) -> AppResult<XpPreview> {
    let dates = dates_of(&completions(pool, id).await?);
    let streak = streaks::streak_if_done_on(&dates, date);
    let primary = stats::completion_xp(minutes, difficulty, streak);
    Ok(XpPreview {
        primary,
        secondary: stats::secondary_xp(primary),
        streak,
        streak_percent: stats::streak_percent(streak),
    })
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

/// Level, XP and streaks for all six stats, plus Luck once it has any XP
/// (see `stat_gains` for how XP is shared out). A stat's streak counts days on which *any* habit training
/// it was done.
/// Every XP gain any stat has received: a habit's primary stat gets each
/// completion's full XP (time × difficulty × that day's streak bonus) and its
/// secondary stat gets half.
pub async fn stat_gains(pool: &SqlitePool) -> AppResult<Vec<StatGain>> {
    let mut out = Vec::new();
    for habit in list(pool).await? {
        let Some(primary) = habit.primary_stat else { continue };
        // Per-habit history so each completion carries that habit's streak bonus.
        for c in completions(pool, habit.id).await? {
            out.push(StatGain { date: c.date, stat: primary, xp: c.xp });
            if let Some(secondary) = habit.secondary_stat {
                out.push(StatGain {
                    date: c.date,
                    stat: secondary,
                    xp: stats::secondary_xp(c.xp),
                });
            }
        }
    }
    Ok(out)
}

/// Daily stat XP and levels from `days` ago (or all time) up to `today`.
pub async fn stat_history(
    pool: &SqlitePool,
    today: NaiveDate,
    days: Option<u32>,
) -> AppResult<StatHistory> {
    let gains = stat_gains(pool).await?;
    let start = history::range_start(&gains, today, days);
    Ok(history::build(&gains, start, today))
}

pub async fn stat_progress(pool: &SqlitePool, today: NaiveDate) -> AppResult<Vec<StatProgress>> {
    let mut xp: BTreeMap<Stat, u32> = BTreeMap::new();
    let mut dates: BTreeMap<Stat, Vec<NaiveDate>> = BTreeMap::new();
    for g in stat_gains(pool).await? {
        *xp.entry(g.stat).or_default() += g.xp;
        dates.entry(g.stat).or_default().push(g.date);
    }

    let mut out: Vec<StatProgress> = Stat::TRAINABLE
        .iter()
        .map(|&stat| {
            let days = dates.get(&stat).map(Vec::as_slice).unwrap_or_default();
            let mut p = stats::progress(stat, xp.get(&stat).copied().unwrap_or(0));
            p.current_streak = streaks::current_streak(days, today);
            p.longest_streak = streaks::longest_streak(days);
            p
        })
        .collect();

    // Luck stays secret until the first lucky roll.
    let (luck_xp,): (i64,) = sqlx::query_as("SELECT COALESCE(SUM(xp), 0) FROM luck_rolls")
        .fetch_one(pool)
        .await?;
    if luck_xp > 0 {
        out.push(stats::progress(Stat::Luck, luck_xp as u32));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::stats::Stat::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).unwrap()
    }

    /// A plain completion: no time tracked, easiest difficulty (10 XP).
    const PLAIN: CompletionInput = CompletionInput { minutes: 0, difficulty: 1 };

    fn log(minutes: u32, difficulty: u32) -> CompletionInput {
        CompletionInput { minutes, difficulty }
    }

    fn input(name: &str, primary: Option<Stat>, secondary: Option<Stat>) -> HabitInput {
        HabitInput {
            name: name.into(),
            description: None,
            primary_stat: primary,
            secondary_stat: secondary,
        }
    }

    fn stat<'a>(all: &'a [StatProgress], stat: Stat) -> &'a StatProgress {
        all.iter().find(|p| p.stat == stat).unwrap()
    }

    #[tokio::test]
    async fn create_list_edit_delete() {
        let pool = db::test_pool().await;

        let h = create(&pool, input("  Exercise ", Some(Strength), Some(Constitution))).await.unwrap();
        assert_eq!(h.name, "Exercise");
        assert_eq!(h.primary_stat, Some(Strength));
        assert_eq!(h.secondary_stat, Some(Constitution));
        assert!(!h.archived);

        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 1);

        let edited = update(
            &pool,
            h.id,
            HabitInput {
                name: "Run".into(),
                description: Some("5k".into()),
                primary_stat: Some(Dexterity),
                secondary_stat: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(edited.name, "Run");
        assert_eq!(edited.description.as_deref(), Some("5k"));
        assert_eq!(edited.primary_stat, Some(Dexterity));
        assert_eq!(edited.secondary_stat, None);

        delete(&pool, h.id).await.unwrap();
        assert!(list(&pool).await.unwrap().is_empty());
        assert!(matches!(delete(&pool, h.id).await, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn empty_name_is_rejected() {
        let pool = db::test_pool().await;
        assert!(matches!(
            create(&pool, input("   ", None, None)).await,
            Err(AppError::Invalid(_))
        ));
    }

    #[tokio::test]
    async fn logging_twice_a_day_counts_once_and_undo_works() {
        let pool = db::test_pool().await;
        let today = d("2026-11-14");
        let h = create(&pool, input("Read", None, None)).await.unwrap();

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
        let h = create(&pool, input("Exercise", None, None)).await.unwrap();
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
        let h = create(&pool, input("Exercise", None, None)).await.unwrap();
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
        let h = create(&pool, input("Read", None, None)).await.unwrap();

        complete(&pool, h.id, today, log(20, 1)).await.unwrap();
        complete(&pool, h.id, today, log(45, 3)).await.unwrap();
        let s = summary(&pool, h.id, today).await.unwrap();
        assert_eq!(s.total_completions, 1);
        assert_eq!(s.total_minutes, 45);
        assert_eq!(s.total_xp, stats::completion_xp(45, 3, 1));
        let today_log = s.today_log.unwrap();
        assert_eq!((today_log.minutes, today_log.difficulty), (45, 3));
    }

    #[tokio::test]
    async fn invalid_logs_are_rejected() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", None, None)).await.unwrap();
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
    async fn invalid_stat_choices_are_rejected() {
        let pool = db::test_pool().await;
        for bad in [
            input("Same twice", Some(Wisdom), Some(Wisdom)),
            input("Secondary only", None, Some(Wisdom)),
            input("Luck", Some(Luck), None),
            input("Luck 2", Some(Wisdom), Some(Luck)),
        ] {
            assert!(matches!(create(&pool, bad).await, Err(AppError::Invalid(_))));
        }
    }

    #[tokio::test]
    async fn primary_gets_full_xp_and_secondary_gets_half() {
        let pool = db::test_pool().await;
        let study = create(&pool, input("Study", Some(Intelligence), Some(Wisdom))).await.unwrap();
        let lift = create(&pool, input("Lift", Some(Strength), Some(Constitution))).await.unwrap();
        create(&pool, input("Untagged", None, None)).await.unwrap();

        complete(&pool, study.id, d("2026-11-13"), log(30, 2)).await.unwrap(); // 60
        complete(&pool, study.id, d("2026-11-14"), log(60, 3)).await.unwrap(); // 140 × 1.05 (day 2) = 147
        complete(&pool, lift.id, d("2026-11-14"), log(45, 4)).await.unwrap(); // 137

        let all = stat_progress(&pool, d("2026-11-14")).await.unwrap();
        let xp: Vec<_> = all.iter().map(|p| (p.stat, p.xp)).collect();
        assert_eq!(
            xp,
            vec![
                (Strength, 137),
                (Dexterity, 0),
                (Constitution, 68),
                (Intelligence, 207),
                (Wisdom, 103),
                (Charisma, 0),
            ],
            "all six stats are always listed; Luck stays hidden"
        );
        // 207 XP: level 2 needs 70, level 3 needs 70 + 190 = 260.
        let int = stat(&all, Intelligence);
        assert_eq!((int.level, int.xp_into_level, int.xp_for_next_level), (2, 137, 190));
        assert_eq!(stat(&all, Dexterity).level, 1);
    }

    #[tokio::test]
    async fn stat_streak_counts_days_any_habit_trained_it() {
        let pool = db::test_pool().await;
        let run = create(&pool, input("Run", Some(Dexterity), Some(Constitution))).await.unwrap();
        let swim = create(&pool, input("Swim", Some(Constitution), None)).await.unwrap();
        let today = d("2026-11-14");

        // Run and swim alternate, so neither habit has a long streak on its
        // own, but Constitution was trained every day from the 11th to the 14th.
        complete(&pool, run.id, d("2026-11-08"), PLAIN).await.unwrap();
        complete(&pool, run.id, d("2026-11-11"), PLAIN).await.unwrap();
        complete(&pool, swim.id, d("2026-11-12"), PLAIN).await.unwrap();
        complete(&pool, run.id, d("2026-11-13"), PLAIN).await.unwrap();
        complete(&pool, swim.id, d("2026-11-14"), PLAIN).await.unwrap();

        let all = stat_progress(&pool, today).await.unwrap();
        assert_eq!(stat(&all, Constitution).current_streak, 4);
        assert_eq!(stat(&all, Constitution).longest_streak, 4);
        assert_eq!(stat(&all, Dexterity).current_streak, 1);
        assert_eq!(summary(&pool, run.id, today).await.unwrap().current_streak, 1);
    }

    #[tokio::test]
    async fn streak_bonus_grows_xp_day_by_day() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", Some(Intelligence), None)).await.unwrap();
        for day in ["2026-11-01", "2026-11-02", "2026-11-03", "2026-11-05"] {
            complete(&pool, h.id, d(day), log(30, 1)).await.unwrap();
        }
        let det = details(&pool, h.id, d("2026-11-05")).await.unwrap();
        let got: Vec<_> = det.completions.iter().map(|c| (c.streak, c.xp)).collect();
        // 40 base XP × 1.0, 1.05, 1.10, then the gap resets to ×1.0.
        assert_eq!(got, vec![(1, 40), (2, 42), (3, 44), (1, 40)]);
        assert_eq!(det.summary.total_xp, 166);
    }

    #[tokio::test]
    async fn preview_uses_the_streak_the_log_would_extend() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", Some(Intelligence), None)).await.unwrap();
        log_streak(&pool, h.id, d("2026-11-06"), 6, 99).await;

        let p = preview_xp(&pool, h.id, d("2026-11-07"), 30, 3).await.unwrap();
        assert_eq!((p.streak, p.streak_percent, p.primary, p.secondary), (7, 130, 104, 52));
        // Matches what logging it actually earns.
        complete_with_roll(&pool, h.id, d("2026-11-07"), log(30, 3), 99).await.unwrap();
        let s = summary(&pool, h.id, d("2026-11-07")).await.unwrap();
        assert_eq!(s.today_log.unwrap().xp, 104);
        // A missed day starts the bonus over.
        let p = preview_xp(&pool, h.id, d("2026-11-09"), 30, 3).await.unwrap();
        assert_eq!((p.streak, p.primary), (1, 80));
    }

    #[tokio::test]
    async fn stat_history_matches_stat_progress() {
        let pool = db::test_pool().await;
        let study = create(&pool, input("Study", Some(Intelligence), Some(Wisdom))).await.unwrap();
        let today = d("2026-11-16");
        complete(&pool, study.id, d("2026-11-14"), log(30, 2)).await.unwrap(); // 60
        complete(&pool, study.id, d("2026-11-16"), log(30, 2)).await.unwrap(); // 60

        let h = stat_history(&pool, today, Some(3)).await.unwrap();
        assert_eq!(h.dates, vec![d("2026-11-14"), d("2026-11-15"), today]);
        let int = h.series.iter().find(|s| s.stat == Intelligence).unwrap();
        assert_eq!(int.xp, vec![60, 60, 120]);
        let wis = h.series.iter().find(|s| s.stat == Wisdom).unwrap();
        assert_eq!(wis.xp, vec![30, 30, 60]);

        // The last day of the history agrees with the Stats panel.
        let now = stat_progress(&pool, today).await.unwrap();
        for s in &h.series {
            let p = stat(&now, s.stat);
            assert_eq!((*s.xp.last().unwrap(), s.end_level), (p.xp, p.level));
        }
    }

    /// Log `days` consecutive days ending on `last`, rolling `roll` each time.
    async fn log_streak(pool: &SqlitePool, id: i64, last: NaiveDate, days: u64, roll: u32) {
        for back in (0..days).rev() {
            let day = last - chrono::Days::new(back);
            complete_with_roll(pool, id, day, PLAIN, roll).await.unwrap();
        }
    }

    #[tokio::test]
    async fn lucky_roll_on_a_milestone_unlocks_luck() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", Some(Intelligence), None)).await.unwrap();
        let day6 = d("2026-11-06");

        // Days 1–6 miss every roll, but they aren't milestones anyway.
        log_streak(&pool, h.id, day6, 6, 0).await;
        assert!(stat_progress(&pool, day6).await.unwrap().iter().all(|p| p.stat != Luck));

        // Day 7 is a weekly milestone; a roll of 0 always wins.
        let win = complete_with_roll(&pool, h.id, d("2026-11-07"), PLAIN, 0).await.unwrap();
        assert_eq!(win, Some(LuckWin { xp: luck::WEEKLY.xp, streak: 7 }));
        let luck_stat = stat(&stat_progress(&pool, d("2026-11-07")).await.unwrap(), Luck).clone();
        assert_eq!(luck_stat.xp, luck::WEEKLY.xp);
    }

    #[tokio::test]
    async fn unlucky_roll_gives_nothing_and_cannot_be_rerolled() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Read", Some(Intelligence), None)).await.unwrap();
        let day7 = d("2026-11-07");

        log_streak(&pool, h.id, day7, 7, 99).await; // day 7 rolls 99: a miss
        assert_eq!(complete_with_roll(&pool, h.id, day7, PLAIN, 0).await.unwrap(), None);
        uncomplete(&pool, h.id, day7).await.unwrap();
        assert_eq!(complete_with_roll(&pool, h.id, day7, PLAIN, 0).await.unwrap(), None);
        assert!(stat_progress(&pool, day7).await.unwrap().iter().all(|p| p.stat != Luck));
    }

    #[tokio::test]
    async fn monthly_milestone_awards_the_bigger_prize() {
        let pool = db::test_pool().await;
        let h = create(&pool, input("Run", Some(Dexterity), None)).await.unwrap();
        let day30 = d("2026-11-30");
        log_streak(&pool, h.id, d("2026-11-29"), 29, 99).await;
        let win = complete_with_roll(&pool, h.id, day30, PLAIN, 0).await.unwrap();
        assert_eq!(win, Some(LuckWin { xp: luck::MONTHLY.xp, streak: 30 }));
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
        let h = create(&pool, input("Exercise", Some(Strength), None)).await.unwrap();
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
