import type { SkillProgress } from "../lib/types";

export function SkillPanel({ skills }: { skills: SkillProgress[] }) {
  if (skills.length === 0) return null;
  return (
    <section className="panel">
      <h2>Skills</h2>
      <ul className="skill-list">
        {skills.map((s) => (
          <li key={s.skill}>
            <div className="row between">
              <span>
                <strong>{s.skill}</strong> <span className="muted">Lv {s.level}</span>
              </span>
              <span className="muted small">{s.xp} XP</span>
            </div>
            <div
              className="xp-bar"
              role="progressbar"
              aria-valuenow={s.xpIntoLevel}
              aria-valuemax={s.xpForNextLevel}
            >
              <div style={{ width: `${(s.xpIntoLevel / s.xpForNextLevel) * 100}%` }} />
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
