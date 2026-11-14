import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { HabitSummary, IsoDate, SkillProgress } from "../lib/types";
import { formatLong } from "../lib/dates";
import { HabitCard } from "../components/HabitCard";
import { HabitForm } from "../components/HabitForm";
import { SkillPanel } from "../components/SkillPanel";

interface Props {
  today: IsoDate;
  onOpen: (id: number) => void;
}

export function Dashboard({ today, onOpen }: Props) {
  const [habits, setHabits] = useState<HabitSummary[] | null>(null);
  const [skills, setSkills] = useState<SkillProgress[]>([]);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
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

  async function toggle(habit: HabitSummary) {
    setBusyId(habit.id);
    try {
      const updated = habit.completedToday
        ? await api.undoToday(habit.id)
        : await api.completeToday(habit.id);
      setHabits((hs) => hs?.map((h) => (h.id === updated.id ? updated : h)) ?? null);
      setSkills(await api.getSkillProgress());
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusyId(null);
    }
  }

  const doneCount = habits?.filter((h) => h.completedToday).length ?? 0;

  return (
    <>
      <header className="page-header">
        <div>
          <h1>Habit Levelling</h1>
          <p className="muted">Today: {formatLong(today)}</p>
        </div>
        {habits && habits.length > 0 && (
          <div className="today-count">
            <strong>
              {doneCount}/{habits.length}
            </strong>
            <span className="muted small">done today</span>
          </div>
        )}
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
              <p>No habits yet.</p>
              <p className="muted">Create your first habit to start levelling up.</p>
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
          <button className="primary create" onClick={() => setCreating(true)}>
            + Create Habit
          </button>
        </section>

        <SkillPanel skills={skills} />
      </div>

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
