import { useState } from "react";
import type { StatProgress } from "../lib/types";
import { STATS } from "../lib/types";

// Geometry in SVG units; the SVG scales to its container's width.
const W = 280;
const H = 250;
const CX = W / 2;
const CY = 128;
const R = 86;
const RINGS = 5;

/** Point at `fraction` of the radius along axis `i` (axis 0 points straight up). */
function point(i: number, fraction: number): [number, number] {
  const angle = -Math.PI / 2 + (i * 2 * Math.PI) / STATS.length;
  return [CX + Math.cos(angle) * R * fraction, CY + Math.sin(angle) * R * fraction];
}

const toPoints = (pts: [number, number][]) => pts.map(([x, y]) => `${x},${y}`).join(" ");

/** Axis maximum: the highest level rounded up to a multiple of 5 (at least 5). */
function scaleMax(levels: number[]) {
  return Math.max(5, Math.ceil(Math.max(...levels) / 5) * 5);
}

/** Hexagonal radar of the six stat levels. Luck is secret, so it isn't plotted. */
export function RadarChart({ stats }: { stats: StatProgress[] }) {
  const [hover, setHover] = useState<number | null>(null);

  const rows = STATS.map((info) => ({
    info,
    progress: stats.find((s) => s.stat === info.key),
  }));
  const levels = rows.map((r) => r.progress?.level ?? 1);
  const max = scaleMax(levels);
  const shape = levels.map((lvl, i) => point(i, lvl / max));

  const summary = rows.map((r, i) => `${r.info.name} ${levels[i]}`).join(", ");
  const hovered = hover === null ? null : rows[hover];
  const [hx, hy] = hover === null ? [0, 0] : shape[hover];

  return (
    <div className="radar" onMouseLeave={() => setHover(null)}>
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`Stat levels: ${summary}`}>
        {/* Recessive grid: concentric hexagons plus spokes. */}
        {Array.from({ length: RINGS }, (_, k) => (
          <polygon
            key={k}
            className="radar-ring"
            points={toPoints(STATS.map((_, i) => point(i, (k + 1) / RINGS)))}
          />
        ))}
        {STATS.map((_, i) => {
          const [x, y] = point(i, 1);
          return <line key={i} className="radar-spoke" x1={CX} y1={CY} x2={x} y2={y} />;
        })}
        {/* Scale labels up the first spoke. */}
        {[RINGS / 5, RINGS].map((k) => {
          const [, y] = point(0, k / RINGS);
          return (
            <text key={k} className="radar-scale" x={CX + 4} y={y + 3}>
              {(max * k) / RINGS}
            </text>
          );
        })}

        <polygon className="radar-shape" points={toPoints(shape)} />

        {rows.map((r, i) => {
          const [x, y] = shape[i];
          const [lx, ly] = point(i, 1.2);
          const anchor = Math.abs(lx - CX) < 4 ? "middle" : lx > CX ? "start" : "end";
          return (
            <g
              key={r.info.key}
              className={`radar-vertex stat-${r.info.key} ${hover === i ? "is-hover" : ""}`}
              onMouseEnter={() => setHover(i)}
            >
              <circle className="radar-dot" cx={x} cy={y} r={4.5} />
              <text className="radar-label" x={lx} y={ly - 2} textAnchor={anchor}>
                {r.info.abbr}
              </text>
              <text className="radar-level" x={lx} y={ly + 11} textAnchor={anchor}>
                {levels[i]}
              </text>
              {/* Generous invisible hit targets on the dot and the label. */}
              <circle className="radar-hit" cx={x} cy={y} r={14} />
              <circle className="radar-hit" cx={lx} cy={ly + 3} r={18} />
            </g>
          );
        })}
      </svg>

      {hovered && (
        <div
          className="radar-tip"
          style={{ left: `${(hx / W) * 100}%`, top: `${(hy / H) * 100}%` }}
          role="status"
        >
          <div className="radar-tip-title">
            <span className={`radar-tip-swatch stat-${hovered.info.key}`} />
            {hovered.info.name}
          </div>
          <div>Level {levels[hover!]}</div>
          <div className="muted">{hovered.progress?.xp ?? 0} XP total</div>
        </div>
      )}
    </div>
  );
}
