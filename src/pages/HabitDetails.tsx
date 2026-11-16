import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { HabitDetails as Details, IsoDate } from "../lib/types";
import { parseIsoDate } from "../lib/dates";
import { Calendar } from "../components/Calendar";
import { HabitForm } from "../components/HabitForm";
import {
  ArrowLeftIcon,
  ChartIcon,
  CheckIcon,
  FlameIcon,
  PencilIcon,
  TrashIcon,
  TrophyIcon,
} from "../components/Icons";

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

  const back = (
    <button className="btn ghost back" onClick={onBack}>
      <ArrowLeftIcon size={16} /> All habits
    </button>
  );

  if (!details) {
    return (
      <>
        {back}
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

  const recent = [...completions].reverse().slice(0, 8);

  return (
    <>
      {back}

      <header className="page-header">
        <div className="title-block">
          {summary.skill && <p className="eyebrow">{summary.skill}</p>}
          <h1>{summary.name}</h1>
          {summary.description && <p className="muted description">{summary.description}</p>}
        </div>
        <div className="actions">
          <button
            className="btn icon"
            onClick={() => setEditing(true)}
            title="Edit habit"
            aria-label="Edit habit"
          >
            <PencilIcon size={16} />
          </button>
          <button
            className="btn icon danger"
            onClick={remove}
            title="Delete habit"
            aria-label="Delete habit"
          >
            <TrashIcon size={16} />
          </button>
          <button
            className={`btn ${summary.completedToday ? "success" : "primary"}`}
            disabled={busy}
            onClick={toggle}
          >
            <CheckIcon size={16} strokeWidth={2.5} />
            {summary.completedToday ? "Completed" : "Complete today"}
          </button>
        </div>
      </header>

      {error && <p className="error">{error}</p>}

      <div className="stats">
        <div className="stat">
          <span className="stat-icon fire">
            <FlameIcon size={18} />
          </span>
          <span className="stat-value">{summary.currentStreak}</span>
          <span className="stat-label">Current streak</span>
        </div>
        <div className="stat">
          <span className="stat-icon gold">
            <TrophyIcon size={18} />
          </span>
          <span className="stat-value">{summary.longestStreak}</span>
          <span className="stat-label">Longest streak</span>
        </div>
        <div className="stat">
          <span className="stat-icon accent">
            <ChartIcon size={18} />
          </span>
          <span className="stat-value">{summary.totalCompletions}</span>
          <span className="stat-label">Total completions</span>
        </div>
      </div>

      <div className="details-grid">
        <section className="panel">
          <Calendar completions={completions} today={today} />
        </section>
        <section className="panel">
          <div className="panel-header">
            <h2>Recent activity</h2>
          </div>
          {recent.length === 0 ? (
            <p className="muted small">No completions yet.</p>
          ) : (
            <ul className="history">
              {recent.map((d) => (
                <li key={d}>
                  <span className="history-dot">
                    <CheckIcon size={12} strokeWidth={3} />
                  </span>
                  <span>
                    {parseIsoDate(d).toLocaleDateString(undefined, {
                      weekday: "short",
                      day: "numeric",
                      month: "short",
                    })}
                  </span>
                  <span className="muted tiny">{d === today ? "Today" : ""}</span>
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
