//! Stat progress over time, for the Analysis page. Pure: turns a list of XP
//! gains into a day-by-day series per stat, so it's testable without a database.

use std::collections::BTreeMap;

use chrono::{Days, NaiveDate};

use crate::models::{StatHistory, StatSeries};
use crate::stats::{self, Stat};

/// XP a stat gained on a day, from one completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatGain {
    pub date: NaiveDate,
    pub stat: Stat,
    pub xp: u32,
}

/// Level as a decimal, e.g. 3.5 = halfway from level 3 to level 4, so the
/// chart's lines climb smoothly instead of in steps.
fn fractional_level(stat: Stat, xp: u32) -> f64 {
    let p = stats::progress(stat, xp);
    let frac = if p.xp_for_next_level == 0 {
        0.0
    } else {
        f64::from(p.xp_into_level) / f64::from(p.xp_for_next_level)
    };
    ((f64::from(p.level) + frac) * 100.0).round() / 100.0
}

/// Daily cumulative XP and level for each trainable stat from `start` to
/// `end` inclusive. XP earned before `start` counts towards the starting values.
pub fn build(gains: &[StatGain], start: NaiveDate, end: NaiveDate) -> StatHistory {
    let mut dates = Vec::new();
    let mut day = start;
    while day <= end {
        dates.push(day);
        day = match day.checked_add_days(Days::new(1)) {
            Some(next) => next,
            None => break,
        };
    }

    // XP per stat per day within the range, and the total before it.
    let mut before: BTreeMap<Stat, u32> = BTreeMap::new();
    let mut by_day: BTreeMap<(Stat, NaiveDate), u32> = BTreeMap::new();
    for g in gains.iter().filter(|g| g.date <= end) {
        if g.date < start {
            *before.entry(g.stat).or_default() += g.xp;
        } else {
            *by_day.entry((g.stat, g.date)).or_default() += g.xp;
        }
    }

    let series = Stat::TRAINABLE
        .iter()
        .map(|&stat| {
            let start_xp = before.get(&stat).copied().unwrap_or(0);
            let mut total = start_xp;
            let mut xp = Vec::with_capacity(dates.len());
            let mut level = Vec::with_capacity(dates.len());
            for d in &dates {
                total += by_day.get(&(stat, *d)).copied().unwrap_or(0);
                xp.push(total);
                level.push(fractional_level(stat, total));
            }
            StatSeries {
                stat,
                start_level: stats::progress(stat, start_xp).level,
                end_level: stats::progress(stat, total).level,
                xp_gained: total - start_xp,
                xp,
                level,
            }
        })
        .collect();

    StatHistory { dates, series }
}

/// First day to chart: `days` back from `today` (inclusive), or for all time
/// the first day with any XP. Always at least a week so a line has shape.
pub fn range_start(gains: &[StatGain], today: NaiveDate, days: Option<u32>) -> NaiveDate {
    let week_ago = today - Days::new(6);
    match days {
        Some(n) => today - Days::new(u64::from(n.max(1)) - 1),
        None => gains
            .iter()
            .map(|g| g.date)
            .min()
            .map_or(week_ago, |first| first.min(week_ago)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Stat::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn gain(date: &str, stat: Stat, xp: u32) -> StatGain {
        StatGain { date: d(date), stat, xp }
    }

    fn series(h: &StatHistory, stat: Stat) -> &StatSeries {
        h.series.iter().find(|s| s.stat == stat).unwrap()
    }

    #[test]
    fn cumulative_xp_per_day() {
        let gains = [
            gain("2026-11-01", Intelligence, 40),
            gain("2026-11-01", Wisdom, 20),
            gain("2026-11-03", Intelligence, 50),
        ];
        let h = build(&gains, d("2026-11-01"), d("2026-11-04"));
        assert_eq!(h.dates.len(), 4);
        assert_eq!(series(&h, Intelligence).xp, vec![40, 40, 90, 90]);
        assert_eq!(series(&h, Wisdom).xp, vec![20, 20, 20, 20]);
        assert_eq!(series(&h, Strength).xp, vec![0, 0, 0, 0]);
        assert_eq!(h.series.len(), 6, "every trainable stat, never Luck");
    }

    #[test]
    fn xp_before_the_range_is_the_starting_point() {
        let gains = [
            gain("2026-10-01", Strength, 100), // level 2 (70) with 30 into it
            gain("2026-11-02", Strength, 160), // 260 total = exactly level 3
        ];
        let h = build(&gains, d("2026-11-01"), d("2026-11-03"));
        let s = series(&h, Strength);
        assert_eq!(s.xp, vec![100, 260, 260]);
        assert_eq!((s.start_level, s.end_level, s.xp_gained), (2, 3, 160));
        assert_eq!(s.level, vec![2.16, 3.0, 3.0]); // 30/190 into level 2
    }

    #[test]
    fn gains_after_the_range_are_ignored() {
        let gains = [gain("2026-11-10", Charisma, 500)];
        let h = build(&gains, d("2026-11-01"), d("2026-11-05"));
        assert_eq!(series(&h, Charisma).xp_gained, 0);
    }

    #[test]
    fn range_start_options() {
        let today = d("2026-11-16");
        let gains = [gain("2026-03-01", Wisdom, 10)];
        assert_eq!(range_start(&gains, today, Some(30)), d("2026-10-18"));
        assert_eq!(range_start(&gains, today, Some(1)), today);
        assert_eq!(range_start(&gains, today, None), d("2026-03-01"));
        assert_eq!(range_start(&[], today, None), d("2026-11-10"), "empty: one week");
        let recent = [gain("2026-11-15", Wisdom, 10)];
        assert_eq!(range_start(&recent, today, None), d("2026-11-10"), "at least a week");
    }
}
