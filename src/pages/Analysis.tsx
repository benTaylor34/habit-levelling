import { useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { StatHistory, StatKey } from "../lib/types";
import { statInfo } from "../lib/types";
import { LineChart } from "../components/LineChart";
import { BoltIcon, ChartIcon, TrophyIcon } from "../components/Icons";

const RANGES: { label: string; days: number | null }[] = [
  { label: "30 days", days: 30 },
  { label: "90 days", days: 90 },
  { label: "1 year", days: 365 },
  { label: "All time", days: null },
];

type Metric = "level" | "xp";

function Segmented<T>({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: { label: string; value: T }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="segmented" role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <button
          key={o.label}
          type="button"
          role="radio"
          aria-checked={o.value === value}
          className={o.value === value ? "selected" : ""}
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Analysis() {
  const [days, setDays] = useState<number | null>(30);
  const [metric, setMetric] = useState<Metric>("level");
  const [history, setHistory] = useState<StatHistory | null>(null);
  const [hidden, setHidden] = useState<Set<StatKey>>(new Set());
  const [highlight, setHighlight] = useState<StatKey | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let stale = false;
    api
      .getStatHistory(days)
      .then((h) => !stale && setHistory(h))
      .catch((e) => setError(errorMessage(e)));
    return () => {
      stale = true;
    };
  }, [days]);

  function toggle(stat: StatKey) {
    setHidden((h) => {
      const next = new Set(h);
      if (next.has(stat)) next.delete(stat);
      else next.add(stat);
      return next;
    });
  }

  const series = history?.series ?? [];
  const xpEarned = series.reduce((sum, s) => sum + s.xpGained, 0);
  const levelsGained = series.reduce((sum, s) => sum + (s.endLevel - s.startLevel), 0);
  const top = series.reduce<(typeof series)[number] | null>(
    (best, s) => (s.xpGained > (best?.xpGained ?? 0) ? s : best),
    null,
  );
  const rangeLabel = RANGES.find((r) => r.days === days)!.label.toLowerCase();
  const hasData = series.some((s) => s.xp[s.xp.length - 1] > 0);

  const chartSeries = series
    .filter((s) => !hidden.has(s.stat))
    .map((s) => ({
      key: s.stat,
      label: statInfo(s.stat).name,
      shortLabel: statInfo(s.stat).abbr,
      colorClass: `stat-${s.stat}`,
      values: metric === "level" ? s.level : s.xp,
    }));
  const formatValue =
    metric === "level"
      ? (v: number) => (Number.isInteger(v) ? String(v) : v.toFixed(1))
      : (v: number) => v.toLocaleString();
  const formatTick =
    metric === "level"
      ? formatValue
      : (v: number) => (v >= 1000 ? `${Number((v / 1000).toFixed(1))}k` : String(v));

  return (
    <>
      <header className="page-header">
        <div>
          <p className="eyebrow">Analysis</p>
          <h1>Stat progress</h1>
        </div>
      </header>

      <div className="filters">
        <Segmented
          label="Time range"
          options={RANGES.map((r) => ({ label: r.label, value: r.days }))}
          value={days}
          onChange={setDays}
        />
        <Segmented
          label="Measure"
          options={[
            { label: "Level", value: "level" as Metric },
            { label: "Total XP", value: "xp" as Metric },
          ]}
          value={metric}
          onChange={setMetric}
        />
      </div>

      {error && <p className="error">{error}</p>}

      {history && (
        <>
          <div className="stats analysis-tiles">
            <div className="stat">
              <span className="stat-icon accent">
                <BoltIcon size={18} />
              </span>
              <span className="stat-value">{xpEarned.toLocaleString()}</span>
              <span className="stat-label">XP earned, {rangeLabel}</span>
            </div>
            <div className="stat">
              <span className="stat-icon success">
                <ChartIcon size={18} />
              </span>
              <span className="stat-value">{Math.abs(levelsGained)}</span>
              <span className="stat-label">
                {levelsGained < 0 ? "Levels lost" : "Levels gained"}
              </span>
            </div>
            <div className="stat">
              <span className="stat-icon gold">
                <TrophyIcon size={18} />
              </span>
              <span className="stat-value">{top ? statInfo(top.stat).name : "—"}</span>
              <span className="stat-label">Most trained</span>
            </div>
          </div>

          <section className="panel">
            <div className="panel-header">
              <h2>{metric === "level" ? "Stat levels over time" : "Total XP over time"}</h2>
              <span className="muted small">Click a stat to show or hide it</span>
            </div>
            <div className="legend" onMouseLeave={() => setHighlight(null)}>
              {series.map((s) => {
                const off = hidden.has(s.stat);
                return (
                  <button
                    key={s.stat}
                    type="button"
                    className={`legend-item stat-${s.stat} ${off ? "off" : ""}`}
                    aria-pressed={!off}
                    onClick={() => toggle(s.stat)}
                    onMouseEnter={() => setHighlight(off ? null : s.stat)}
                    onFocus={() => setHighlight(off ? null : s.stat)}
                    onBlur={() => setHighlight(null)}
                  >
                    <span className="legend-key" />
                    {statInfo(s.stat).name}
                  </button>
                );
              })}
            </div>
            {!hasData ? (
              <p className="muted small chart-empty">
                Complete habits that train a stat to see your progress here.
              </p>
            ) : chartSeries.length === 0 ? (
              <p className="muted small chart-empty">All stats are hidden.</p>
            ) : (
              <LineChart
                dates={history.dates}
                series={chartSeries}
                yMin={metric === "level" ? 1 : 0}
                formatValue={formatValue}
                formatTick={formatTick}
                highlight={highlight}
                ariaLabel={`${metric === "level" ? "Stat levels" : "Total XP"} per day, ${rangeLabel}. Exact figures are in the table below.`}
              />
            )}
          </section>

          <section className="panel">
            <div className="panel-header">
              <h2>Summary</h2>
              <span className="muted small">{rangeLabel}</span>
            </div>
            <table className="data-table">
              <thead>
                <tr>
                  <th>Stat</th>
                  <th className="num">Level</th>
                  <th className="num">Change</th>
                  <th className="num">XP earned</th>
                  <th className="num">Lost to decay</th>
                  <th className="num">Total XP</th>
                </tr>
              </thead>
              <tbody>
                {series.map((s) => {
                  const change = s.endLevel - s.startLevel;
                  return (
                    <tr key={s.stat} className={`stat-${s.stat}`}>
                      <td>
                        <span className="legend-key" /> {statInfo(s.stat).name}
                      </td>
                      <td className="num">
                        {s.startLevel === s.endLevel ? s.endLevel : `${s.startLevel} → ${s.endLevel}`}
                      </td>
                      <td className={`num ${change > 0 ? "up" : change < 0 ? "down" : "muted"}`}>
                        {change > 0 ? `+${change}` : change < 0 ? `−${-change}` : "–"}
                      </td>
                      <td className="num">{s.xpGained.toLocaleString()}</td>
                      <td className={`num ${s.xpDecayed > 0 ? "down" : "muted"}`}>
                        {s.xpDecayed > 0 ? `−${s.xpDecayed.toLocaleString()}` : "–"}
                      </td>
                      <td className="num muted">{s.xp[s.xp.length - 1].toLocaleString()}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </section>
        </>
      )}
    </>
  );
}
