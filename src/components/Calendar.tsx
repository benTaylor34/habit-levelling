import { useState } from "react";
import type { IsoDate } from "../lib/types";
import { parseIsoDate, toIsoDate } from "../lib/dates";

interface Props {
  completions: IsoDate[];
  today: IsoDate;
}

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

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
        <button className="ghost" onClick={() => setMonth(new Date(year, m - 1, 1))}>
          ‹
        </button>
        <div>
          <strong>
            {month.toLocaleDateString(undefined, { month: "long", year: "numeric" })}
          </strong>
          <div className="muted small">{completedThisMonth} days completed</div>
        </div>
        <button
          className="ghost"
          disabled={isCurrentMonth}
          onClick={() => setMonth(new Date(year, m + 1, 1))}
        >
          ›
        </button>
      </div>
      <div className="calendar-grid">
        {WEEKDAYS.map((d) => (
          <div key={d} className="calendar-weekday">
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
