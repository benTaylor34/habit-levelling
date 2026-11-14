import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { HabitDetails as Details, IsoDate } from "../lib/types";
import { parseIsoDate } from "../lib/dates";
import { Calendar } from "../components/Calendar";
import { HabitForm } from "../components/HabitForm";
import { StreakBadge } from "../components/StreakBadge";

interface Props {
  id: number;
  today: IsoDate;
  onBack: () => void;
}

export function HabitDetails({ id, today, onBack }: Props) {
  const [details, setDetails] = useState<Details | null>(null);
  const [editing, setEditing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setDetails(await api.getHabitDetails(id));
    } catch (e) {
      setError(errorMessage(e));
    }
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  if (!details) {
    return (
      <>
        <button className="ghost" onClick={onBack}>
          ← Back
        </button>
        {error ? <p className="error">{error}</p> : <p className="muted">Loading…</p>}
      </>
    );
  }

  const { summary, completions } = details;

  async function toggle() {
    setBusy(true);
    try {
      if (summary.completedToday) await api.undoToday(id);
      else await api.completeToday(id);
      await load();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    const ok = window.confirm(
      `Delete "${summary.name}" and its entire completion history? This can't be undone.`,
    );
    if (!ok) return;
    try {
      await api.deleteHabit(id);
      onBack();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  const recent = [...completions].reverse().slice(0, 10);

  return (
    <>
      <button className="ghost" onClick={onBack}>
        ← Back
      </button>

      <header className="page-header">
        <div>
          <h1>{summary.name}</h1>
          {summary.skill && <span className="skill-tag">{summary.skill}</span>}
          {summary.description && <p className="muted">{summary.description}</p>}
        </div>
        <div className="row">
          <button className="secondary" onClick={() => setEditing(true)}>
            Edit
          </button>
          <button className="danger" onClick={remove}>
            Delete
          </button>
        </div>
      </header>

      {error && <p className="error">{error}</p>}

      <div className="stats">
        <div className="stat">
          <span className="muted small">Current streak</span>
          <StreakBadge days={summary.currentStreak} label="days" />
        </div>
        <div className="stat">
          <span className="muted small">Longest streak</span>
          <strong>{summary.longestStreak} days</strong>
        </div>
        <div className="stat">
          <span className="muted small">Total completions</span>
          <strong>{summary.totalCompletions}</strong>
        </div>
        <div className="stat">
          <button
            className={summary.completedToday ? "secondary" : "primary"}
            disabled={busy}
            onClick={toggle}
          >
            {summary.completedToday ? "✓ Done · Undo" : "Complete today"}
          </button>
        </div>
      </div>

      <div className="details-grid">
        <section className="panel">
          <Calendar completions={completions} today={today} />
        </section>
        <section className="panel">
          <h2>Recent completions</h2>
          {recent.length === 0 ? (
            <p className="muted">Nothing yet.</p>
          ) : (
            <ul className="history">
              {recent.map((d) => (
                <li key={d}>
                  ✓{" "}
                  {parseIsoDate(d).toLocaleDateString(undefined, {
                    weekday: "short",
                    day: "numeric",
                    month: "short",
                    year: "numeric",
                  })}
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>

      {editing && (
        <HabitForm
          initial={summary}
          onCancel={() => setEditing(false)}
          onSubmit={async (input) => {
            await api.updateHabit(id, input);
            setEditing(false);
            await load();
          }}
        />
      )}
    </>
  );
}
