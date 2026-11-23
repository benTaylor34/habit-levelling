//! Daily quests in the app: today's offer (from habit_core) scaled to each
//! stat's level at the start of the day, plus recording completions.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::habits;
use habit_core::quests::{self, DailyQuest};
use habit_core::stats::Stat;

const DATE_FORMAT: &str = "%Y-%m-%d";

/// Stat levels at the end of `day`, for scaling quests. Decay is included.
async fn levels_on(pool: &SqlitePool, day: NaiveDate) -> AppResult<BTreeMap<Stat, u32>> {
    Ok(habits::stat_progress(pool, day)
        .await?
        .into_iter()
        .map(|p| (p.stat, p.level))
        .collect())
}

/// Today's three quests, marked completed where done. Targets use yesterday's
/// levels, so they don't change when a stat levels up during the day.
pub async fn today(pool: &SqlitePool, today: NaiveDate) -> AppResult<Vec<DailyQuest>> {
    let start_of_day = today.pred_opt().unwrap_or(today);
    let mut offer = quests::daily_quests(today, &levels_on(pool, start_of_day).await?);
    let done: Vec<(String,)> = sqlx::query_as("SELECT quest_id FROM quest_completions WHERE date = ?")
        .bind(today.format(DATE_FORMAT).to_string())
        .fetch_all(pool)
        .await?;
    for q in &mut offer {
        q.completed = done.iter().any(|(id,)| *id == q.id);
    }
    Ok(offer)
}

/// Mark one of today's quests done, recording its target and XP. Idempotent.
pub async fn complete(pool: &SqlitePool, today_date: NaiveDate, quest_id: &str) -> AppResult<Vec<DailyQuest>> {
    let offer = today(pool, today_date).await?;
    let q = offer
        .iter()
        .find(|q| q.id == quest_id)
        .ok_or_else(|| AppError::Invalid(format!("'{quest_id}' isn't one of today's quests")))?;
    sqlx::query(
        "INSERT OR IGNORE INTO quest_completions (date, quest_id, stat, amount, xp) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(today_date.format(DATE_FORMAT).to_string())
    .bind(&q.id)
    .bind(q.stat)
    .bind(q.amount)
    .bind(q.xp)
    .execute(pool)
    .await?;
    today(pool, today_date).await
}

/// Undo one of today's quests (its XP is removed with it).
pub async fn undo(pool: &SqlitePool, today_date: NaiveDate, quest_id: &str) -> AppResult<Vec<DailyQuest>> {
    sqlx::query("DELETE FROM quest_completions WHERE date = ? AND quest_id = ?")
        .bind(today_date.format(DATE_FORMAT).to_string())
        .bind(quest_id)
        .execute(pool)
        .await?;
    today(pool, today_date).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::habits::{complete as log_habit, create, stat_progress};
    use habit_core::models::{CompletionInput, HabitInput};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).unwrap()
    }

    fn xp_of(all: &[habit_core::models::StatProgress], stat: Stat) -> u32 {
        all.iter().find(|p| p.stat == stat).unwrap().xp
    }

    #[tokio::test]
    async fn completing_a_quest_gives_its_stat_xp_and_undo_takes_it_back() {
        let pool = db::test_pool().await;
        let day = d("2026-11-17");
        let offer = today(&pool, day).await.unwrap();
        assert_eq!(offer.len(), 3);
        assert!(offer.iter().all(|q| !q.completed));
        let q = offer[0].clone();

        let after = complete(&pool, day, &q.id).await.unwrap();
        assert!(after.iter().find(|x| x.id == q.id).unwrap().completed);
        assert_eq!(xp_of(&stat_progress(&pool, day).await.unwrap(), q.stat), q.xp);

        // Completing twice doesn't double the XP.
        complete(&pool, day, &q.id).await.unwrap();
        assert_eq!(xp_of(&stat_progress(&pool, day).await.unwrap(), q.stat), q.xp);

        let undone = undo(&pool, day, &q.id).await.unwrap();
        assert!(!undone.iter().any(|x| x.completed));
        assert_eq!(xp_of(&stat_progress(&pool, day).await.unwrap(), q.stat), 0);
    }

    #[tokio::test]
    async fn only_todays_quests_can_be_completed() {
        let pool = db::test_pool().await;
        let day = d("2026-11-17");
        let todays: Vec<String> = today(&pool, day).await.unwrap().into_iter().map(|q| q.id).collect();
        let other = quests::CATALOG.iter().find(|t| !todays.iter().any(|id| id == t.id)).unwrap();
        assert!(matches!(complete(&pool, day, other.id).await, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn targets_dont_move_when_a_stat_levels_up_mid_day() {
        let pool = db::test_pool().await;
        let day = d("2026-11-17");
        let before = today(&pool, day).await.unwrap();
        // Level up every stat today with a long, hard habit session each.
        for (i, stat) in Stat::TRAINABLE.iter().enumerate() {
            let h = create(
                &pool,
                HabitInput { name: format!("Train {i}"), description: None, primary_stat: Some(*stat), secondary_stat: None },
            )
            .await
            .unwrap();
            log_habit(&pool, h.id, day, CompletionInput { minutes: 240, difficulty: 5 }).await.unwrap();
        }
        assert!(stat_progress(&pool, day).await.unwrap().iter().all(|p| p.level > 1));
        let after: Vec<(String, u32, u32)> =
            today(&pool, day).await.unwrap().into_iter().map(|q| (q.id, q.amount, q.xp)).collect();
        let before: Vec<(String, u32, u32)> = before.into_iter().map(|q| (q.id, q.amount, q.xp)).collect();
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn quests_scale_from_yesterdays_level() {
        let pool = db::test_pool().await;
        let yesterday = d("2026-11-16");
        let day = d("2026-11-17");
        let fresh = today(&pool, day).await.unwrap();
        for (i, stat) in Stat::TRAINABLE.iter().enumerate() {
            let h = create(
                &pool,
                HabitInput { name: format!("Train {i}"), description: None, primary_stat: Some(*stat), secondary_stat: None },
            )
            .await
            .unwrap();
            log_habit(&pool, h.id, yesterday, CompletionInput { minutes: 240, difficulty: 5 }).await.unwrap();
        }
        let trained = today(&pool, day).await.unwrap();
        for (f, t) in fresh.iter().zip(&trained) {
            assert_eq!(f.id, t.id, "same quests: chosen by date");
            assert!(t.xp > f.xp, "{}: more XP at a higher level", t.id);
            assert!(t.amount >= f.amount, "{}: at least as demanding", t.id);
        }
    }
}
