//! A year of daily activity for the Analysis page's heatmap. Pure: built from
//! completions, so it's testable without a database.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate};
use serde::Serialize;

use crate::models::Completion;

/// Number of shaded levels above "nothing" (like GitHub's four greens).
pub const LEVELS: u32 = 4;

/// One day on the heatmap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDay {
    pub date: NaiveDate,
    /// Habits logged that day.
    pub completions: u32,
    pub minutes: u32,
    pub xp: u32,
    /// Shade from 0 (nothing logged) to `LEVELS` (the busiest days).
    pub level: u32,
}

/// The heatmap: whole weeks, Monday first, ending with the current week.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    /// Every day from a Monday ~a year ago up to today, oldest first. The
    /// first entry is always a Monday, so day `i` sits in column `i / 7`,
    /// row `i % 7`. Days after today are left out.
    pub days: Vec<ActivityDay>,
    /// Totals over the past 365 days (today included).
    pub total_completions: u32,
    pub active_days: u32,
    /// Most habits logged on a single day.
    pub best_day: u32,
}

/// Shade for a day with `count` completions when the busiest day had `max`.
/// Any activity gets at least level 1; the busiest days get `LEVELS`.
pub fn level(count: u32, max: u32) -> u32 {
    if count == 0 || max == 0 {
        0
    } else {
        ((count * LEVELS).div_ceil(max)).clamp(1, LEVELS)
    }
}

/// Monday of the week containing the day 364 days before `today`, so the grid
/// covers a full year in whole weeks (53 columns at most).
pub fn grid_start(today: NaiveDate) -> NaiveDate {
    let year_ago = today - Days::new(364);
    year_ago - Days::new(u64::from(year_ago.weekday().num_days_from_monday()))
}

pub fn build(completions: &[Completion], today: NaiveDate) -> Activity {
    let start = grid_start(today);
    let mut per_day: BTreeMap<NaiveDate, (u32, u32, u32)> = BTreeMap::new();
    for c in completions.iter().filter(|c| c.date >= start && c.date <= today) {
        let e = per_day.entry(c.date).or_default();
        e.0 += 1;
        e.1 += c.minutes;
        e.2 += c.xp;
    }

    let max = per_day.values().map(|v| v.0).max().unwrap_or(0);
    let year_ago = today - Days::new(364);
    let in_year = per_day.range(year_ago..=today);
    let total_completions = in_year.clone().map(|(_, v)| v.0).sum();
    let active_days = in_year.clone().count() as u32;
    let best_day = in_year.map(|(_, v)| v.0).max().unwrap_or(0);

    let mut days = Vec::new();
    let mut day = start;
    while day <= today {
        let (completions, minutes, xp) = per_day.get(&day).copied().unwrap_or_default();
        days.push(ActivityDay { date: day, completions, minutes, xp, level: level(completions, max) });
        day = match day.checked_add_days(Days::new(1)) {
            Some(next) => next,
            None => break,
        };
    }

    Activity { days, total_completions, active_days, best_day }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn c(date: &str, minutes: u32, xp: u32) -> Completion {
        Completion { date: d(date), minutes, difficulty: 1, streak: 1, xp }
    }

    #[test]
    fn levels_scale_to_the_busiest_day() {
        assert_eq!(level(0, 4), 0);
        assert_eq!(level(1, 4), 1);
        assert_eq!(level(2, 4), 2);
        assert_eq!(level(4, 4), 4);
        assert_eq!(level(1, 10), 1, "any activity is visible");
        assert_eq!(level(10, 10), 4);
        assert_eq!(level(3, 0), 0);
    }

    #[test]
    fn grid_is_whole_weeks_ending_today() {
        let today = d("2026-11-16"); // a Monday
        let a = build(&[], today);
        assert_eq!(a.days[0].date.weekday(), Weekday::Mon);
        assert_eq!(a.days.last().unwrap().date, today);
        assert!(a.days[0].date <= today - Days::new(364));
        assert!(a.days.len() <= 53 * 7);

        let sunday = d("2026-11-22");
        let a = build(&[], sunday);
        assert_eq!(a.days.len() % 7, 0, "ends on a Sunday: full columns");
    }

    #[test]
    fn days_add_up_completions_minutes_and_xp() {
        let today = d("2026-11-16");
        let completions = [
            c("2026-11-16", 30, 60),
            c("2026-11-16", 15, 25),
            c("2026-11-10", 0, 10),
            c("2024-01-01", 99, 99), // too old: ignored
        ];
        let a = build(&completions, today);
        let last = a.days.last().unwrap();
        assert_eq!((last.completions, last.minutes, last.xp, last.level), (2, 45, 85, 4));
        let nov10 = a.days.iter().find(|x| x.date == d("2026-11-10")).unwrap();
        assert_eq!((nov10.completions, nov10.level), (1, 2));
        assert_eq!((a.total_completions, a.active_days, a.best_day), (3, 2, 2));
    }
}
