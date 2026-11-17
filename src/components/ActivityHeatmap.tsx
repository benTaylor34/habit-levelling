import { useLayoutEffect, useRef, useState } from "react";
import type { Activity, IsoDate } from "../lib/types";
import { formatMinutes, parseIsoDate } from "../lib/dates";

const CELL = 12;
const GAP = 3;
const STEP = CELL + GAP;
const LEFT = 32; // weekday labels
const TOP = 18; // month labels
const ROW_LABELS: [number, string][] = [
  [0, "Mon"],
  [2, "Wed"],
  [4, "Fri"],
];

/** Columns from either edge where the tooltip hugs that edge instead of centring. */
const EDGE_COLUMNS = 6;
/** Rows 0–2 (Mon–Wed) open the tooltip below the square; later rows open it above. */
const FLIP_BELOW_ROW = 3;

/**
 * Place the tooltip so it never leaves the heatmap (the scroll area clips
 * anything outside it): below the square on the top rows, above it on the
 * bottom rows, and left- or right-aligned near the ends of the year.
 */
function tipPosition(index: number, columns: number): React.CSSProperties {
  const col = Math.floor(index / 7);
  const row = index % 7;
  const cellLeft = LEFT + col * STEP;
  const cellTop = TOP + row * STEP;
  const below = row < FLIP_BELOW_ROW;

  let left: number;
  let x: string;
  if (col < EDGE_COLUMNS) {
    left = cellLeft; // grow rightwards from the square's left edge
    x = "0";
  } else if (col >= columns - EDGE_COLUMNS) {
    left = cellLeft + CELL; // grow leftwards from the square's right edge
    x = "-100%";
  } else {
    left = cellLeft + CELL / 2;
    x = "-50%";
  }
  return below
    ? { left, top: cellTop + CELL + 6, transform: `translateX(${x})` }
    : { left, top: cellTop - 6, transform: `translate(${x}, -100%)` };
}

/** GitHub-style year of activity: one column per week (Monday first), shaded by habits logged. */
export function ActivityHeatmap({ activity, today }: { activity: Activity; today: IsoDate }) {
  const scroller = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<number | null>(null);
  const { days } = activity;
  const columns = Math.ceil(days.length / 7);
  const width = LEFT + columns * STEP;
  const height = TOP + 7 * STEP;

  // Most recent weeks matter most: start scrolled to the right on narrow windows.
  useLayoutEffect(() => {
    const el = scroller.current;
    if (el) el.scrollLeft = el.scrollWidth;
  }, [days.length]);

  // Month labels where a new month starts, skipping ones that would collide.
  const months: { col: number; label: string }[] = [];
  for (let col = 0; col < columns; col++) {
    const date = parseIsoDate(days[col * 7].date);
    const prev = col > 0 ? parseIsoDate(days[(col - 1) * 7].date) : null;
    if (!prev || prev.getMonth() !== date.getMonth()) {
      if (months.length && col - months[months.length - 1].col < 3) continue;
      months.push({ col, label: date.toLocaleDateString(undefined, { month: "short" }) });
    }
  }

  function onMove(e: React.PointerEvent<SVGSVGElement>) {
    const rect = e.currentTarget.getBoundingClientRect();
    const col = Math.floor((e.clientX - rect.left - LEFT) / STEP);
    const row = Math.floor((e.clientY - rect.top - TOP) / STEP);
    const i = col * 7 + row;
    setHover(col >= 0 && row >= 0 && row < 7 && i < days.length ? i : null);
  }

  const h = hover === null ? null : days[hover];
  const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

  return (
    <div className="heatmap">
      <div className="heatmap-scroll" ref={scroller}>
        <div className="heatmap-inner" style={{ width }}>
          <svg
            width={width}
            height={height}
            role="img"
            aria-label={`Activity over the last year: ${plural(activity.totalCompletions, "habit")} logged on ${plural(activity.activeDays, "day")}.`}
            onPointerDown={onMove}
            onPointerMove={onMove}
            onPointerLeave={(e) => e.pointerType !== "touch" && setHover(null)}
          >
            {months.map((m) => (
              <text key={m.col} className="hm-label" x={LEFT + m.col * STEP} y={11}>
                {m.label}
              </text>
            ))}
            {ROW_LABELS.map(([row, label]) => (
              <text key={row} className="hm-label" x={0} y={TOP + row * STEP + CELL - 2}>
                {label}
              </text>
            ))}
            {days.map((day, i) => (
              <rect
                key={day.date}
                className={`hm-cell lvl-${day.level} ${day.date === today ? "is-today" : ""} ${
                  hover === i ? "is-hover" : ""
                }`}
                x={LEFT + Math.floor(i / 7) * STEP}
                y={TOP + (i % 7) * STEP}
                width={CELL}
                height={CELL}
                rx={2.5}
              />
            ))}
          </svg>

          {h && hover !== null && (
            <div className="hm-tip" style={tipPosition(hover, columns)}
            >
              <strong>
                {h.completions === 0 ? "No habits logged" : `${plural(h.completions, "habit")} logged`}
              </strong>
              {h.completions > 0 && (
                <span className="muted">
                  {" · "}
                  {[formatMinutes(h.minutes), `${h.xp} XP`].filter(Boolean).join(" · ")}
                </span>
              )}
              <div className="muted">
                {parseIsoDate(h.date).toLocaleDateString(undefined, {
                  weekday: "short",
                  day: "numeric",
                  month: "short",
                  year: "numeric",
                })}
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="hm-footer">
        <span className="muted tiny">
          {plural(activity.activeDays, "active day")} · best day {plural(activity.bestDay, "habit")}
        </span>
        <span className="hm-legend muted tiny" aria-hidden>
          Less
          {[0, 1, 2, 3, 4].map((l) => (
            <span key={l} className={`hm-swatch lvl-${l}`} />
          ))}
          More
        </span>
      </div>
    </div>
  );
}
