import type { HabitSummary } from "../lib/types";
import { difficultyLabel, statInfo } from "../lib/types";
import { formatMinutes } from "../lib/dates";
import { BoltIcon, CheckIcon, PlusIcon, WarningIcon } from "./Icons";
import { StreakBadge } from "./StreakBadge";

interface Props {
  habit: HabitSummary;
  busy: boolean;
  onToggle: (habit: HabitSummary) => void;
  /** Add another session's time to today's log. */
  onAddTime: (habit: HabitSummary) => void;
  onOpen: (id: number) => void;
}

const WEEKDAY = ["S", "M", "T", "W", "T", "F", "S"];

export function HabitCard({ habit, busy, onToggle, onAddTime, onOpen }: Props) {
  // Labels for the last-7-days strip (ending today).
  const labels = habit.recentDays.map((_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - (habit.recentDays.length - 1 - i));
    return WEEKDAY[d.getDay()];
  });

  return (
    <article className={`habit-card ${habit.completedToday ? "done" : ""}`}>
      <button
        className="check"
        aria-pressed={habit.completedToday}
        aria-label={habit.completedToday ? `Undo ${habit.name}` : `Complete ${habit.name}`}
        title={habit.completedToday ? "Completed today · click to undo" : "Log today"}
        disabled={busy}
        onClick={() => onToggle(habit)}
      >
        <CheckIcon size={16} strokeWidth={3} />
      </button>

      <button className="habit-body" onClick={() => onOpen(habit.id)}>
        <span className="habit-name">
          {habit.name}
          <span className="habit-level" title={`Habit level ${habit.level}`}>
            Lv {habit.level}
          </span>
          {habit.decay !== "none" && (
            <span
              className="idle-warning"
              title={`Not done in ${habit.idleDays} days. Its level is fading${
                habit.decay === "level" ? " and can drop" : ""
              } until you do it again.`}
            >
              <WarningIcon size={13} strokeWidth={2.4} />
              {habit.idleDays}d
            </span>
          )}
        </span>
        <span className="habit-meta">
          <StreakBadge days={habit.currentStreak} />
          {habit.primaryStat && (
            <span className="stat-tags">
              <span
                className={`stat-tag stat-${habit.primaryStat}`}
                title={`Primary: ${statInfo(habit.primaryStat).name}`}
              >
                {statInfo(habit.primaryStat).abbr}
              </span>
              {habit.secondaryStat && (
                <span
                  className={`stat-tag secondary stat-${habit.secondaryStat}`}
                  title={`Secondary: ${statInfo(habit.secondaryStat).name}`}
                >
                  {statInfo(habit.secondaryStat).abbr}
                </span>
              )}
            </span>
          )}
          {habit.todayLog && (
            <span className="log-meta" title={difficultyLabel(habit.todayLog.difficulty)}>
              {formatMinutes(habit.todayLog.minutes) || difficultyLabel(habit.todayLog.difficulty)}
              <span className="xp-gain">
                <BoltIcon size={12} />+{habit.todayLog.xp} XP
              </span>
            </span>
          )}
        </span>
      </button>

      {habit.completedToday && (
        <button
          className="btn ghost add-time"
          disabled={busy}
          onClick={() => onAddTime(habit)}
          title="Add another session to today's log"
          aria-label={`Add time to ${habit.name}`}
        >
          <PlusIcon size={14} strokeWidth={2.4} />
          Time
        </button>
      )}

      <div className="week" aria-label="Last 7 days">
        {habit.recentDays.map((done, i) => (
          <div key={i} className="week-col">
            <span
              className={`week-dot ${done ? "filled" : ""} ${
                i === habit.recentDays.length - 1 ? "today" : ""
              }`}
            />
            <span className="week-label">{labels[i]}</span>
          </div>
        ))}
      </div>
    </article>
  );
}
