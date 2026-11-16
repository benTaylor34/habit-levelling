// Mirrors src-tauri/src/models.rs (serialised camelCase).

/** A local calendar date, "YYYY-MM-DD". */
export type IsoDate = string;

export interface Habit {
  id: number;
  name: string;
  description: string | null;
  skill: string | null;
  createdAt: string;
  archived: boolean;
}

export interface HabitInput {
  name: string;
  description: string | null;
  skill: string | null;
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
  xp: number;
}

export interface HabitSummary extends Habit {
  currentStreak: number;
  longestStreak: number;
  completedToday: boolean;
  totalCompletions: number;
  totalMinutes: number;
  totalXp: number;
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

export interface SkillProgress {
  skill: string;
  xp: number;
  level: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
  /** Consecutive days any habit in this skill was done. */
  currentStreak: number;
  longestStreak: number;
}

/** Display names for difficulty 1–5 (the XP multipliers live in Rust). */
export const DIFFICULTY_LABELS = ["Easy", "Moderate", "Hard", "Very hard", "Brutal"];

export function difficultyLabel(difficulty: number): string {
  return DIFFICULTY_LABELS[difficulty - 1] ?? "—";
}

/** Suggested skills for the form. Users can also type their own. */
export const SUGGESTED_SKILLS = [
  "Fitness",
  "Knowledge",
  "Mental",
  "Creativity",
  "Technical",
  "Social",
  "Discipline",
];
