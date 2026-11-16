// Display-only date helpers. Never use `new Date("YYYY-MM-DD")`: that parses
// as UTC midnight and can show the wrong day. Build local dates instead.
import type { IsoDate } from "./types";

export function parseIsoDate(iso: IsoDate): Date {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function toIsoDate(date: Date): IsoDate {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function formatLong(iso: IsoDate): string {
  return parseIsoDate(iso).toLocaleDateString(undefined, {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
}

/** 95 -> "1h 35m", 30 -> "30 min", 0 -> "" (time not tracked). */
export function formatMinutes(minutes: number): string {
  if (minutes <= 0) return "";
  if (minutes < 60) return `${minutes} min`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m === 0 ? `${h}h` : `${h}h ${m}m`;
}
