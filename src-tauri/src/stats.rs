//! D&D-style stats and levelling. XP is derived from each completion's time
//! spent, difficulty and the habit's streak that day; levels are derived from
//! total XP. Nothing here is stored.

use serde::{Deserialize, Serialize};

use crate::models::StatProgress;

/// The six trainable stats, plus the secret Luck stat which no habit can
/// train directly (see `luck.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
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
pub const MIN_DIFFICULTY: u32 = 1;
pub const MAX_DIFFICULTY: u32 = 5;
/// Like Pokémon, levels stop at 100.
pub const MAX_LEVEL: u32 = 100;

/// Difficulty multiplier as a percentage: 1 = ×1, 2 = ×1.5, 3 = ×2,
/// 4 = ×2.5, 5 = ×3. Out-of-range values are clamped.
pub fn difficulty_percent(difficulty: u32) -> u32 {
    50 + 50 * difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// Streak bonus as a percentage: +5% for every day the streak has run
/// beyond the first, capped at +100% (×2) from a 21-day streak onwards.
/// Day 1 = ×1, day 7 = ×1.3, day 14 = ×1.65, day 21+ = ×2.
pub const STREAK_BONUS_PER_DAY: u32 = 5;
pub const MAX_STREAK_BONUS: u32 = 100;

pub fn streak_percent(streak: u32) -> u32 {
    100 + (streak.saturating_sub(1) * STREAK_BONUS_PER_DAY).min(MAX_STREAK_BONUS)
}

/// XP for one completion, which the habit's primary stat gets in full:
///
/// (10 + 1 per minute) × difficulty multiplier × streak multiplier
///
/// `streak` is the habit's streak on the day of the completion (1 = first day).
/// e.g. 30 minutes, Hard (×2), day 7 of a streak (×1.3) = 40 × 2 × 1.3 = 104 XP.
pub fn completion_xp(minutes: u32, difficulty: u32, streak: u32) -> u32 {
    let base = BASE_XP + minutes.min(MAX_MINUTES);
    base * difficulty_percent(difficulty) * streak_percent(streak) / 10_000
}

/// The secondary stat gets half of what the primary stat gets.
pub fn secondary_xp(primary_xp: u32) -> u32 {
    primary_xp / 2
}

/// Total XP needed to *reach* `level`, using Pokémon's "medium fast" cubic
/// curve (n³) scaled by 10 and shifted so level 1 starts at 0 XP.
pub fn total_xp_for_level(level: u32) -> u32 {
    10 * (level.pow(3) - 1)
}

/// XP needed to go from `level` to `level + 1`. Grows quadratically, so low
/// levels come quickly and high levels take real commitment:
/// 1→2 = 70, 2→3 = 190, 5→6 = 910, 10→11 = 3,310, 50→51 = 76,510.
pub fn xp_for_next_level(level: u32) -> u32 {
    total_xp_for_level(level + 1) - total_xp_for_level(level)
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
        assert_eq!(lvl(69), (1, 69));
        assert_eq!(lvl(70), (2, 0));
        assert_eq!(lvl(259), (2, 189));
        assert_eq!(lvl(260), (3, 0));
    }

    #[test]
    fn each_level_costs_more_than_the_last() {
        let costs: Vec<u32> = (1..MAX_LEVEL).map(xp_for_next_level).collect();
        assert_eq!(&costs[..5], &[70, 190, 370, 610, 910]);
        assert!(costs.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn total_xp_matches_cubic_curve() {
        for level in 1..=20 {
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
        assert_eq!(completion_xp(0, 1, 1), 10);
        assert_eq!(completion_xp(30, 1, 1), 40);
        let by_difficulty: Vec<u32> = (1..=5).map(|d| completion_xp(30, d, 1)).collect();
        assert_eq!(by_difficulty, vec![40, 60, 80, 100, 120]);
        assert_eq!(secondary_xp(80), 40);
    }

    #[test]
    fn longer_streaks_give_more_xp_up_to_double() {
        assert_eq!(streak_percent(0), 100);
        assert_eq!(streak_percent(1), 100);
        assert_eq!(streak_percent(7), 130);
        assert_eq!(streak_percent(21), 200);
        assert_eq!(streak_percent(365), 200);
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
        assert_eq!(completion_xp(MAX_MINUTES, 5, 1000), (10 + MAX_MINUTES) * 6);
    }
}
