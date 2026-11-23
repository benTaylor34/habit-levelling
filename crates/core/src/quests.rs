//! Daily quests: three small, at-home challenges a day, each for a different
//! stat, scaled to the player's level.
//!
//! - The day's quests come from a deterministic shuffle seeded by the date, so
//!   reopening the app shows the same three. Nothing about the offer is stored.
//! - Targets scale with the stat's level at the *start* of the day (the caller
//!   passes yesterday's levels), so levelling up mid-day doesn't move them.
//!   They're randomised by about ±40% and capped so that all three stay
//!   doable in one day.
//! - XP is a share of the XP the stat needs for its next level: about 14% at
//!   level 1 falling to about 3% at level 100, scaled ×0.6–1.4 by how
//!   demanding the target rolled. That keeps quests worthwhile at every level.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::stats::{self, Stat, MAX_LEVEL};

/// Quests offered each day, each for a different stat.
pub const QUESTS_PER_DAY: usize = 3;

/// One quest in the catalog. The target at a level is
/// `base + per_level × (level − 1)`, capped at `cap`, then randomised by
/// ±`jitter` and rounded to a multiple of `step`.
#[derive(Debug, Clone, Copy)]
pub struct QuestTemplate {
    pub id: &'static str,
    pub stat: Stat,
    /// Wording with `{n}` for the target, used when the target is 1.
    pub one: &'static str,
    /// Wording with `{n}` for the target, used otherwise.
    pub many: &'static str,
    pub base: f64,
    pub per_level: f64,
    pub cap: f64,
    pub jitter: f64,
    pub step: u32,
}

const fn q(
    id: &'static str,
    stat: Stat,
    one: &'static str,
    many: &'static str,
    (base, per_level, cap): (f64, f64, f64),
    jitter: f64,
    step: u32,
) -> QuestTemplate {
    QuestTemplate { id, stat, one, many, base, per_level, cap, jitter, step }
}

use Stat::*;

/// Every quest, 4 per trainable stat. All doable at home with no equipment,
/// mixing stat training with healthy habits (sleep, water, walks, rest from
/// screens, connection with people).
pub const CATALOG: &[QuestTemplate] = &[
    // Strength
    q("str-pushups", Strength, "Do {n} push-up", "Do {n} push-ups", (5.0, 3.0, 150.0), 0.4, 1),
    q("str-squats", Strength, "Do {n} bodyweight squat", "Do {n} bodyweight squats", (5.0, 4.0, 250.0), 0.4, 5),
    q("str-situps", Strength, "Do {n} situp", "Do {n} situps", (5.0, 3.5, 200.0), 0.4, 5),
    // Dexterity
    q("dex-balance", Dexterity, "Balance on one foot for {n} second on each leg", "Balance on one foot for {n} seconds on each leg", (20.0, 3.0, 180.0), 0.4, 5),
    q("dex-stretch", Dexterity, "Stretch for {n} minute", "Stretch for {n} minutes", (2.0, 0.4, 40.0), 0.4, 1),
    q("dex-shadowbox", Dexterity, "Shadowbox for {n} minute", "Shadowbox for {n} minutes", (1.0, 0.3, 30.0), 0.4, 1),
    q("dex-hands", Dexterity, "Spend {n} minute on a hands-on skill (drawing, an instrument, juggling, a craft)", "Spend {n} minutes on a hands-on skill (drawing, an instrument, juggling, a craft)", (10.0, 0.5, 60.0), 0.4, 5),
    // Constitution
    q("con-wallsit", Constitution, "Hold a wall sit for {n} second in total", "Hold a wall sit for {n} seconds in total", (30.0, 6.0, 600.0), 0.4, 5),
    q("con-walk", Constitution, "Go for a {n}-minute walk", "Go for a {n}-minute walk", (15.0, 1.0, 90.0), 0.4, 5),
    q("con-water", Constitution, "Drink {n} glass of water", "Drink {n} glasses of water", (3.0, 1.0, 8.0), 0.15, 1),
    q("con-sleep", Constitution, "Be in bed, lights out, at least {n} hour before you need to get up", "Be in bed, lights out, at least {n} hours before you need to get up", (6.0, 0.5, 8.0), 0.07, 1),
    // Intelligence
    q("int-read", Intelligence, "Read {n} page of a book", "Read {n} pages of a book", (10.0, 1.0, 100.0), 0.4, 5),
    q("int-words", Intelligence, "Learn {n} new word", "Learn {n} new words", (5.0, 0.5, 40.0), 0.4, 1),
    q("int-learn", Intelligence, "Spend {n} minute learning something new", "Spend {n} minutes learning something new", (10.0, 1.0, 90.0), 0.4, 5),
    q("int-puzzles", Intelligence, "Solve {n} puzzle (sudoku, crossword, chess puzzle…)", "Solve {n} puzzles (sudoku, crossword, chess puzzles…)", (1.0, 0.15, 15.0), 0.4, 1),
    // Wisdom
    q("wis-journal", Wisdom, "Journal for {n} minute", "Journal for {n} minutes", (2.0, 0.4, 40.0), 0.4, 1),
    q("wis-meditate", Wisdom, "Meditate for {n} minute", "Meditate for {n} minutes", (1.0, 1.0, 60.0), 0.4, 1),
    q("wis-gratitude", Wisdom, "Write down {n} thing you're grateful for", "Write down {n} things you're grateful for", (3.0, 0.1, 15.0), 0.4, 1),
    q("wis-screens", Wisdom, "Spend the last {n} minute before bed screen-free", "Spend the last {n} minutes before bed screen-free", (10.0, 1.5, 120.0), 0.4, 5),
    // Charisma
    q("cha-talk", Charisma, "Start a conversation with {n} person you don't usually talk to", "Start a conversation with {n} people you don't usually talk to", (1.0, 0.05, 5.0), 0.3, 1),
    q("cha-compliment", Charisma, "Give {n} genuine compliment", "Give {n} genuine compliments", (1.0, 0.1, 10.0), 0.4, 1),
    q("cha-reach-out", Charisma, "Call or message {n} friend or family member you haven't spoken to lately", "Call or message {n} friends or family members you haven't spoken to lately", (1.0, 0.04, 5.0), 0.3, 1),
    q("cha-speak", Charisma, "Practise speaking aloud for {n} minute (read aloud or rehearse a speech)", "Practise speaking aloud for {n} minutes (read aloud or rehearse a speech)", (1.0, 0.3, 30.0), 0.4, 1),
];

