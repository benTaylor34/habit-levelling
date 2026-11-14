//! Skill levelling. Each completion of a habit gives +1 XP to that habit's
//! skill; levels are derived from XP and never stored.

use crate::models::SkillProgress;

/// XP needed to go from `level` to `level + 1`. Each level costs 10 more
/// than the last: 1→2 = 10 XP, 2→3 = 20 XP, 3→4 = 30 XP, ...
pub fn xp_for_next_level(level: u32) -> u32 {
    level * 10
}

/// Turn a total XP figure into level progress.
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
        assert_eq!(lvl(9), (1, 9));
        assert_eq!(lvl(10), (2, 0));
        assert_eq!(lvl(29), (2, 19));
        assert_eq!(lvl(30), (3, 0));
        assert_eq!(lvl(60), (4, 0));
    }
}
