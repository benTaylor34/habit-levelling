//! Suggests a primary and secondary stat for a habit from its name and
//! description, using a keyword table. Pure and offline: no AI, no network.
//!
//! Each keyword names the stat it mainly trains and, optionally, a stat it
//! trains on the side. Matches score points for both; words in the name count
//! double because a name is a stronger signal than a description. The two
//! highest-scoring stats become the recommendation.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::stats::Stat::{self, *};

/// (keyword, main stat, side stat). Keywords are matched against the start of
/// each word ("run" matches "running"); keywords of 5+ letters also match
/// inside words ("lifting" matches "weightlifting").
const KEYWORDS: &[(&str, Stat, Option<Stat>)] = &[
    // Strength
    ("lift", Strength, Some(Constitution)),
    ("weight", Strength, Some(Constitution)),
    ("gym", Strength, Some(Constitution)),
    ("squat", Strength, Some(Constitution)),
    ("deadlift", Strength, Some(Constitution)),
    ("bench", Strength, Some(Constitution)),
    ("pushup", Strength, Some(Constitution)),
    ("push-up", Strength, Some(Constitution)),
    ("pullup", Strength, Some(Dexterity)),
    ("pull-up", Strength, Some(Dexterity)),
    ("chinup", Strength, Some(Dexterity)),
    ("strength", Strength, Some(Constitution)),
    ("muscle", Strength, Some(Constitution)),
    ("kettlebell", Strength, Some(Constitution)),
    ("crossfit", Strength, Some(Constitution)),
    ("calisthenic", Strength, Some(Dexterity)),
    ("plank", Strength, Some(Constitution)),
    ("climb", Strength, Some(Dexterity)),
    ("boulder", Strength, Some(Dexterity)),
    ("rowing", Strength, Some(Constitution)),
    ("workout", Strength, Some(Constitution)),
    ("exercise", Strength, Some(Constitution)),
    // Dexterity
    ("run", Dexterity, Some(Constitution)),
    ("jog", Dexterity, Some(Constitution)),
    ("sprint", Dexterity, Some(Constitution)),
    ("yoga", Dexterity, Some(Wisdom)),
    ("stretch", Dexterity, Some(Constitution)),
    ("pilates", Dexterity, Some(Constitution)),
    ("dance", Dexterity, Some(Charisma)),
    ("guitar", Dexterity, Some(Charisma)),
    ("piano", Dexterity, Some(Charisma)),
    ("drum", Dexterity, Some(Charisma)),
    ("violin", Dexterity, Some(Charisma)),
    ("instrument", Dexterity, Some(Charisma)),
    ("juggl", Dexterity, Some(Wisdom)),
    ("skate", Dexterity, Some(Constitution)),
    ("surf", Dexterity, Some(Constitution)),
    ("martial", Dexterity, Some(Strength)),
    ("boxing", Dexterity, Some(Strength)),
    ("karate", Dexterity, Some(Strength)),
    ("judo", Dexterity, Some(Strength)),
    ("jiu", Dexterity, Some(Strength)),
    ("tennis", Dexterity, Some(Constitution)),
    ("football", Dexterity, Some(Constitution)),
    ("soccer", Dexterity, Some(Constitution)),
    ("basketball", Dexterity, Some(Constitution)),
    ("badminton", Dexterity, Some(Constitution)),
    ("sport", Dexterity, Some(Constitution)),
    ("typing", Dexterity, Some(Intelligence)),
    ("knit", Dexterity, Some(Wisdom)),
    ("sew", Dexterity, Some(Charisma)),
    ("craft", Dexterity, Some(Intelligence)),
    ("woodwork", Dexterity, Some(Strength)),
    // Constitution
    ("swim", Constitution, Some(Dexterity)),
    ("cycl", Constitution, Some(Dexterity)),
    ("bike", Constitution, Some(Dexterity)),
    ("walk", Constitution, Some(Wisdom)),
    ("steps", Constitution, Some(Dexterity)),
    ("hike", Constitution, Some(Strength)),
    ("hiking", Constitution, Some(Strength)),
    ("cardio", Constitution, Some(Dexterity)),
    ("marathon", Constitution, Some(Dexterity)),
    ("sleep", Constitution, Some(Wisdom)),
    ("bed", Constitution, Some(Wisdom)),
    ("water", Constitution, Some(Wisdom)),
    ("hydrat", Constitution, Some(Wisdom)),
    ("diet", Constitution, Some(Wisdom)),
    ("eat", Constitution, Some(Wisdom)),
    ("nutrition", Constitution, Some(Intelligence)),
    ("vegetable", Constitution, Some(Wisdom)),
    ("veggie", Constitution, Some(Wisdom)),
    ("fruit", Constitution, Some(Wisdom)),
    ("protein", Constitution, Some(Strength)),
    ("fasting", Constitution, Some(Wisdom)),
    ("cold", Constitution, Some(Wisdom)),
    ("sauna", Constitution, Some(Wisdom)),
    ("vitamin", Constitution, Some(Wisdom)),
    ("supplement", Constitution, Some(Wisdom)),
    ("alcohol", Constitution, Some(Wisdom)),
    ("sober", Constitution, Some(Wisdom)),
    ("smok", Constitution, Some(Wisdom)),
    ("sugar", Constitution, Some(Wisdom)),
    ("health", Constitution, Some(Wisdom)),
    ("endurance", Constitution, Some(Strength)),
    // Intelligence
    ("study", Intelligence, Some(Wisdom)),
    ("studi", Intelligence, Some(Wisdom)),
    ("read", Intelligence, Some(Wisdom)),
    ("book", Intelligence, Some(Wisdom)),
    ("learn", Intelligence, Some(Wisdom)),
    ("code", Intelligence, Some(Wisdom)),
    ("coding", Intelligence, Some(Wisdom)),
    ("program", Intelligence, Some(Wisdom)),
    ("leetcode", Intelligence, Some(Wisdom)),
    ("rust", Intelligence, Some(Wisdom)),
    ("python", Intelligence, Some(Wisdom)),
    ("math", Intelligence, Some(Wisdom)),
    ("science", Intelligence, Some(Wisdom)),
    ("research", Intelligence, Some(Wisdom)),
    ("homework", Intelligence, Some(Wisdom)),
    ("revis", Intelligence, Some(Wisdom)),
    ("exam", Intelligence, Some(Wisdom)),
    ("lecture", Intelligence, Some(Wisdom)),
    ("course", Intelligence, Some(Wisdom)),
    ("flashcard", Intelligence, Some(Wisdom)),
    ("anki", Intelligence, Some(Wisdom)),
    ("chess", Intelligence, Some(Wisdom)),
    ("puzzle", Intelligence, Some(Wisdom)),
    ("sudoku", Intelligence, Some(Wisdom)),
    ("crossword", Intelligence, Some(Wisdom)),
    ("history", Intelligence, Some(Wisdom)),
    ("documentar", Intelligence, Some(Wisdom)),
    ("language", Intelligence, Some(Charisma)),
    ("duolingo", Intelligence, Some(Charisma)),
    ("vocab", Intelligence, Some(Charisma)),
    ("spanish", Intelligence, Some(Charisma)),
    ("french", Intelligence, Some(Charisma)),
    ("german", Intelligence, Some(Charisma)),
    ("japanese", Intelligence, Some(Charisma)),
    ("mandarin", Intelligence, Some(Charisma)),
    ("write", Intelligence, Some(Charisma)),
    ("writing", Intelligence, Some(Charisma)),
    ("blog", Intelligence, Some(Charisma)),
    ("essay", Intelligence, Some(Charisma)),
    // Wisdom
    ("meditat", Wisdom, Some(Constitution)),
    ("mindful", Wisdom, Some(Constitution)),
    ("breath", Wisdom, Some(Constitution)),
    ("journal", Wisdom, Some(Intelligence)),
    ("diary", Wisdom, Some(Intelligence)),
    ("reflect", Wisdom, Some(Intelligence)),
    ("gratitude", Wisdom, Some(Charisma)),
    ("grateful", Wisdom, Some(Charisma)),
    ("pray", Wisdom, Some(Charisma)),
    ("spiritual", Wisdom, Some(Charisma)),
    ("bible", Wisdom, Some(Intelligence)),
    ("philosoph", Wisdom, Some(Intelligence)),
    ("stoic", Wisdom, Some(Intelligence)),
    ("therapy", Wisdom, Some(Charisma)),
    ("planning", Wisdom, Some(Intelligence)),
    ("planner", Wisdom, Some(Intelligence)),
    ("budget", Wisdom, Some(Intelligence)),
    ("finance", Wisdom, Some(Intelligence)),
    ("saving", Wisdom, Some(Intelligence)),
    ("invest", Wisdom, Some(Intelligence)),
    ("screen", Wisdom, Some(Constitution)),
    ("phone", Wisdom, Some(Constitution)),
    ("social media", Wisdom, Some(Constitution)),
    ("tidy", Wisdom, Some(Constitution)),
    ("clean", Wisdom, Some(Constitution)),
    ("declutter", Wisdom, Some(Constitution)),
    ("nature", Wisdom, Some(Constitution)),
    ("garden", Wisdom, Some(Constitution)),
    ("podcast", Wisdom, Some(Intelligence)),
    ("affirmation", Wisdom, Some(Charisma)),
    ("routine", Wisdom, Some(Constitution)),
    // Charisma
    ("friend", Charisma, Some(Wisdom)),
    ("call", Charisma, Some(Wisdom)),
    ("social", Charisma, Some(Wisdom)),
    ("network", Charisma, Some(Intelligence)),
    ("speak", Charisma, Some(Intelligence)),
    ("speech", Charisma, Some(Intelligence)),
    ("public", Charisma, Some(Intelligence)),
    ("present", Charisma, Some(Intelligence)),
    ("talk", Charisma, Some(Wisdom)),
    ("conversation", Charisma, Some(Wisdom)),
    ("family", Charisma, Some(Wisdom)),
    ("parent", Charisma, Some(Wisdom)),
    ("compliment", Charisma, Some(Wisdom)),
    ("volunteer", Charisma, Some(Wisdom)),
    ("meet", Charisma, Some(Wisdom)),
    ("date", Charisma, Some(Wisdom)),
    ("sing", Charisma, Some(Dexterity)),
    ("acting", Charisma, Some(Dexterity)),
    ("theat", Charisma, Some(Dexterity)),
    ("improv", Charisma, Some(Wisdom)),
    ("perform", Charisma, Some(Dexterity)),
    ("draw", Charisma, Some(Dexterity)),
    ("sketch", Charisma, Some(Dexterity)),
    ("paint", Charisma, Some(Dexterity)),
    ("photo", Charisma, Some(Dexterity)),
    ("music", Charisma, Some(Dexterity)),
    ("creative", Charisma, Some(Intelligence)),
    ("content", Charisma, Some(Intelligence)),
    ("stream", Charisma, Some(Intelligence)),
    ("video", Charisma, Some(Intelligence)),
    ("groom", Charisma, Some(Constitution)),
    ("skincare", Charisma, Some(Constitution)),
    ("style", Charisma, Some(Wisdom)),
    ("confiden", Charisma, Some(Wisdom)),
];

