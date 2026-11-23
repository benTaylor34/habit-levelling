//! Habit levels: a slow, steady measure of how much time a habit has had.
//!
//! A habit earns the same base XP as a stat (10 + 1 per minute) but none of
//! the difficulty or streak multipliers, and each habit level costs twice the
//! XP of the same stat level. For a typical habit (daily, Moderate, streak
//! kept) that makes habit levels take roughly 4× as long as its stat's levels:
//! about 3× early on and 5× at level 20, because stats speed up as streaks
//! grow. Only the level is shown; the XP is an internal detail. Derived from
//! the logs, never stored.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::decay::{self, DecayPhase};
use crate::stats::{self, MAX_LEVEL, MAX_MINUTES, MIN_XP_MINUTES};

/// How many times more XP a habit level costs than the same stat level.
pub const COST_VS_STATS: u32 = 2;

/// XP a habit earns for one completed day: 10 + 1 per minute, no multipliers.
/// Like stats, logs under `MIN_XP_MINUTES` earn nothing.
pub fn completion_xp(minutes: u32) -> u32 {
    if minutes < MIN_XP_MINUTES {
        return 0;
    }
    stats::BASE_XP + minutes.min(MAX_MINUTES)
}

/// XP needed to go from `level` to `level + 1`.
pub fn xp_for_next_level(level: u32) -> u32 {
    COST_VS_STATS * stats::xp_for_next_level(level)
}

/// Level for a habit's total XP (level 1 at 0 XP, capped at 100).
pub fn level_for_xp(xp: u32) -> u32 {
    let mut level = 1;
    let mut remaining = xp;
    while level < MAX_LEVEL && remaining >= xp_for_next_level(level) {
        remaining -= xp_for_next_level(level);
        level += 1;
    }
    level
}

/// A habit's level without decay, from the minutes logged on each completed day.
pub fn level(minutes_per_day: impl IntoIterator<Item = u32>) -> u32 {
    level_for_xp(minutes_per_day.into_iter().map(completion_xp).sum())
}

/// A habit's level on `today` and whether it's fading, from (date, minutes)
/// for each completed day. Habits fade like stats (see `decay`), on the habit
/// curve: XP from 1 week without doing it, levels from 3 weeks. Any completed
/// day stops the fade, even one too short to earn XP.
pub fn status(days: impl IntoIterator<Item = (NaiveDate, u32)>, today: NaiveDate) -> (u32, DecayPhase) {
    let gains: BTreeMap<NaiveDate, u32> =
        days.into_iter().map(|(date, minutes)| (date, completion_xp(minutes))).collect();
    let xp = decay::simulate_on(xp_for_next_level, &gains, today)
        .last()
        .map_or(0, |(_, xp)| *xp);
    let idle = gains
        .keys()
        .next_back()
        .map(|last| (today - *last).num_days().max(0) as u32);
    (level_for_xp(xp), decay::phase_on(xp_for_next_level, xp, idle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stat;

    #[test]
    fn xp_is_the_stat_base_without_multipliers() {
        assert_eq!(completion_xp(0), 0, "under 10 minutes earns nothing");
        assert_eq!(completion_xp(9), 0);
        assert_eq!(completion_xp(10), 20);
        assert_eq!(completion_xp(30), 40);
        assert_eq!(completion_xp(100_000), 10 + MAX_MINUTES, "clamped like stats");
    }

    #[test]
    fn each_level_costs_twice_the_stat_level() {
        assert_eq!(xp_for_next_level(1), 20);
        assert_eq!(xp_for_next_level(2), 100);
        assert_eq!(level_for_xp(19), 1);
        assert_eq!(level_for_xp(20), 2);
        assert_eq!(level_for_xp(119), 2);
        assert_eq!(level_for_xp(120), 3);
        assert_eq!(level_for_xp(u32::MAX), 100);
    }

    #[test]
    fn half_an_hour_a_day() {
        let days = |n: usize| level(std::iter::repeat(30).take(n));
        assert_eq!(days(1), 2, "a first completion still levels up");
        assert_eq!(days(14), 5);
        assert_eq!(days(77), 10);
        assert_eq!(days(352), 20, "about a year");
    }

    /// Days of a daily habit (30 min, Moderate, streak kept) to reach `target`.
    fn days_to(target: u32, habit: bool) -> u32 {
        let (mut habit_xp, mut stat_xp) = (0, 0);
        for day in 1.. {
            habit_xp += completion_xp(30);
            stat_xp += stats::completion_xp(30, 2, day);
            let reached = if habit {
                level_for_xp(habit_xp)
            } else {
                stats::progress(Stat::Wisdom, stat_xp).level
            };
            if reached >= target {
                return day;
            }
        }
        unreachable!()
    }

    #[test]
    fn roughly_four_times_slower_than_stats() {
        for (target, low, high) in [(10, 3.5, 5.0), (15, 3.5, 5.5), (20, 4.0, 6.0)] {
            let ratio = f64::from(days_to(target, true)) / f64::from(days_to(target, false));
            assert!((low..=high).contains(&ratio), "level {target}: {ratio:.1}x");
        }
    }

    #[test]
    fn only_minutes_count() {
        // A short log earns nothing; order doesn't matter.
        assert_eq!(level([5, 60]), level_for_xp(70));
        assert_eq!(level([45, 0, 10]), level([10, 45, 0]));
    }

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    /// 30 days of 60 minutes ending on 2026-01-30: habit level 8.
    fn a_month_of_hours() -> Vec<(NaiveDate, u32)> {
        (0..30).map(|i| (d("2026-01-01") + chrono::Days::new(i), 60)).collect()
    }

    #[test]
    fn habits_fade_like_stats_when_not_done() {
        let days = a_month_of_hours();
        let at = |day: &str| status(days.clone(), d(day));
        let fresh = at("2026-01-30").0;
        assert_eq!(fresh, level(days.iter().map(|(_, m)| *m)));
        assert_eq!(at("2026-01-30"), (fresh, DecayPhase::None));
        assert_eq!(at("2026-02-05"), (fresh, DecayPhase::None), "6 days idle: grace");
        assert_eq!(at("2026-02-06"), (fresh, DecayPhase::Xp), "7 days: XP fades, level kept");
        assert_eq!(at("2026-02-19").1, DecayPhase::Xp, "20 days: still only XP");
        assert_eq!(at("2026-02-20").1, DecayPhase::Level, "3 weeks: levels can go");
        assert!(at("2026-03-31").0 < fresh, "a few idle months cost levels");
    }

    #[test]
    fn any_completed_day_stops_the_fade_even_a_short_one() {
        let mut days = a_month_of_hours();
        days.push((d("2026-02-10"), 5)); // 5 minutes: no XP, but it was done
        let (_, phase) = status(days, d("2026-02-12"));
        assert_eq!(phase, DecayPhase::None);
    }

    #[test]
    fn a_never_done_habit_has_nothing_to_lose() {
        assert_eq!(status([], d("2026-01-30")), (1, DecayPhase::None));
    }
}
