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