/// Points for a keyword match: [main stat, side stat], by where it matched.
const NAME_POINTS: [u32; 2] = [6, 2];
const DESCRIPTION_POINTS: [u32; 2] = [3, 1];

/// Suggested stats for a habit, plus the keywords that led to them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRecommendation {
    pub primary: Option<Stat>,
    pub secondary: Option<Stat>,
    /// Matched keywords, strongest first, for "suggested because…" text.
    pub matched: Vec<String>,
}

/// Lowercase words, keeping hyphens inside words ("push-ups").
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

fn matches(keyword: &str, text: &str, words: &[String]) -> bool {
    if keyword.contains(' ') {
        return text.contains(keyword); // multi-word phrase, e.g. "social media"
    }
    words
        .iter()
        .any(|w| w.starts_with(keyword) || (keyword.len() >= 5 && w.contains(keyword)))
}

pub fn recommend(name: &str, description: Option<&str>) -> StatRecommendation {
    let mut scores: BTreeMap<Stat, u32> = BTreeMap::new();
    let mut matched: Vec<(u32, &str)> = Vec::new();

    for (text, [main_pts, side_pts]) in [
        (name, NAME_POINTS),
        (description.unwrap_or(""), DESCRIPTION_POINTS),
    ] {
        let lower = text.to_lowercase();
        let ws = words(text);
        for &(keyword, main, side) in KEYWORDS {
            if matches(keyword, &lower, &ws) {
                *scores.entry(main).or_default() += main_pts;
                if let Some(side) = side {
                    *scores.entry(side).or_default() += side_pts;
                }
                match matched.iter_mut().find(|(_, k)| *k == keyword) {
                    Some((pts, _)) => *pts += main_pts,
                    None => matched.push((main_pts, keyword)),
                }
            }
        }
    }

    // Highest score first; ties go to the earlier stat in D&D order.
    let mut ranked: Vec<(Stat, u32)> = scores.into_iter().filter(|(_, s)| *s > 0).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    matched.sort_by(|a, b| b.0.cmp(&a.0));

    StatRecommendation {
        primary: ranked.first().map(|r| r.0),
        secondary: ranked.get(1).map(|r| r.0),
        matched: matched.into_iter().map(|(_, k)| k.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(name: &str, description: Option<&str>) -> (Option<Stat>, Option<Stat>) {
        let r = recommend(name, description);
        (r.primary, r.secondary)
    }

    #[test]
    fn examples_from_the_brief() {
        assert_eq!(rec("Studying", None), (Some(Intelligence), Some(Wisdom)));
        assert_eq!(rec("Weightlifting", None), (Some(Strength), Some(Constitution)));
    }

    #[test]
    fn common_habits() {
        assert_eq!(rec("Morning run", None), (Some(Dexterity), Some(Constitution)));
        assert_eq!(rec("Meditate 10 minutes", None), (Some(Wisdom), Some(Constitution)));
        assert_eq!(rec("Read 20 pages", None), (Some(Intelligence), Some(Wisdom)));
        assert_eq!(rec("Call a friend", None), (Some(Charisma), Some(Wisdom)));
        assert_eq!(rec("Swim laps", None), (Some(Constitution), Some(Dexterity)));
        assert_eq!(rec("Practice guitar", None), (Some(Dexterity), Some(Charisma)));
        assert_eq!(rec("Journal before bed", None).0, Some(Wisdom));
        assert_eq!(rec("100 push-ups", None), (Some(Strength), Some(Constitution)));
    }

    #[test]
    fn description_helps_when_the_name_is_vague() {
        assert_eq!(rec("Morning routine", None).0, Some(Wisdom));
        assert_eq!(
            rec("Daily practice", Some("Spanish vocabulary on Duolingo")),
            (Some(Intelligence), Some(Charisma))
        );
    }

    #[test]
    fn name_outweighs_description() {
        // Name says running; the description mentions a podcast while running.
        assert_eq!(
            rec("Run 5k", Some("listen to a podcast")).0,
            Some(Dexterity)
        );
    }

    #[test]
    fn unknown_habits_get_no_suggestion() {
        let r = recommend("Water the plants", None);
        assert_eq!(r.primary, Some(Constitution), "'water' is still a keyword");
        let r = recommend("Xyzzy", None);
        assert_eq!((r.primary, r.secondary), (None, None));
        assert!(r.matched.is_empty());
        assert_eq!(rec("", None), (None, None));
    }

    #[test]
    fn short_keywords_only_match_word_starts() {
        // "read" must not match inside "already", nor "eat" inside "great".
        assert_eq!(rec("Already great", None), (None, None));
    }

    #[test]
    fn matched_keywords_are_reported() {
        let r = recommend("Study maths", Some("then read a book"));
        assert_eq!(r.matched[..2], ["study".to_string(), "math".to_string()]);
        assert!(r.matched.contains(&"book".to_string()));
    }

    #[test]
    fn primary_and_secondary_always_differ() {
        for (name, _, _) in KEYWORDS {
            let r = recommend(name, None);
            assert!(r.primary.is_some(), "{name} should recommend something");
            assert_ne!(r.primary, r.secondary, "{name}");
        }
    }
}
