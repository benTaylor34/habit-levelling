//! Stat decay: stats you stop training slowly fade.
//!
//! - Days 0–6 without XP in a stat: nothing happens.
//! - From day 7: each day the stat loses 10% of its current level's XP bar,
//!   down to 0 XP into the level. The level itself is safe.
//! - From day 21, once the bar is empty: the stat drops a level, landing at
//!   90% of that level's bar (one day's decay below full), and keeps losing
//!   10% a day, dropping again each time the bar empties. Never below level 1.
//!
//! Like everything else, decay is derived by replaying history day by day,
//! never stored, so undoing or editing a log recalculates it exactly.

use std::collections::BTreeMap;

use chrono::{Days, NaiveDate};
use serde::Serialize;

use crate::stats::{self, xp_for_next_level};

/// Days without XP before a stat starts losing XP.
pub const GRACE_DAYS: u32 = 7;
/// Days without XP before a stat can start losing levels.
pub const LEVEL_LOSS_DAYS: u32 = 21;
/// Share of a level's XP bar lost per day while decaying.
pub const DECAY_PERCENT_PER_DAY: u32 = 10;

/// Where a stat is in the decay cycle, for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DecayPhase {
    /// Trained recently (or nothing left to lose).
    None,
    /// Losing XP within its level.
    Xp,
    /// Losing XP and levels.
    Level,
}

/// XP lost per day at `level`: 10% of that level's bar, at least 1.
pub fn daily_loss(level: u32) -> u32 {
    (xp_for_next_level(level) * DECAY_PERCENT_PER_DAY / 100).max(1)
}

/// Total XP after one day of decay, `days_idle` days after the last gain.
pub fn step(xp: u32, days_idle: u32) -> u32 {
    if days_idle < GRACE_DAYS {
        return xp;
    }
    let (level, into) = stats::level_for_xp(xp);
    if into > 0 {
        xp - into.min(daily_loss(level))
    } else if days_idle >= LEVEL_LOSS_DAYS && level > 1 {
        // Drop into the level below, just short of full.
        xp - daily_loss(level - 1)
    } else {
        xp
    }
}

/// Which decay phase a stat with `xp` is in after `days_idle` idle days.
pub fn phase(xp: u32, days_idle: Option<u32>) -> DecayPhase {
    match days_idle {
        _ if xp == 0 => DecayPhase::None,
        Some(d) if d >= LEVEL_LOSS_DAYS && stats::level_for_xp(xp).0 > 1 => DecayPhase::Level,
        Some(d) if d >= GRACE_DAYS => DecayPhase::Xp,
        _ => DecayPhase::None,
    }
}

/// Replay a stat's XP gains day by day with decay, from its first gain up to
/// `end`. Returns the total XP at the end of each day (gains first, then
/// decay). Empty when the stat has never gained XP.
pub fn simulate(gains: &BTreeMap<NaiveDate, u32>, end: NaiveDate) -> Vec<(NaiveDate, u32)> {
    let Some(&first) = gains.keys().next() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut xp: u32 = 0;
    let mut last_gain = first;
    let mut day = first;
    while day <= end {
        if let Some(&gain) = gains.get(&day) {
            xp += gain;
            last_gain = day;
        }
        let idle = (day - last_gain).num_days() as u32;
        xp = step(xp, idle);
        out.push((day, xp));
        day = match day.checked_add_days(Days::new(1)) {
            Some(next) => next,
            None => break,
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::total_xp_for_level;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    /// (level, xp into level) after `days` days of replay from one gain.
    fn after(start_xp: u32, days: u64) -> (u32, u32) {
        let start = d("2026-01-01");
        let gains = BTreeMap::from([(start, start_xp)]);
        let xp = simulate(&gains, start + Days::new(days)).last().unwrap().1;
        stats::level_for_xp(xp)
    }

    #[test]
    fn nothing_happens_in_the_first_week() {
        let xp = total_xp_for_level(10) + 100;
        assert_eq!(after(xp, 0), (10, 100));
        assert_eq!(after(xp, 6), (10, 100));
    }

    #[test]
    fn xp_drains_from_day_7_but_the_level_is_kept() {
        // The example from the brief: level 10 with 100 XP.
        let xp = total_xp_for_level(10) + 100;
        assert_eq!(daily_loss(10), 331);
        assert_eq!(after(xp, 7), (10, 0), "100 < one day's loss, so empty in a day");
        assert_eq!(after(xp, 20), (10, 0), "level held until three weeks");

        // A fuller bar takes several days: 1,000 into level 10 loses 331/day.
        let xp = total_xp_for_level(10) + 1000;
        assert_eq!(after(xp, 7), (10, 669));
        assert_eq!(after(xp, 8), (10, 338));
        assert_eq!(after(xp, 9), (10, 7));
        assert_eq!(after(xp, 10), (10, 0));
    }

    #[test]
    fn levels_drop_from_day_21_landing_just_short_of_full() {
        let xp = total_xp_for_level(10) + 100;
        let full9 = xp_for_next_level(9); // 2,710
        let loss9 = daily_loss(9); // 271
        assert_eq!(after(xp, 21), (9, full9 - loss9));
        assert_eq!(after(xp, 22), (9, full9 - 2 * loss9));
        // 90% of the bar drains in 9 days; the next day drops another level.
        assert_eq!(after(xp, 30), (9, 0));
        assert_eq!(after(xp, 31), (8, xp_for_next_level(8) - daily_loss(8)));
    }

    #[test]
    fn never_below_level_1_with_0_xp() {
        assert_eq!(after(500, 365), (1, 0));
        assert_eq!(after(0, 30), (1, 0));
    }

    #[test]
    fn a_new_gain_resets_the_clock() {
        let start = d("2026-01-01");
        let xp = total_xp_for_level(5) + 500;
        let gains = BTreeMap::from([(start, xp), (d("2026-01-09"), 10)]);
        let days = simulate(&gains, d("2026-01-15"));
        let by_day: BTreeMap<_, _> = days.into_iter().collect();
        // Day 7 (Jan 8) loses one day's XP, then the Jan 9 gain stops decay.
        let jan8 = xp - daily_loss(5);
        assert_eq!(by_day[&d("2026-01-08")], jan8);
        assert_eq!(by_day[&d("2026-01-09")], jan8 + 10);
        assert_eq!(by_day[&d("2026-01-15")], jan8 + 10, "6 idle days: still in grace");
    }

    #[test]
    fn phases() {
        let xp = total_xp_for_level(4) + 50;
        assert_eq!(phase(xp, Some(6)), DecayPhase::None);
        assert_eq!(phase(xp, Some(7)), DecayPhase::Xp);
        assert_eq!(phase(xp, Some(21)), DecayPhase::Level);
        assert_eq!(phase(0, Some(40)), DecayPhase::None, "nothing left to lose");
        assert_eq!(phase(30, Some(40)), DecayPhase::Xp, "level 1 can only lose XP");
        assert_eq!(phase(xp, None), DecayPhase::None);
    }
}
