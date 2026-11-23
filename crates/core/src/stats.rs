//! D&D-style stats and levelling. XP is derived from each completion's time
//! spent, difficulty and the habit's streak that day; levels are derived from
//! total XP. Nothing here is stored.

use serde::{Deserialize, Serialize};

use crate::models::StatProgress;

/// The six trainable stats, plus the secret Luck stat which no habit can
/// train directly (see `luck.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "lowercase"))]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    Luck,
}

impl Stat {
    /// Stats a habit can be assigned, in the classic D&D order.
    pub const TRAINABLE: [Stat; 6] = [
        Stat::Strength,
        Stat::Dexterity,
        Stat::Constitution,
        Stat::Intelligence,
        Stat::Wisdom,
        Stat::Charisma,
    ];
}

/// XP every completion earns before time is counted.
pub const BASE_XP: u32 = 10;
/// Longest time a single day's log may record.
pub const MAX_MINUTES: u32 = 24 * 60;
/// Shortest log that earns XP. Shorter logs still keep the streak going.
pub const MIN_XP_MINUTES: u32 = 10;
pub const MIN_DIFFICULTY: u32 = 1;
pub const MAX_DIFFICULTY: u32 = 5;
/// Like Pokémon, levels stop at 100.
pub const MAX_LEVEL: u32 = 100;