/// A quest offered today.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyQuest {
    /// Catalog id, unique within a day.
    pub id: String,
    pub stat: Stat,
    /// Ready-to-show wording, e.g. "Do 12 push-ups".
    pub text: String,
    /// The target number in the wording.
    pub amount: u32,
    /// XP the stat earns for completing it.
    pub xp: u32,
    /// Filled in by the app from what's been completed today.
    pub completed: bool,
}

/// Small deterministic PRNG (SplitMix64): same seed, same sequence.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn day_seed(date: NaiveDate) -> u64 {
    0x5155_4553_5453_0001 ^ (date.num_days_from_ce() as u64).wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn quest_seed(date: NaiveDate, id: &str) -> u64 {
    id.bytes()
        .fold(day_seed(date), |h, b| (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3))
}

/// The typical target at `level` before randomising (capped).
pub fn typical_amount(t: &QuestTemplate, level: u32) -> f64 {
    let level = level.clamp(1, MAX_LEVEL);
    (t.base + t.per_level * f64::from(level - 1)).min(t.cap)
}

/// Share of the next level's XP a typical quest gives: 14% at level 1,
/// falling steadily to 3% at level 100.
pub fn base_xp_share(level: u32) -> f64 {
    let level = level.clamp(1, MAX_LEVEL);
    0.14 - 0.11 * f64::from(level - 1) / f64::from(MAX_LEVEL - 1)
}

/// XP for completing a quest with target `amount` at `level`.
pub fn quest_xp(t: &QuestTemplate, level: u32, amount: u32) -> u32 {
    let typical = typical_amount(t, level).max(1.0);
    let demand = (f64::from(amount) / typical).clamp(0.6, 1.4);
    let xp = f64::from(stats::xp_for_next_level(level.clamp(1, MAX_LEVEL))) * base_xp_share(level) * demand;
    (xp.round() as u32).max(1)
}

/// The randomised target for `t` on `date` at `level`.
pub fn amount(t: &QuestTemplate, level: u32, date: NaiveDate) -> u32 {
    let mut rng = Rng::new(quest_seed(date, t.id));
    let factor = 1.0 + t.jitter * (2.0 * rng.unit() - 1.0);
    let raw = typical_amount(t, level) * factor;
    let step = f64::from(t.step.max(1));
    ((raw / step).round() * step).max(step) as u32
}

fn text(t: &QuestTemplate, amount: u32) -> String {
    let wording = if amount == 1 { t.one } else { t.many };
    wording.replace("{n}", &amount.to_string())
}

/// Today's quests: three different stats, one quest each, scaled to each
/// stat's level at the start of the day (missing stats count as level 1).
pub fn daily_quests(date: NaiveDate, levels: &BTreeMap<Stat, u32>) -> Vec<DailyQuest> {
    let mut rng = Rng::new(day_seed(date));
    let mut stats = Stat::TRAINABLE.to_vec();
    // Fisher–Yates shuffle, then take the first few stats.
    for i in (1..stats.len()).rev() {
        stats.swap(i, rng.below(i + 1));
    }
    stats
        .into_iter()
        .take(QUESTS_PER_DAY)
        .map(|stat| {
            let options: Vec<&QuestTemplate> = CATALOG.iter().filter(|t| t.stat == stat).collect();
            let t = options[rng.below(options.len())];
            let level = levels.get(&stat).copied().unwrap_or(1);
            let amount = amount(t, level, date);
            DailyQuest {
                id: t.id.to_string(),
                stat,
                text: text(t, amount),
                amount,
                xp: quest_xp(t, level, amount),
                completed: false,
            }
        })
        .collect()
}

