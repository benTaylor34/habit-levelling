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
  Activity,
  DailyQuest,
  BackupInfo,
  BackupSummary,
  RestoreResult,
  LogResult,
  StatHistory,
  StatRecommendation,
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
  /** Log (or correct) an earlier day; returns the habit's refreshed details. */
  logDay: (id: number, date: IsoDate, log: CompletionInput) =>
    invoke<HabitDetails>("log_habit_day", { id, date, log }),
  undoDay: (id: number, date: IsoDate) => invoke<HabitDetails>("undo_habit_day", { id, date }),
  getStats: () => invoke<StatProgress[]>("get_stats"),
  getActivity: () => invoke<Activity>("get_activity"),
  getDailyQuests: () => invoke<DailyQuest[]>("get_daily_quests"),
  completeQuest: (id: string) => invoke<DailyQuest[]>("complete_quest", { id }),
  undoQuest: (id: string) => invoke<DailyQuest[]>("undo_quest", { id }),
  /** Builds a backup; the page saves it via the system Save dialog. */
  exportBackup: () => invoke<{ fileName: string; json: string }>("export_backup"),
  inspectBackup: (json: string) => invoke<BackupSummary>("inspect_backup", { json }),
  restoreBackup: (json: string) => invoke<RestoreResult>("restore_backup", { json }),
  getBackupInfo: () => invoke<BackupInfo>("get_backup_info"),
  /** `days` = null means all time. */
  getStatHistory: (days: number | null) => invoke<StatHistory>("get_stat_history", { days }),
  /** XP preview for today, or for `date` when filling in an earlier day. */
  previewXp: (id: number, log: CompletionInput, date?: IsoDate) =>
    invoke<XpPreview>("preview_xp", { id, ...log, date: date ?? null }),
  recommendStats: (name: string, description: string | null) =>
    invoke<StatRecommendation>("recommend_stats", { name, description }),
};

/** Rust errors arrive as plain strings. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