/// Difficulty multiplier as a percentage: 1 = ×1, 2 = ×1.5, 3 = ×2,
/// 4 = ×2.5, 5 = ×3. Out-of-range values are clamped.
pub fn difficulty_percent(difficulty: u32) -> u32 {
    50 + 50 * difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// Streak bonus as a percentage. It ramps up fast, then keeps creeping up:
///
/// - +5% for every day the streak has run beyond the first, reaching ×2 on day 21.
/// - After that, +1% for every full week past day 21, up to ×2.5 (day 371, about a year).
///
/// Day 1 = ×1, day 7 = ×1.3, day 21 = ×2, day 28 = ×2.01, day 371+ = ×2.5.
pub const STREAK_BONUS_PER_DAY: u32 = 5;
/// Bonus from the daily ramp, reached on `STREAK_RAMP_DAYS`.
pub const STREAK_RAMP_BONUS: u32 = 100;
pub const STREAK_RAMP_DAYS: u32 = 21;
/// Slow growth after the ramp: +1% per full week.
pub const STREAK_BONUS_PER_WEEK: u32 = 1;
pub const MAX_STREAK_BONUS: u32 = 150;

pub fn streak_percent(streak: u32) -> u32 {
    let ramp = (streak.saturating_sub(1) * STREAK_BONUS_PER_DAY).min(STREAK_RAMP_BONUS);
    let weeks = streak.saturating_sub(STREAK_RAMP_DAYS) / 7;
    100 + (ramp + weeks * STREAK_BONUS_PER_WEEK).min(MAX_STREAK_BONUS)
}

/// XP for one completion, which the habit's primary stat gets in full:
///
/// (10 + 1 per minute) × difficulty multiplier × streak multiplier
///
/// `streak` is the habit's streak on the day of the completion (1 = first day).
/// e.g. 30 minutes, Hard (×2), day 7 of a streak (×1.3) = 40 × 2 × 1.3 = 104 XP.
///
/// Logs shorter than `MIN_XP_MINUTES` earn no XP, but still count for the streak.
pub fn completion_xp(minutes: u32, difficulty: u32, streak: u32) -> u32 {
    if minutes < MIN_XP_MINUTES {
        return 0;
    }
    let base = BASE_XP + minutes.min(MAX_MINUTES);
    base * difficulty_percent(difficulty) * streak_percent(streak) / 10_000
}

/// The secondary stat gets half of what the primary stat gets.
pub fn secondary_xp(primary_xp: u32) -> u32 {
    primary_xp / 2
}

/// XP needed to go from `level` to `level + 1`: 10 XP for 1→2, so any first
/// completion levels a stat up, then each level costs 40 more than the one
/// before: 1→2 = 10, 2→3 = 50, 5→6 = 170, 10→11 = 370, 50→51 = 1,970,
/// 99→100 = 3,930.
pub fn xp_for_next_level(level: u32) -> u32 {
    FIRST_LEVEL_XP + LEVEL_XP_STEP * level.saturating_sub(1)
}

/// XP cost of levelling from 1 to 2.
pub const FIRST_LEVEL_XP: u32 = 10;
/// How much more each level costs than the one before.
pub const LEVEL_XP_STEP: u32 = 40;

/// Total XP needed to *reach* `level` (level 1 starts at 0 XP). Level 100
/// takes 195,030 XP: about 11 months of 2 hours a day at Hard on a streak.
/// Only tests need it; the app walks level by level in `level_for_xp`.
#[cfg(test)]
pub fn total_xp_for_level(level: u32) -> u32 {
    let n = level.saturating_sub(1);
    FIRST_LEVEL_XP * n + LEVEL_XP_STEP * n * n.saturating_sub(1) / 2
}

/// Turn a total XP figure into level progress. The XP bar shows
/// `xp_into_level` out of `xp_for_next_level`, so it resets to 0 on each level up.
/// At the max level `xp_for_next_level` is 0. Streaks and decay start empty;
/// the caller fills them in from history.
/// (level, XP earned since reaching that level) for a total XP figure.
pub fn level_for_xp(xp: u32) -> (u32, u32) {
    let mut level = 1;
    let mut remaining = xp;
    while level < MAX_LEVEL && remaining >= xp_for_next_level(level) {
        remaining -= xp_for_next_level(level);
        level += 1;
    }
    (level, remaining)
}

pub fn progress(stat: Stat, xp: u32) -> StatProgress {
    let (level, remaining) = level_for_xp(xp);
    let (xp_into_level, xp_for_next) = if level == MAX_LEVEL {
        (0, 0)
    } else {
        (remaining, xp_for_next_level(level))
    };
    StatProgress {
        stat,
        xp,
        level,
        xp_into_level,
        xp_for_next_level: xp_for_next,
        current_streak: 0,
        longest_streak: 0,
        days_idle: None,
        decay: crate::decay::DecayPhase::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lvl(xp: u32) -> (u32, u32) {
        let p = progress(Stat::Strength, xp);
        (p.level, p.xp_into_level)
    }

    #[test]
    fn xp_resets_each_level() {
        assert_eq!(lvl(0), (1, 0));
        assert_eq!(lvl(9), (1, 9));
        assert_eq!(lvl(10), (2, 0));
        assert_eq!(lvl(59), (2, 49));
        assert_eq!(lvl(60), (3, 0));
    }

    #[test]
    fn each_level_costs_more_than_the_last() {
        let costs: Vec<u32> = (1..MAX_LEVEL).map(xp_for_next_level).collect();
        assert_eq!(&costs[..5], &[10, 50, 90, 130, 170]);
        assert!(costs.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn total_xp_matches_the_sum_of_level_costs() {
        for level in 1..=MAX_LEVEL {
            let sum: u32 = (1..level).map(xp_for_next_level).sum();
            assert_eq!(total_xp_for_level(level), sum);
            let p = progress(Stat::Wisdom, total_xp_for_level(level));
            assert_eq!((p.level, p.xp_into_level), (level, 0));
        }
    }

    #[test]
    fn levels_stop_at_100() {
        let p = progress(Stat::Charisma, u32::MAX);
        assert_eq!((p.level, p.xp_into_level, p.xp_for_next_level), (100, 0, 0));
    }

    #[test]
    fn more_time_and_difficulty_give_more_xp() {
        assert_eq!(completion_xp(10, 1, 1), 20);
        assert_eq!(completion_xp(30, 1, 1), 40);
        let by_difficulty: Vec<u32> = (1..=5).map(|d| completion_xp(30, d, 1)).collect();
        assert_eq!(by_difficulty, vec![40, 60, 80, 100, 120]);
        assert_eq!(secondary_xp(80), 40);
    }

    #[test]
    fn any_first_completion_with_xp_reaches_level_2() {
        let smallest = completion_xp(MIN_XP_MINUTES, MIN_DIFFICULTY, 1);
        assert_eq!(progress(Stat::Strength, smallest).level, 2);
    }

    #[test]
    fn under_ten_minutes_earns_no_xp_even_with_bonuses() {
        assert_eq!(completion_xp(0, MAX_DIFFICULTY, 365), 0);
        assert_eq!(completion_xp(MIN_XP_MINUTES - 1, MAX_DIFFICULTY, 365), 0);
        assert!(completion_xp(MIN_XP_MINUTES, MIN_DIFFICULTY, 1) > 0);
    }

    #[test]
    fn a_year_of_hard_two_hour_days_reaches_level_100() {
        let mut xp = 0;
        let mut day_100 = None;
        for day in 1..=365 {
            xp += completion_xp(120, 3, day);
            if day_100.is_none() && progress(Stat::Strength, xp).level == MAX_LEVEL {
                day_100 = Some(day);
            }
        }
        let day_100 = day_100.expect("level 100 within a year");
        assert!((300..=365).contains(&day_100), "reached on day {day_100}");
    }

    #[test]
    fn longer_streaks_give_more_xp_fast_then_slowly() {
        assert_eq!(streak_percent(0), 100);
        assert_eq!(streak_percent(1), 100);
        assert_eq!(streak_percent(7), 130);
        assert_eq!(streak_percent(21), 200);
        assert_eq!(streak_percent(27), 200);
        assert_eq!(streak_percent(28), 201);
        assert_eq!(streak_percent(365), 249);
        assert_eq!(streak_percent(371), 250);
        assert_eq!(streak_percent(5000), 250);
        assert_eq!(completion_xp(30, 3, 1), 80);
        assert_eq!(completion_xp(30, 3, 7), 104);
        assert_eq!(completion_xp(30, 3, 30), 160);
    }

    #[test]
    fn out_of_range_inputs_are_clamped() {
        assert_eq!(completion_xp(0, 0, 1), completion_xp(0, 1, 1));
        assert_eq!(completion_xp(0, 9, 1), completion_xp(0, 5, 1));
        assert_eq!(completion_xp(10_000, 1, 1), completion_xp(MAX_MINUTES, 1, 1));
        // Largest possible XP doesn't overflow.
        assert_eq!(completion_xp(MAX_MINUTES, 5, 1000), (10 + MAX_MINUTES) * 3 * 250 / 100);
    }
}
