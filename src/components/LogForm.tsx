import { useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { Completion, CompletionInput, Habit, XpPreview } from "../lib/types";
import { DIFFICULTY_LABELS, statInfo } from "../lib/types";
import { formatMinutes } from "../lib/dates";
import { BoltIcon, CheckIcon } from "./Icons";

interface Props {
  habit: Habit;
  /** Today's existing log when editing it. */
  initial?: Completion | null;
  onSubmit: (log: CompletionInput) => Promise<void>;
  onCancel: () => void;
}

const QUICK_MINUTES = [15, 30, 45, 60, 90];

/** Popup shown when completing a habit: time spent + how hard it was. */
export function LogForm({ habit, initial, onSubmit, onCancel }: Props) {
  const [minutes, setMinutes] = useState(initial ? String(initial.minutes) : "");
  const [difficulty, setDifficulty] = useState(initial?.difficulty ?? 2);
  const [xp, setXp] = useState<XpPreview | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const mins = minutes.trim() === "" ? 0 : Number(minutes);
  const minutesValid = Number.isInteger(mins) && mins >= 0 && mins <= 1440;

  // XP is calculated in Rust; ask it for a preview whenever the inputs change.
  useEffect(() => {
    if (!minutesValid) return;
    let stale = false;
    api
      .previewXp({ minutes: mins, difficulty })
      .then((v) => !stale && setXp(v))
      .catch(() => {});
    return () => {
      stale = true;
    };
  }, [mins, difficulty, minutesValid]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onCancel();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!minutesValid) return;
    setSaving(true);
    setError(null);
    try {
      await onSubmit({ minutes: mins, difficulty });
    } catch (err) {
      setError(errorMessage(err));
      setSaving(false);
    }
  }

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <form className="modal" onSubmit={submit} onClick={(e) => e.stopPropagation()}>
        <div>
          <h2>{initial ? "Edit today's log" : "Nice work!"}</h2>
          <p className="muted small">
            How did <strong className="text">{habit.name}</strong> go today?
          </p>
        </div>

        <label>
          <span>
            Time spent <span className="muted">(minutes)</span>
          </span>
          <input
            autoFocus
            type="number"
            inputMode="numeric"
            min={0}
            max={1440}
            step={1}
            value={minutes}
            onChange={(e) => setMinutes(e.target.value)}
            placeholder="0"
          />
        </label>
        <div className="chips" role="group" aria-label="Quick time">
          {QUICK_MINUTES.map((m) => (
            <button
              key={m}
              type="button"
              className={`chip ${mins === m ? "selected" : ""}`}
              onClick={() => setMinutes(String(m))}
            >
              {formatMinutes(m)}
            </button>
          ))}
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

        {!minutesValid && <p className="error">Enter a whole number of minutes, up to 24 hours.</p>}
        {error && <p className="error">{error}</p>}

        <div className="modal-actions log-actions">
          <span className="xp-preview" aria-live="polite">
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
          <button type="submit" className="btn primary" disabled={saving || !minutesValid}>
            <CheckIcon size={16} strokeWidth={2.5} />
            {initial ? "Save" : "Log it"}
          </button>
        </div>
      </form>
    </div>
  );
}
