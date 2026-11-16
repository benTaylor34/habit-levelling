import type { HabitSummary } from "../lib/types";
import { difficultyLabel, statInfo } from "../lib/types";
import { formatMinutes } from "../lib/dates";
import { BoltIcon, CheckIcon, WarningIcon } from "./Icons";
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
          {habit.needsAttention && (
            <span
              className="idle-warning"
              title={`Not done in ${habit.idleDays} days. Its stats fade unless another habit trains them.`}
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
