//! Streak calculation. Pure functions over completion dates, so they are
//! trivially testable and never depend on stored (possibly stale) state.

use chrono::{Days, NaiveDate};
use std::collections::BTreeSet;

/// Number of consecutive completed days ending today.
///
/// If today is not completed yet the streak is still "alive" and counts back
/// from yesterday: the day isn't over, so the streak isn't broken yet.
/// Duplicate and future dates are ignored.
pub fn current_streak(dates: &[NaiveDate], today: NaiveDate) -> u32 {
    let set: BTreeSet<NaiveDate> = dates.iter().copied().filter(|d| *d <= today).collect();

    let mut day = if set.contains(&today) {
        today
    } else {
        match today.pred_opt() {
            Some(yesterday) if set.contains(&yesterday) => yesterday,
            _ => return 0,
        }
    };

    let mut streak = 0;
    while set.contains(&day) {
        streak += 1;
        match day.pred_opt() {
            Some(prev) => day = prev,
            None => break,
        }
    }
    streak
}

/// Longest run of consecutive completed days anywhere in the history.
pub fn longest_streak(dates: &[NaiveDate]) -> u32 {
    let set: BTreeSet<NaiveDate> = dates.iter().copied().collect();

    let mut longest = 0;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for &day in &set {
        run = match prev {
            Some(p) if p.checked_add_days(Days::new(1)) == Some(day) => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        prev = Some(day);
    }
    longest
}

/// Completion flags for the `n` days ending at `today` (oldest first).
pub fn recent_days(dates: &[NaiveDate], today: NaiveDate, n: u64) -> Vec<bool> {
    let set: BTreeSet<NaiveDate> = dates.iter().copied().collect();
    (0..n)
        .rev()
        .map(|back| {
            today
                .checked_sub_days(Days::new(back))
                .is_some_and(|d| set.contains(&d))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn days(list: &[&str]) -> Vec<NaiveDate> {
        list.iter().map(|s| d(s)).collect()
    }

    #[test]
    fn empty_history_has_no_streak() {
        assert_eq!(current_streak(&[], d("2026-11-14")), 0);
        assert_eq!(longest_streak(&[]), 0);
    }

    #[test]
    fn five_consecutive_days_is_five() {
        let h = days(&["2026-11-10", "2026-11-11", "2026-11-12", "2026-11-13", "2026-11-14"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 5);
        assert_eq!(longest_streak(&h), 5);
    }

    #[test]
    fn gap_resets_current_streak() {
        // Nov 12 missed -> current streak is 2 (13th, 14th)
        let h = days(&["2026-11-10", "2026-11-11", "2026-11-13", "2026-11-14"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 2);
        assert_eq!(longest_streak(&h), 2);
    }

    #[test]
    fn today_not_done_yet_keeps_streak_from_yesterday() {
        let h = days(&["2026-11-11", "2026-11-12", "2026-11-13"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 3);
    }

    #[test]
    fn missing_yesterday_and_today_breaks_streak() {
        let h = days(&["2026-11-10", "2026-11-11", "2026-11-12"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 0);
        assert_eq!(longest_streak(&h), 3);
    }

    #[test]
    fn duplicates_count_once() {
        let h = days(&["2026-11-14", "2026-11-14", "2026-11-14"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 1);
        assert_eq!(longest_streak(&h), 1);
    }

    #[test]
    fn unsorted_input_is_fine() {
        let h = days(&["2026-11-14", "2026-11-12", "2026-11-13"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 3);
    }

    #[test]
    fn future_dates_do_not_count_toward_current() {
        let h = days(&["2026-11-14", "2026-11-15", "2026-11-16"]);
        assert_eq!(current_streak(&h, d("2026-11-14")), 1);
    }

    #[test]
    fn longest_streak_finds_earlier_run() {
        let h = days(&[
            "2026-10-01", "2026-10-02", "2026-10-03", "2026-10-04", // 4
            "2026-11-13", "2026-11-14",                             // 2
        ]);
        assert_eq!(longest_streak(&h), 4);
        assert_eq!(current_streak(&h, d("2026-11-14")), 2);
    }

    #[test]
    fn streak_crosses_month_and_year_boundaries() {
        let h = days(&["2025-12-30", "2025-12-31", "2026-01-01", "2026-01-02"]);
        assert_eq!(longest_streak(&h), 4);
        assert_eq!(current_streak(&h, d("2026-01-02")), 4);
    }

    #[test]
    fn recent_days_oldest_first() {
        let h = days(&["2026-11-12", "2026-11-14"]);
        // Nov 11, 12, 13, 14
        assert_eq!(recent_days(&h, d("2026-11-14"), 4), vec![false, true, false, true]);
    }
}
