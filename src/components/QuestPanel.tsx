import type { DailyQuest } from "../lib/types";
import { statInfo } from "../lib/types";
import { BoltIcon, CheckIcon } from "./Icons";

interface Props {
  quests: DailyQuest[];
  busyId: string | null;
  onToggle: (quest: DailyQuest) => void;
}

/** Today's three quests. Targets and XP are worked out in Rust (habit_core::quests). */
export function QuestPanel({ quests, busyId, onToggle }: Props) {
  if (quests.length === 0) return null;
  const done = quests.filter((q) => q.completed).length;

  return (
    <section className="panel quest-panel">
      <div className="panel-header">
        <h2>Daily quests</h2>
        <span className="muted small">
          {done === quests.length ? "All done, new quests tomorrow" : `${done}/${quests.length} done`}
        </span>
      </div>
      <ul className="quest-list">
        {quests.map((q) => {
          const info = statInfo(q.stat);
          return (
            <li key={q.id} className={`quest stat-${q.stat} ${q.completed ? "done" : ""}`}>
              <button
                className="check"
                aria-pressed={q.completed}
                aria-label={q.completed ? `Undo quest: ${q.text}` : `Complete quest: ${q.text}`}
                title={q.completed ? "Done today · click to undo" : "Mark done"}
                disabled={busyId === q.id}
                onClick={() => onToggle(q)}
              >
                <CheckIcon size={16} strokeWidth={3} />
              </button>
              <div className="quest-body">
                <span className="quest-text">{q.text}</span>
                <span className="quest-meta">
                  <span className={`stat-tag stat-${q.stat}`} title={info.name}>
                    {info.abbr}
                  </span>
                  <span className="xp-gain">
                    <BoltIcon size={12} />+{q.xp} XP
                  </span>
                </span>
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
