//! Skill levelling. XP is derived from each completion's time spent and
//! difficulty; levels are derived from total XP. Nothing here is stored.

use crate::models::SkillProgress;

/// XP every completion earns before time is counted.
pub const BASE_XP: u32 = 10;
/// Longest time a single day's log may record.
pub const MAX_MINUTES: u32 = 24 * 60;
pub const MIN_DIFFICULTY: u32 = 1;
pub const MAX_DIFFICULTY: u32 = 5;

/// Difficulty multiplier as a percentage: 1 = ×1, 2 = ×1.5, 3 = ×2,
/// 4 = ×2.5, 5 = ×3. Out-of-range values are clamped.
pub fn difficulty_percent(difficulty: u32) -> u32 {
    50 + 50 * difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// XP for one completion: (10 + 1 per minute) × difficulty multiplier.
/// e.g. 30 minutes at difficulty 3 (Hard) = (10 + 30) × 2 = 80 XP.
pub fn completion_xp(minutes: u32, difficulty: u32) -> u32 {
    (BASE_XP + minutes.min(MAX_MINUTES)) * difficulty_percent(difficulty) / 100
}

/// XP needed to go from `level` to `level + 1`. Each level costs 100 more
/// than the last: 1→2 = 100 XP, 2→3 = 200 XP, 3→4 = 300 XP, ...
pub fn xp_for_next_level(level: u32) -> u32 {
    level * 100
}

/// Turn a total XP figure into level progress (streaks start at 0; the
/// caller fills them in from completion dates).
pub fn progress(skill: String, xp: u32) -> SkillProgress {
    let mut level = 1;
    let mut remaining = xp;
    while remaining >= xp_for_next_level(level) {
        remaining -= xp_for_next_level(level);
        level += 1;
    }
    SkillProgress {
        skill,
        xp,
        level,
        xp_into_level: remaining,
        xp_for_next_level: xp_for_next_level(level),
        current_streak: 0,
        longest_streak: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lvl(xp: u32) -> (u32, u32) {
        let p = progress("Fitness".into(), xp);
        (p.level, p.xp_into_level)
    }

    #[test]
    fn levels_follow_increasing_curve() {
        assert_eq!(lvl(0), (1, 0));
        assert_eq!(lvl(99), (1, 99));
        assert_eq!(lvl(100), (2, 0));
        assert_eq!(lvl(299), (2, 199));
        assert_eq!(lvl(300), (3, 0));
        assert_eq!(lvl(600), (4, 0));
    }

    #[test]
    fn more_time_gives_more_xp() {
        assert_eq!(completion_xp(0, 1), 10);
        assert_eq!(completion_xp(30, 1), 40);
        assert_eq!(completion_xp(90, 1), 100);
    }

    #[test]
    fn harder_work_gives_more_xp() {
        let xp: Vec<u32> = (1..=5).map(|d| completion_xp(30, d)).collect();
        assert_eq!(xp, vec![40, 60, 80, 100, 120]);
    }

    #[test]
    fn out_of_range_inputs_are_clamped() {
        assert_eq!(completion_xp(0, 0), completion_xp(0, 1));
        assert_eq!(completion_xp(0, 9), completion_xp(0, 5));
        assert_eq!(completion_xp(10_000, 1), completion_xp(MAX_MINUTES, 1));
    }
}
