import type { HabitSummary } from "../lib/types";
import { StreakBadge } from "./StreakBadge";

interface Props {
  habit: HabitSummary;
  busy: boolean;
  onToggle: (habit: HabitSummary) => void;
  onOpen: (id: number) => void;
}

const WEEKDAY = ["S", "M", "T", "W", "T", "F", "S"];

export function HabitCard({ habit, busy, onToggle, onOpen }: Props) {
  // Labels for the last-7-days strip (ending today).
  const labels = habit.recentDays.map((_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - (habit.recentDays.length - 1 - i));
    return WEEKDAY[d.getDay()];
  });

  return (
    <article className={`habit-card ${habit.completedToday ? "done" : ""}`}>
      <button className="habit-main" onClick={() => onOpen(habit.id)}>
        <div className="habit-title">
          <h3>{habit.name}</h3>
          {habit.skill && <span className="skill-tag">{habit.skill}</span>}
        </div>
        <StreakBadge days={habit.currentStreak} />
        <div className="week-strip" aria-label="Last 7 days">
          {habit.recentDays.map((done, i) => (
            <span key={i} className={`week-day ${done ? "filled" : ""}`}>
              {labels[i]}
            </span>
          ))}
        </div>
      </button>
      <button
        className={habit.completedToday ? "secondary" : "primary"}
        disabled={busy}
        onClick={() => onToggle(habit)}
      >
        {habit.completedToday ? "✓ Done · Undo" : "Complete"}
      </button>
    </article>
  );
}
