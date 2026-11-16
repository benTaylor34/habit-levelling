import type { SkillProgress } from "../lib/types";

export function SkillPanel({ skills }: { skills: SkillProgress[] }) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Skills</h2>
        {skills.length > 0 && (
          <span className="muted small">
            {skills.reduce((sum, s) => sum + s.xp, 0)} XP total
          </span>
        )}
      </div>
      {skills.length === 0 ? (
        <p className="muted small">
          Tag a habit with a skill to start earning XP.
        </p>
      ) : (
        <ul className="skill-list">
          {skills.map((s) => (
            <li key={s.skill}>
              <div className="skill-row">
                <span className="level-badge">{s.level}</span>
                <div className="skill-info">
                  <div className="skill-top">
                    <span className="skill-name">{s.skill}</span>
                    <span className="muted tiny">
                      {s.xpIntoLevel}/{s.xpForNextLevel} XP
                    </span>
                  </div>
                  <div
                    className="xp-bar"
                    role="progressbar"
                    aria-label={`${s.skill} level ${s.level}`}
                    aria-valuenow={s.xpIntoLevel}
                    aria-valuemax={s.xpForNextLevel}
                  >
                    <div style={{ width: `${(s.xpIntoLevel / s.xpForNextLevel) * 100}%` }} />
                  </div>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
