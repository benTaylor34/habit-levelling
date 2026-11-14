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

export interface HabitSummary extends Habit {
  currentStreak: number;
  longestStreak: number;
  completedToday: boolean;
  totalCompletions: number;
  /** Last 7 days, oldest first; the last entry is today. */
  recentDays: boolean[];
}

export interface HabitDetails {
  summary: HabitSummary;
  completions: IsoDate[];
}

export interface SkillProgress {
  skill: string;
  xp: number;
  level: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
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
