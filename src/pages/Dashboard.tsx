import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { CompletionInput, HabitSummary, IsoDate, SkillProgress } from "../lib/types";
import { formatLong } from "../lib/dates";
import { HabitCard } from "../components/HabitCard";
import { HabitForm } from "../components/HabitForm";
import { LogForm } from "../components/LogForm";
import { SkillPanel } from "../components/SkillPanel";
import { PlusIcon } from "../components/Icons";

interface Props {
  today: IsoDate;
  onOpen: (id: number) => void;
}

function ProgressRing({ done, total }: { done: number; total: number }) {
  const r = 22;
  const c = 2 * Math.PI * r;
  const pct = total === 0 ? 0 : done / total;
  return (
    <div className="ring" aria-label={`${done} of ${total} done today`}>
      <svg width="56" height="56" viewBox="0 0 56 56">
        <circle cx="28" cy="28" r={r} className="ring-track" />
        <circle
          cx="28"
          cy="28"
          r={r}
          className="ring-fill"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - pct)}
        />
      </svg>
      <span className="ring-text">
        {done}/{total}
      </span>
    </div>
  );
}

export function Dashboard({ today, onOpen }: Props) {
  const [habits, setHabits] = useState<HabitSummary[] | null>(null);
  const [skills, setSkills] = useState<SkillProgress[]>([]);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [logging, setLogging] = useState<HabitSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [h, s] = await Promise.all([api.listHabits(), api.getSkillProgress()]);
      setHabits(h);
      setSkills(s);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  async function apply(id: number, action: () => Promise<HabitSummary>) {
    setBusyId(id);
    try {
      const updated = await action();
      setHabits((hs) => hs?.map((h) => (h.id === updated.id ? updated : h)) ?? null);
      setSkills(await api.getSkillProgress());
    } finally {
      setBusyId(null);
    }
  }

  // Unchecking undoes today straight away; checking asks for time + difficulty.
  function toggle(habit: HabitSummary) {
    if (habit.completedToday) {
      apply(habit.id, () => api.undoToday(habit.id)).catch((e) => setError(errorMessage(e)));
    } else {
      setLogging(habit);
    }
  }

  async function logToday(habit: HabitSummary, log: CompletionInput) {
    await apply(habit.id, () => api.completeToday(habit.id, log));
    setLogging(null);
  }

  const total = habits?.length ?? 0;
  const doneCount = habits?.filter((h) => h.completedToday).length ?? 0;
  const bestStreak = habits?.reduce((m, h) => Math.max(m, h.currentStreak), 0) ?? 0;
  const xpToday = habits?.reduce((sum, h) => sum + (h.todayLog?.xp ?? 0), 0) ?? 0;

  return (
    <>
      <header className="page-header">
        <div>
          <p className="eyebrow">{formatLong(today)}</p>
          <h1>Today</h1>
        </div>
        <button className="btn primary" onClick={() => setCreating(true)}>
          <PlusIcon size={16} /> New habit
        </button>
      </header>

      {error && (
        <p className="error" onClick={() => setError(null)}>
          {error}
        </p>
      )}

      <div className="dashboard">
        <section className="habit-list">
          {habits === null ? (
            <p className="muted">Loading…</p>
          ) : habits.length === 0 ? (
            <div className="empty">
              <h3>No habits yet</h3>
              <p className="muted">Create your first habit to start levelling up.</p>
              <button className="btn primary" onClick={() => setCreating(true)}>
                <PlusIcon size={16} /> Create habit
              </button>
            </div>
          ) : (
            habits.map((h) => (
              <HabitCard
                key={h.id}
                habit={h}
                busy={busyId === h.id}
                onToggle={toggle}
                onOpen={onOpen}
              />
            ))
          )}
        </section>

        <aside className="sidebar">
          {total > 0 && (
            <section className="panel overview">
              <ProgressRing done={doneCount} total={total} />
              <div>
                <div className="overview-title">
                  {doneCount === total ? "All done today" : `${total - doneCount} left today`}
                </div>
                <div className="muted small">
                  +{xpToday} XP today · best streak {bestStreak}d
                </div>
              </div>
            </section>
          )}
          <SkillPanel skills={skills} />
        </aside>
      </div>

      {logging && (
        <LogForm
          habitName={logging.name}
          onCancel={() => setLogging(null)}
          onSubmit={(log) => logToday(logging, log)}
        />
      )}

      {creating && (
        <HabitForm
          onCancel={() => setCreating(false)}
          onSubmit={async (input) => {
            await api.createHabit(input);
            setCreating(false);
            await load();
          }}
        />
      )}
    </>
  );
}
