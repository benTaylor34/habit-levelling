// Mirrors src-tauri/src/models.rs (serialised camelCase).

/** A local calendar date, "YYYY-MM-DD". */
export type IsoDate = string;

/** The six trainable stats, plus the secret Luck stat. */
export type StatKey =
  | "strength"
  | "dexterity"
  | "constitution"
  | "intelligence"
  | "wisdom"
  | "charisma"
  | "luck";

export interface Habit {
  id: number;
  name: string;
  description: string | null;
  /** Gets the full XP of each completion. */
  primaryStat: StatKey | null;
  /** Gets half the XP of each completion. */
  secondaryStat: StatKey | null;
  createdAt: string;
  archived: boolean;
}

export interface HabitInput {
  name: string;
  description: string | null;
  primaryStat: StatKey | null;
  secondaryStat: StatKey | null;
}

/** What the user enters when logging a day. */
export interface CompletionInput {
  minutes: number;
  /** 1 (easy) to 5 (brutal). */
  difficulty: number;
}

/** One logged day, with the XP Rust calculated for it. */
export interface Completion {
  date: IsoDate;
  minutes: number;
  difficulty: number;
  /** The habit's streak on this day (1 = first day). */
  streak: number;
  /** XP earned by the primary stat, including the streak bonus. */
  xp: number;
}

export interface HabitSummary extends Habit {
  currentStreak: number;
  longestStreak: number;
  completedToday: boolean;
  totalCompletions: number;
  totalMinutes: number;
  totalXp: number;
  /** Days since last done (or since created, if never done). */
  idleDays: number;
  /** Not done for a week or more: its stats are decaying. */
  needsAttention: boolean;
  /** Today's log, if completed today. */
  todayLog: Completion | null;
  /** Last 7 days, oldest first; the last entry is today. */
  recentDays: boolean[];
}

export interface HabitDetails {
  summary: HabitSummary;
  /** Every logged day, oldest first. */
  completions: Completion[];
}

export interface StatProgress {
  stat: StatKey;
  /** Total XP ever earned. */
  xp: number;
  level: number;
  /** XP since the last level up (the bar resets each level). */
  xpIntoLevel: number;
  /** XP needed for the next level; 0 at max level. */
  xpForNextLevel: number;
  /** Consecutive days any habit training this stat was done. */
  currentStreak: number;
  longestStreak: number;
  /** Days since this stat last gained XP; null if it never has. */
  daysIdle: number | null;
  /** "xp" = losing XP within its level, "level" = losing levels too. */
  decay: DecayPhase;
}

export type DecayPhase = "none" | "xp" | "level";

/** Mirrors decay.rs: days without XP before XP, then levels, start fading. */
export const DECAY_GRACE_DAYS = 7;

export interface LuckWin {
  xp: number;
  streak: number;
}

export interface LogResult {
  summary: HabitSummary;
  /** Set only when a streak milestone's Luck roll won. */
  luck: LuckWin | null;
}

export interface XpPreview {
  primary: number;
  secondary: number;
  /** The streak this log would be part of. */
  streak: number;
  /** Streak multiplier as a percentage (100 = no bonus). */
  streakPercent: number;
}

/** Stats suggested for a habit from its name and description. */
export interface StatRecommendation {
  primary: StatKey | null;
  secondary: StatKey | null;
  /** Keywords that led to the suggestion, strongest first. */
  matched: string[];
}

/** One stat's day-by-day progress, lined up with `StatHistory.dates`. */
export interface StatSeries {
  stat: StatKey;
  /** Cumulative XP at the end of each day. */
  xp: number[];
  /** Level as a decimal at the end of each day (3.5 = halfway to level 4). */
  level: number[];
  startLevel: number;
  endLevel: number;
  /** XP earned within the range (before decay). */
  xpGained: number;
  /** XP lost to decay within the range. */
  xpDecayed: number;
}

export interface StatHistory {
  dates: IsoDate[];
  series: StatSeries[];
}

/** One day on the activity heatmap. */
export interface ActivityDay {
  date: IsoDate;
  completions: number;
  minutes: number;
  xp: number;
  /** Shade from 0 (nothing logged) to 4 (busiest days). */
  level: number;
}

/** A year of activity in whole weeks; `days[0]` is always a Monday. */
export interface Activity {
  days: ActivityDay[];
  totalCompletions: number;
  activeDays: number;
  bestDay: number;
}

export interface StatInfo {
  key: StatKey;
  name: string;
  abbr: string;
  description: string;
}

/** Display copy for each stat, D&D style. Trainable stats first, in order. */
export const STATS: StatInfo[] = [
  {
    key: "strength",
    name: "Strength",
    abbr: "STR",
    description: "Raw physical power. Lifting, carrying and pushing your limits. Trained by weightlifting, climbing and hard physical work.",
  },
  {
    key: "dexterity",
    name: "Dexterity",
    abbr: "DEX",
    description: "Agility, reflexes and balance. Quick feet and steady hands. Trained by running, sports, dance, instruments and crafts.",
  },
  {
    key: "constitution",
    name: "Constitution",
    abbr: "CON",
    description: "Health, stamina and endurance. The grit to keep going. Trained by cardio, good sleep, nutrition and cold showers.",
  },
  {
    key: "intelligence",
    name: "Intelligence",
    abbr: "INT",
    description: "Reasoning, memory and learning. The power of the mind. Trained by studying, reading, coding and learning languages.",
  },
  {
    key: "wisdom",
    name: "Wisdom",
    abbr: "WIS",
    description: "Insight, perception and self-awareness. Knowing what matters. Trained by meditation, journaling and reflection.",
  },
  {
    key: "charisma",
    name: "Charisma",
    abbr: "CHA",
    description: "Confidence and force of personality. Trained by socialising, public speaking, performing and creative expression.",
  },
];

export const LUCK: StatInfo = {
  key: "luck",
  name: "Luck",
  abbr: "LCK",
  description: "Fortune favours the consistent. Luck can't be trained; it may strike when a streak reaches a milestone.",
};

export function statInfo(key: StatKey): StatInfo {
  return key === "luck" ? LUCK : STATS.find((s) => s.key === key)!;
}

/** Display names for difficulty 1–5 (the XP multipliers live in Rust). */
export const DIFFICULTY_LABELS = ["Easy", "Moderate", "Hard", "Very hard", "Brutal"];

export function difficultyLabel(difficulty: number): string {
  return DIFFICULTY_LABELS[difficulty - 1] ?? "—";
}
