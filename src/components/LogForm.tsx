import { useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import { useBackHandler } from "../platform";
import type { Completion, CompletionInput, HabitSummary, XpPreview } from "../lib/types";
import { DIFFICULTY_LABELS, statInfo } from "../lib/types";
import { formatMinutes } from "../lib/dates";
import { BoltIcon, CheckIcon, FlameIcon } from "./Icons";

interface Props {
  habit: HabitSummary;
  /** Today's existing log, when the habit is already done today. */
  initial?: Completion | null;
  onSubmit: (log: CompletionInput) => Promise<void>;
  onCancel: () => void;
}

/** Longest day the backend accepts, and the most that can be entered. */
const MAX_DAY_MINUTES = 24 * 60;
/** The quick-add buttons, in minutes. */
const TIME_BUTTONS = [5, 10, 30];

const showMinutes = (m: number) => formatMinutes(m) || "0 min";

type Mode = "add" | "edit";

/**
 * Popup shown when completing a habit: time spent + how hard it was. If the
 * habit is already done today it can add another session's time to today's
 * log (the default) or edit the day's total.
 */
export function LogForm({ habit, initial, onSubmit, onCancel }: Props) {
  const [mode, setMode] = useState<Mode>(initial ? "add" : "edit");
  // Time entered so far: the session being added, or the day's total.
  const [mins, setMins] = useState(0);
  const [difficulty, setDifficulty] = useState(initial?.difficulty ?? 2);
  const [typing, setTyping] = useState<string | null>(null);
  const [xp, setXp] = useState<XpPreview | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const adding = mode === "add" && !!initial;
  const sofar = adding ? initial.minutes : 0;
  const maxMins = MAX_DAY_MINUTES - sofar;
  const total = Math.min(MAX_DAY_MINUTES, sofar + mins);

  // XP is calculated in Rust; ask it for a preview whenever the inputs change.
  useEffect(() => {
    let stale = false;
    api
      .previewXp(habit.id, { minutes: total, difficulty })
      .then((v) => !stale && setXp(v))
      .catch(() => {});
    return () => {
      stale = true;
    };
  }, [habit.id, total, difficulty]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onCancel();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);
  useBackHandler(onCancel); // Android Back closes the popup

  function switchMode(next: Mode) {
    setMode(next);
    setTyping(null);
    setMins(next === "add" ? 0 : (initial?.minutes ?? 0));
  }

  /** Apply a typed time, ignoring anything that isn't a sensible number. */
  function commitTyped() {
    if (typing === null) return;
    const n = Math.round(Number(typing));
    if (typing.trim() !== "" && Number.isFinite(n)) {
      setMins(Math.min(maxMins, Math.max(0, n)));
    }
    setTyping(null);
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSubmit({ minutes: total, difficulty });
    } catch (err) {
      setError(errorMessage(err));
      setSaving(false);
    }
  }

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <form className="modal" onSubmit={submit} onClick={(e) => e.stopPropagation()}>
        <div>
          <h2>{initial ? "Today's log" : "Nice work!"}</h2>
          <p className="muted small">
            {initial ? (
              <>
                <strong className="text">{habit.name}</strong> is done today with{" "}
                {showMinutes(initial.minutes)} logged.
              </>
            ) : (
              <>
                How did <strong className="text">{habit.name}</strong> go today?
              </>
            )}
          </p>
        </div>

        {initial && (
          <div className="segmented log-mode" role="radiogroup" aria-label="Change today's log">
            {(
              [
                ["add", "Add time"],
                ["edit", "Edit total"],
              ] as const
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={mode === value}
                className={mode === value ? "selected" : ""}
                onClick={() => switchMode(value)}
              >
                {label}
              </button>
            ))}
          </div>
        )}

        <div className="field">
          <span className="field-label">{adding ? "Time to add" : "Time spent"}</span>
          <div className="time-entry">
            {typing === null ? (
              <button
                type="button"
                className="time-value"
                onClick={() => setTyping(mins ? String(mins) : "")}
                title="Type an exact time"
              >
                {adding && "+"}
                {showMinutes(mins)}
              </button>
            ) : (
              <span className="time-typed">
                <input
                  autoFocus
                  type="number"
                  inputMode="numeric"
                  min={0}
                  max={maxMins}
                  step={1}
                  value={typing}
                  placeholder="0"
                  aria-label="Minutes"
                  onChange={(e) => setTyping(e.target.value)}
                  onBlur={commitTyped}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      commitTyped();
                    } else if (e.key === "Escape") {
                      e.stopPropagation(); // cancel typing, not the whole form
                      setTyping(null);
                    }
                  }}
                />
                min
              </span>
            )}
            <div className="time-buttons">
              {TIME_BUTTONS.map((m) => (
                <button
                  key={m}
                  type="button"
                  className="time-add"
                  disabled={mins >= maxMins}
                  onClick={() => {
                    setTyping(null);
                    setMins((v) => Math.min(maxMins, v + m));
                  }}
                >
                  +{m} min
                </button>
              ))}
              <button
                type="button"
                className="time-reset"
                disabled={mins === 0}
                onClick={() => {
                  setTyping(null);
                  setMins(0);
                }}
              >
                Reset
              </button>
            </div>
          </div>
          {adding && (
            <p className="muted small time-total">
              {showMinutes(sofar)} + {showMinutes(mins)} ={" "}
              <strong className="text">{showMinutes(total)}</strong> today
            </p>
          )}
        </div>

        <div className="field">
          <span className="field-label">How difficult was it?</span>
          <div className="difficulty" role="radiogroup" aria-label="Difficulty">
            {DIFFICULTY_LABELS.map((label, i) => {
              const level = i + 1;
              return (
                <button
                  key={label}
                  type="button"
                  role="radio"
                  aria-checked={difficulty === level}
                  className={`difficulty-option d${level} ${difficulty === level ? "selected" : ""}`}
                  onClick={() => setDifficulty(level)}
                >
                  <span className="difficulty-bars" aria-hidden>
                    {[1, 2, 3, 4, 5].map((b) => (
                      <span key={b} className={b <= level ? "on" : ""} />
                    ))}
                  </span>
                  {label}
                </button>
              );
            })}
          </div>
        </div>

        {xp && (
          <div className={`streak-bonus ${xp.streakPercent > 100 ? "active" : ""}`}>
            <FlameIcon size={16} />
            <span>
              {xp.streak > 1 ? `Day ${xp.streak} of your streak` : "Day 1 of a new streak"}
            </span>
            <span className="streak-bonus-mult">
              ×{xp.streakPercent / 100} XP
            </span>
          </div>
        )}

        {error && <p className="error">{error}</p>}

        <div className="modal-actions log-actions">
          <span className="xp-preview" aria-live="polite" title={initial ? "Today's total XP" : undefined}>
            <BoltIcon size={16} />
            {habit.primaryStat ? (
              <>
                <span className={`stat-gain stat-${habit.primaryStat}`}>
                  +{xp?.primary ?? "…"} {statInfo(habit.primaryStat).abbr}
                </span>
                {habit.secondaryStat && (
                  <span className={`stat-gain stat-${habit.secondaryStat}`}>
                    +{xp?.secondary ?? "…"} {statInfo(habit.secondaryStat).abbr}
                  </span>
                )}
              </>
            ) : (
              <span>+{xp?.primary ?? "…"} XP</span>
            )}
          </span>
          <button type="button" className="btn ghost" onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" className="btn primary" disabled={saving || typing !== null || mins === 0}>
            <CheckIcon size={16} strokeWidth={2.5} />
            {adding ? "Add time" : initial ? "Save" : "Log it"}
          </button>
        </div>
      </form>
    </div>
  );
}