/// Look up a catalog quest by id.
pub fn template(id: &str) -> Option<&'static QuestTemplate> {
    CATALOG.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn all_at(level: u32) -> BTreeMap<Stat, u32> {
        Stat::TRAINABLE.iter().map(|&s| (s, level)).collect()
    }

    #[test]
    fn catalog_has_three_to_five_quests_per_stat_with_unique_ids() {
        for stat in Stat::TRAINABLE {
            let n = CATALOG.iter().filter(|t| t.stat == stat).count();
            assert!((3..=5).contains(&n), "{stat:?} has {n}");
        }
        let ids: HashSet<_> = CATALOG.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), CATALOG.len());
        assert!(CATALOG.iter().all(|t| t.one.contains("{n}") && t.many.contains("{n}")));
    }

    #[test]
    fn three_quests_for_three_different_stats() {
        for day in 1..=60 {
            let date = d("2026-11-01") + chrono::Days::new(day);
            let qs = daily_quests(date, &all_at(5));
            assert_eq!(qs.len(), 3);
            let stats: HashSet<_> = qs.iter().map(|q| q.stat).collect();
            assert_eq!(stats.len(), 3, "{date}");
            assert!(qs.iter().all(|q| q.xp >= 1 && !q.text.contains("{n}")));
        }
    }

    #[test]
    fn same_day_same_quests_but_days_differ() {
        let levels = all_at(10);
        assert_eq!(daily_quests(d("2026-11-17"), &levels), daily_quests(d("2026-11-17"), &levels));
        let distinct: HashSet<Vec<String>> = (0..30)
            .map(|i| {
                let date = d("2026-11-01") + chrono::Days::new(i);
                daily_quests(date, &levels).into_iter().map(|q| q.id).collect()
            })
            .collect();
        assert!(distinct.len() > 20, "only {} different sets in 30 days", distinct.len());
    }

    #[test]
    fn targets_scale_with_level_and_stay_within_reason() {
        let pushups = template("str-pushups").unwrap();
        let range = |level| {
            let amounts: Vec<u32> = (0..200)
                .map(|i| amount(pushups, level, d("2026-01-01") + chrono::Days::new(i)))
                .collect();
            (*amounts.iter().min().unwrap(), *amounts.iter().max().unwrap())
        };
        let (lo1, hi1) = range(1);
        let (lo10, hi10) = range(10);
        let (lo100, hi100) = range(100);
        assert!(lo1 >= 3 && hi1 <= 8, "level 1: {lo1}-{hi1}");
        assert!(lo10 >= 18 && hi10 <= 50, "level 10: {lo10}-{hi10}");
        assert!(lo100 >= 90 && hi100 <= 210, "level 100: {lo100}-{hi100}");
        assert!(hi1 < lo10 && hi10 < lo100);
    }

    #[test]
    fn sleep_and_water_stay_healthy() {
        // They grow with Constitution but never past sensible amounts.
        for (id, lowest, highest) in [("con-sleep", 6, 9), ("con-water", 2, 9)] {
            let t = template(id).unwrap();
            for level in [1, 10, 50, 100] {
                for day in 0..30 {
                    let a = amount(t, level, d("2026-11-01") + chrono::Days::new(day));
                    assert!((lowest..=highest).contains(&a), "{id} at {level}: {a}");
                }
            }
        }
    }

    #[test]
    fn xp_is_one_to_twenty_percent_of_the_next_level() {
        for t in CATALOG {
            for level in [1, 5, 20, 50, 99] {
                for day in 0..20 {
                    let a = amount(t, level, d("2026-02-01") + chrono::Days::new(day));
                    let share = f64::from(quest_xp(t, level, a)) / f64::from(stats::xp_for_next_level(level));
                    assert!((0.01..=0.21).contains(&share), "{} L{level}: {share:.3}", t.id);
                }
            }
        }
    }

    #[test]
    fn higher_levels_get_a_smaller_share_but_more_xp() {
        let t = template("int-read").unwrap();
        let typical = |level| quest_xp(t, level, typical_amount(t, level).round() as u32);
        assert!(base_xp_share(1) > base_xp_share(50) && base_xp_share(50) > base_xp_share(100));
        assert!(typical(1) < typical(20) && typical(20) < typical(80), "absolute XP keeps rising");
    }

    #[test]
    fn harder_rolls_give_more_xp() {
        let t = template("str-squats").unwrap();
        assert!(quest_xp(t, 10, 60) > quest_xp(t, 10, 40));
    }

    #[test]
    fn plural_wording() {
        let t = template("cha-talk").unwrap();
        assert_eq!(text(t, 1), "Start a conversation with 1 person you don't usually talk to");
        assert!(text(t, 3).contains("3 people"));
    }
}
