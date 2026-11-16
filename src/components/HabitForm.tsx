import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import type { Habit, HabitInput, StatKey, StatRecommendation } from "../lib/types";
import { statInfo } from "../lib/types";
import { SparkleIcon } from "./Icons";
import { StatPicker } from "./StatPicker";

interface Props {
  initial?: Habit;
  onSubmit: (input: HabitInput) => Promise<void>;
  onCancel: () => void;
}

export function HabitForm({ initial, onSubmit, onCancel }: Props) {
  const [name, setName] = useState(initial?.name ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [primary, setPrimary] = useState<StatKey | null>(initial?.primaryStat ?? null);
  const [secondary, setSecondary] = useState<StatKey | null>(initial?.secondaryStat ?? null);
  const [suggestion, setSuggestion] = useState<StatRecommendation | null>(null);
  // While false, suggestions fill the stat pickers as you type. Picking a stat
  // yourself stops that. Editing never overwrites a habit's existing stats.
  const [statsTouched, setStatsTouched] = useState(Boolean(initial));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function apply(p: StatKey | null, s: StatKey | null) {
    setPrimary(p);
    // A secondary stat needs a primary, and can't be the same stat.
    setSecondary(p === null || s === p ? null : s);
  }

  // Ask Rust for a recommendation shortly after the user stops typing.
  useEffect(() => {
    if (!name.trim() && !description.trim()) {
      setSuggestion(null);
      return;
    }
    let stale = false;
    const t = setTimeout(() => {
      api
        .recommendStats(name, description || null)
        .then((rec) => {
          if (stale) return;
          setSuggestion(rec);
          if (!statsTouched && rec.primary) apply(rec.primary, rec.secondary);
        })
        .catch(() => {});
    }, 250);
    return () => {
      stale = true;
      clearTimeout(t);
    };
  }, [name, description, statsTouched]);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSubmit({
        name,
        description: description || null,
        primaryStat: primary,
        secondaryStat: secondary,
      });
    } catch (err) {
      setError(String(err));
      setSaving(false);
    }
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onCancel();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  const suggested = suggestion?.primary ? suggestion : null;
  const usingSuggestion =
    suggested !== null && primary === suggested.primary && secondary === suggested.secondary;

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <form className="modal" onSubmit={submit} onClick={(e) => e.stopPropagation()}>
        <div>
          <h2>{initial ? "Edit habit" : "New habit"}</h2>
          <p className="muted small">
            {initial ? "Update the details of this habit." : "Something you want to do every day."}
          </p>
        </div>

        <label>
          <span>Name</span>
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Study for an hour"
            required
          />
        </label>

        <label>
          <span>Description <span className="muted">(optional)</span></span>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
          />
        </label>

        {suggested && (
          <div className={`suggestion ${usingSuggestion ? "applied" : ""}`} aria-live="polite">
            <SparkleIcon size={16} />
            <div className="suggestion-text">
              <div>
                {usingSuggestion ? "Suggested " : "Suggestion: "}
                <strong className={`stat-${suggested.primary}`}>
                  {statInfo(suggested.primary!).name}
                </strong>
                {suggested.secondary && (
                  <>
                    {" + "}
                    <strong className={`stat-${suggested.secondary}`}>
                      {statInfo(suggested.secondary).name}
                    </strong>
                  </>
                )}
              </div>
              <div className="muted tiny">
                Based on “{suggested.matched.slice(0, 3).join("”, “")}”
              </div>
            </div>
            {!usingSuggestion && (
              <button
                type="button"
                className="btn ghost suggestion-use"
                onClick={() => apply(suggested.primary, suggested.secondary)}
              >
                Use
              </button>
            )}
          </div>
        )}

        <StatPicker
          label="Primary stat"
          hint="(full XP)"
          value={primary}
          onChange={(s) => {
            setStatsTouched(true);
            apply(s, secondary);
          }}
        />

        <StatPicker
          label="Secondary stat"
          hint="(half XP, optional)"
          value={secondary}
          onChange={(s) => {
            setStatsTouched(true);
            setSecondary(s);
          }}
          exclude={primary}
          disabled={primary === null}
        />

        {error && <p className="error">{error}</p>}

        <div className="modal-actions">
          <button type="button" className="btn ghost" onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" className="btn primary" disabled={saving || !name.trim()}>
            {initial ? "Save" : "Create"}
          </button>
        </div>
      </form>
    </div>
  );
}
