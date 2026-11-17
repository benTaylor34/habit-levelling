import { useLayoutEffect, useRef, useState } from "react";
import type { IsoDate } from "../lib/types";
import { parseIsoDate } from "../lib/dates";

export interface LineSeries {
  key: string;
  label: string;
  /** Short name printed at the end of the line, e.g. "STR". */
  shortLabel: string;
  /** CSS class that sets `--stat` (the series colour). */
  colorClass: string;
  values: number[];
}

interface Props {
  dates: IsoDate[];
  series: LineSeries[];
  /** Lowest value on the y axis (e.g. 1 for levels, 0 for XP). */
  yMin: number;
  /** Exact value, used in the tooltip. */
  formatValue: (v: number) => string;
  /** Compact value for axis ticks; defaults to `formatValue`. */
  formatTick?: (v: number) => string;
  /** Series to draw on top and emphasise (e.g. hovered in the legend). */
  highlight?: string | null;
  ariaLabel: string;
}

const HEIGHT = 280;
const M = { top: 12, right: 48, bottom: 28, left: 46 };
/** Minimum vertical gap between end-of-line labels. */
const LABEL_GAP = 13;

/** Spread label y positions so none overlap, keeping each near its line end. */
function spreadLabels(ys: number[], top: number, bottom: number): number[] {
  const order = ys.map((y, i) => ({ y, i })).sort((a, b) => a.y - b.y);
  for (let k = 1; k < order.length; k++) {
    order[k].y = Math.max(order[k].y, order[k - 1].y + LABEL_GAP);
  }
  // If the stack ran off the bottom, push it back up.
  const overflow = order.length ? order[order.length - 1].y - bottom : 0;
  if (overflow > 0) order.forEach((o) => (o.y -= overflow));
  for (let k = 0; k < order.length && order[k].y < top; k++) order[k].y = top + k * LABEL_GAP;
  const out = new Array<number>(ys.length);
  order.forEach((o) => (out[o.i] = o.y));
  return out;
}

/** Round numbers for axis ticks: steps of 1, 2, 2.5 or 5 × 10^k. */
function niceTicks(min: number, max: number, target = 4): number[] {
  const span = Math.max(max - min, 1e-9);
  const raw = span / target;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
  const top = Math.ceil(max / step) * step;
  const ticks: number[] = [];
  for (let v = Math.floor(min / step) * step; v <= top + step / 2; v += step) {
    ticks.push(Math.round(v * 1000) / 1000);
  }
  return ticks;
}

function formatDay(iso: IsoDate, long: boolean) {
  return parseIsoDate(iso).toLocaleDateString(
    undefined,
    long ? { month: "short", year: "2-digit" } : { day: "numeric", month: "short" },
  );
}

