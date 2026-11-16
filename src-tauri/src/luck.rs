//! The secret Luck stat. No habit trains it directly: instead, whenever a
//! habit's streak hits a milestone there is a small random chance of Luck XP.
//!
//! The rules are pure functions; the dice roll is passed in as a number from
//! 0 to 99 so tests can force a win or a miss.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

/// A streak milestone and what it can win.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Milestone {
    /// Percent chance (out of 100) of winning Luck XP.
    pub chance_percent: u32,
    /// Luck XP awarded on a win.
    pub xp: u32,
}

/// Every 30 days of an unbroken streak.
pub const MONTHLY: Milestone = Milestone { chance_percent: 15, xp: 100 };
/// Every 7 days of an unbroken streak (when it isn't also a monthly one).
pub const WEEKLY: Milestone = Milestone { chance_percent: 3, xp: 25 };

/// Which milestone, if any, a streak of this length has just reached.
pub fn milestone(streak: u32) -> Option<Milestone> {
    match streak {
        0 => None,
        s if s % 30 == 0 => Some(MONTHLY),
        s if s % 7 == 0 => Some(WEEKLY),
        _ => None,
    }
}

/// Luck XP won for a roll of `roll` (0–99) at this milestone.
pub fn award(milestone: Milestone, roll: u32) -> u32 {
    if roll < milestone.chance_percent {
        milestone.xp
    } else {
        0
    }
}

/// A random number from 0 to 99. Uses std's randomly seeded hasher so the
/// app doesn't need a random-number crate; that's plenty for a dice roll.
pub fn roll_d100() -> u32 {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default(),
    );
    (hasher.finish() % 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestones_are_weekly_and_monthly() {
        assert_eq!(milestone(0), None);
        assert_eq!(milestone(6), None);
        assert_eq!(milestone(7), Some(WEEKLY));
        assert_eq!(milestone(14), Some(WEEKLY));
        assert_eq!(milestone(29), None);
        assert_eq!(milestone(30), Some(MONTHLY));
        assert_eq!(milestone(60), Some(MONTHLY));
        // 210 is both 30×7 and 7×30: the bigger monthly prize wins.
        assert_eq!(milestone(210), Some(MONTHLY));
    }

    #[test]
    fn low_rolls_win_high_rolls_miss() {
        assert_eq!(award(MONTHLY, 0), 100);
        assert_eq!(award(MONTHLY, 14), 100);
        assert_eq!(award(MONTHLY, 15), 0);
        assert_eq!(award(WEEKLY, 2), 25);
        assert_eq!(award(WEEKLY, 3), 0);
    }

    #[test]
    fn rolls_are_in_range() {
        assert!((0..1000).map(|_| roll_d100()).all(|r| r < 100));
    }
}
