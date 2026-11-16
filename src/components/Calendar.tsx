import { useState } from "react";
import type { IsoDate } from "../lib/types";
import { parseIsoDate, toIsoDate } from "../lib/dates";
import { ChevronLeftIcon, ChevronRightIcon } from "./Icons";

interface Props {
  completions: IsoDate[];
  today: IsoDate;
}

const WEEKDAYS = ["M", "T", "W", "T", "F", "S", "S"];

/** Month grid (Monday first) with completed days highlighted. */
export function Calendar({ completions, today }: Props) {
  const todayDate = parseIsoDate(today);
  const [month, setMonth] = useState(
    new Date(todayDate.getFullYear(), todayDate.getMonth(), 1),
  );
  const done = new Set(completions);

  const year = month.getFullYear();
  const m = month.getMonth();
  const daysInMonth = new Date(year, m + 1, 0).getDate();
  const leadingBlanks = (month.getDay() + 6) % 7; // Monday = 0

  const cells: (Date | null)[] = [
    ...Array<null>(leadingBlanks).fill(null),
    ...Array.from({ length: daysInMonth }, (_, i) => new Date(year, m, i + 1)),
  ];

  const completedThisMonth = cells.filter((c) => c && done.has(toIsoDate(c))).length;
  const isCurrentMonth =
    year === todayDate.getFullYear() && m === todayDate.getMonth();

  return (
    <div className="calendar">
      <div className="calendar-header">
        <div>
          <h2>{month.toLocaleDateString(undefined, { month: "long", year: "numeric" })}</h2>
          <div className="muted small">
            {completedThisMonth} {completedThisMonth === 1 ? "day" : "days"} completed
          </div>
        </div>
        <div className="calendar-nav">
          <button
            className="btn icon"
            aria-label="Previous month"
            onClick={() => setMonth(new Date(year, m - 1, 1))}
          >
            <ChevronLeftIcon size={16} />
          </button>
          <button
            className="btn icon"
            aria-label="Next month"
            disabled={isCurrentMonth}
            onClick={() => setMonth(new Date(year, m + 1, 1))}
          >
            <ChevronRightIcon size={16} />
          </button>
        </div>
      </div>
      <div className="calendar-grid">
        {WEEKDAYS.map((d, i) => (
          <div key={i} className="calendar-weekday">
            {d}
          </div>
        ))}
        {cells.map((date, i) => {
          if (!date) return <div key={i} />;
          const iso = toIsoDate(date);
          const classes = ["calendar-day"];
          if (done.has(iso)) classes.push("completed");
          if (iso === today) classes.push("today");
          if (iso > today) classes.push("future");
          return (
            <div key={i} className={classes.join(" ")} title={iso}>
              {date.getDate()}
            </div>
          );
        })}
      </div>
    </div>
  );
}