/** Multi-series line chart with a hover/keyboard crosshair that reads out every series. */
export function LineChart({
  dates,
  series,
  yMin,
  formatValue,
  formatTick = formatValue,
  highlight,
  ariaLabel,
}: Props) {
  const wrap = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(640);
  const [hover, setHover] = useState<number | null>(null);

  // Measure before paint so the first frame is already the right width.
  useLayoutEffect(() => {
    const el = wrap.current;
    if (!el) return;
    setWidth(Math.max(280, el.getBoundingClientRect().width));
    const ro = new ResizeObserver(([entry]) => setWidth(Math.max(280, entry.contentRect.width)));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const n = dates.length;
  const plotW = width - M.left - M.right;
  const plotH = HEIGHT - M.top - M.bottom;
  const maxVal = Math.max(yMin + 1, ...series.flatMap((s) => s.values));
  const yTicks = niceTicks(yMin, maxVal);
  const yLo = yTicks[0];
  const yHi = yTicks[yTicks.length - 1];

  const x = (i: number) => M.left + (n <= 1 ? plotW / 2 : (i / (n - 1)) * plotW);
  const y = (v: number) => M.top + plotH - ((v - yLo) / (yHi - yLo)) * plotH;

  const longRange = n > 400; // month + year only once day labels would repeat
  const xTickCount = Math.min(n, Math.max(2, Math.floor(plotW / 90)));
  const xTicks = Array.from({ length: xTickCount }, (_, k) =>
    Math.round((k / Math.max(1, xTickCount - 1)) * (n - 1)),
  );

  function indexAt(clientX: number) {
    const rect = wrap.current!.getBoundingClientRect();
    const px = clientX - rect.left - M.left;
    return Math.min(n - 1, Math.max(0, Math.round((px / plotW) * (n - 1))));
  }

  function onKey(e: React.KeyboardEvent) {
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      const step = e.key === "ArrowLeft" ? -1 : 1;
      setHover((h) => Math.min(n - 1, Math.max(0, (h ?? n - 1) + step)));
    } else if (e.key === "Escape") {
      setHover(null);
    }
  }

  // Highlighted series last so it draws on top.
  const ordered = [...series].sort((a, b) => Number(a.key === highlight) - Number(b.key === highlight));
  const readout =
    hover === null
      ? []
      : [...series].sort((a, b) => b.values[hover] - a.values[hover]);
  const endYs = series.map((s) => y(s.values[n - 1]));
  const labelYs = spreadLabels(endYs, M.top + 4, M.top + plotH);
  const tipOnLeft = hover !== null && x(hover) > M.left + plotW * 0.6;

  return (
    <div className="line-chart" ref={wrap}>
      <svg
        width={width}
        height={HEIGHT}
        role="img"
        aria-label={ariaLabel}
        tabIndex={0}
        onKeyDown={onKey}
        onBlur={() => setHover(null)}
        onPointerDown={(e) => setHover(indexAt(e.clientX))}
        onPointerMove={(e) => setHover(indexAt(e.clientX))}
        // On touch, "leave" fires as the finger lifts; keep the readout until
        // the next tap elsewhere (blur) instead.
        onPointerLeave={(e) => e.pointerType !== "touch" && setHover(null)}
      >
        {/* Recessive hairline grid and y labels. */}
        {yTicks.map((t) => (
          <g key={t}>
            <line className="lc-grid" x1={M.left} x2={M.left + plotW} y1={y(t)} y2={y(t)} />
            <text className="lc-axis" x={M.left - 8} y={y(t) + 3.5} textAnchor="end">
              {formatTick(t)}
            </text>
          </g>
        ))}
        {xTicks.map((i, k) => (
          <text
            key={`${i}-${k}`}
            className="lc-axis"
            x={x(i)}
            y={HEIGHT - 8}
            textAnchor={k === 0 ? "start" : k === xTicks.length - 1 ? "end" : "middle"}
          >
            {formatDay(dates[i], longRange)}
          </text>
        ))}

        {ordered.map((s) => (
          <g
            key={s.key}
            className={`lc-series ${s.colorClass} ${highlight && highlight !== s.key ? "is-dim" : ""}`}
          >
            <polyline
              className="lc-line"
              points={s.values.map((v, i) => `${x(i)},${y(v)}`).join(" ")}
            />
            {/* End marker: the latest value. */}
            <circle className="lc-dot" cx={x(n - 1)} cy={y(s.values[n - 1])} r={4} />
          </g>
        ))}

        {/* Direct labels at the line ends: identity never rests on colour alone. */}
        {series.map((s, k) => {
          const moved = Math.abs(labelYs[k] - endYs[k]) > 2;
          return (
            <g
              key={s.key}
              className={`lc-end ${s.colorClass} ${highlight && highlight !== s.key ? "is-dim" : ""}`}
            >
              {moved && (
                <line
                  className="lc-leader"
                  x1={x(n - 1) + 5}
                  y1={endYs[k]}
                  x2={x(n - 1) + 11}
                  y2={labelYs[k]}
                />
              )}
              <text className="lc-end-label" x={x(n - 1) + 13} y={labelYs[k] + 3.5}>
                {s.shortLabel}
              </text>
            </g>
          );
        })}

        {hover !== null && (
          <g className="lc-hover">
            <line className="lc-crosshair" x1={x(hover)} x2={x(hover)} y1={M.top} y2={M.top + plotH} />
            {series.map((s) => (
              <circle
                key={s.key}
                className={`lc-dot ${s.colorClass}`}
                cx={x(hover)}
                cy={y(s.values[hover])}
                r={4.5}
              />
            ))}
          </g>
        )}
      </svg>

      {hover !== null && (
        <div
          className="lc-tip"
          style={{
            left: x(hover),
            top: M.top,
            transform: tipOnLeft ? "translateX(calc(-100% - 12px))" : "translateX(12px)",
          }}
        >
          <div className="lc-tip-date">
            {parseIsoDate(dates[hover]).toLocaleDateString(undefined, {
              weekday: "short",
              day: "numeric",
              month: "short",
              year: "numeric",
            })}
          </div>
          {readout.map((s) => (
            <div key={s.key} className={`lc-tip-row ${s.colorClass}`}>
              <span className="lc-tip-key" />
              <strong>{formatValue(s.values[hover])}</strong>
              <span className="muted">{s.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
