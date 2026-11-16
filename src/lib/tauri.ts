// Typed wrappers around the Rust commands in src-tauri/src/commands.rs.
// All business logic (streaks, XP, "today") lives in Rust.
import { invoke } from "@tauri-apps/api/core";
import type {
  CompletionInput,
  Habit,
  HabitDetails,
  HabitInput,
  HabitSummary,
  IsoDate,
  LogResult,
  StatProgress,
  XpPreview,
} from "./types";

export const api = {
  getToday: () => invoke<IsoDate>("get_today"),
  listHabits: () => invoke<HabitSummary[]>("list_habits"),
  getHabitDetails: (id: number) => invoke<HabitDetails>("get_habit_details", { id }),
  createHabit: (input: HabitInput) => invoke<Habit>("create_habit", { input }),
  updateHabit: (id: number, input: HabitInput) =>
    invoke<Habit>("update_habit", { id, input }),
  deleteHabit: (id: number) => invoke<void>("delete_habit", { id }),
  completeToday: (id: number, log: CompletionInput) =>
    invoke<LogResult>("complete_habit_today", { id, log }),
  undoToday: (id: number) => invoke<HabitSummary>("undo_habit_today", { id }),
  getStats: () => invoke<StatProgress[]>("get_stats"),
  previewXp: (log: CompletionInput) => invoke<XpPreview>("preview_xp", { ...log }),
};

/** Rust errors arrive as plain strings. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
