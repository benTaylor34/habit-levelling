import type { StatProgress } from "../lib/types";
import { statInfo } from "../lib/types";
import { StreakBadge } from "./StreakBadge";

function StatRow({ s }: { s: StatProgress }) {
  const info = statInfo(s.stat);
  const maxed = s.xpForNextLevel === 0;
  const pct = maxed ? 100 : (s.xpIntoLevel / s.xpForNextLevel) * 100;
  return (
    <li className={`stat-row stat-${s.stat}`} title={info.description}>
      <span className="level-badge" aria-label={`Level ${s.level}`}>
        {s.level}
      </span>
      <div className="skill-info">
        <div className="skill-top">
          <span className="skill-name">
            {info.name} <span className="stat-abbr-inline">{info.abbr}</span>
          </span>
          {s.stat !== "luck" && (
            <span title={`Longest: ${s.longestStreak} days`}>
              <StreakBadge days={s.currentStreak} compact />
            </span>
          )}
        </div>
        <div
          className="xp-bar"
          role="progressbar"
          aria-label={`${info.name} level ${s.level}`}
          aria-valuenow={s.xpIntoLevel}
          aria-valuemax={s.xpForNextLevel}
        >
          <div style={{ width: `${pct}%` }} />
        </div>
        <div className="muted tiny skill-xp">
          {maxed ? "Max level" : `${s.xpIntoLevel} / ${s.xpForNextLevel} XP to level ${s.level + 1}`}
        </div>
      </div>
    </li>
  );
}

export function StatPanel({ stats }: { stats: StatProgress[] }) {
  const trainable = stats.filter((s) => s.stat !== "luck");
  const luck = stats.find((s) => s.stat === "luck");
  const totalLevel = stats.reduce((sum, s) => sum + s.level, 0);

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Stats</h2>
        {stats.length > 0 && <span className="muted small">Total level {totalLevel}</span>}
      </div>
      <ul className="skill-list">
        {trainable.map((s) => (
          <StatRow key={s.stat} s={s} />
        ))}
      </ul>
      {luck && (
        <div className="luck-section">
          <div className="luck-label">Secret stat unlocked</div>
          <ul className="skill-list">
            <StatRow s={luck} />
          </ul>
        </div>
      )}
    </section>
  );
}
