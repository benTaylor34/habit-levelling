import { useEffect, useState } from "react";
import { api, errorMessage } from "./lib/tauri";
import { useBackHandler } from "./lib/back";
import type { IsoDate } from "./lib/types";
import { Analysis } from "./pages/Analysis";
import { Data } from "./pages/Data";
import { Dashboard } from "./pages/Dashboard";
import { HabitDetails } from "./pages/HabitDetails";
import "./App.css";

type View =
  | { page: "dashboard" }
  | { page: "analysis" }
  | { page: "data" }
  | { page: "details"; id: number };

function App() {
  const [view, setView] = useState<View>({ page: "dashboard" });
  const [today, setToday] = useState<IsoDate | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Android Back: from any other page, go back to Today; on Today, exit.
  useBackHandler(() => setView({ page: "dashboard" }), view.page !== "dashboard");

  // Ask Rust what "today" is; re-check when the window regains focus so
  // leaving the app open past midnight rolls over to the new day.
  useEffect(() => {
    const refresh = () =>
      api.getToday().then(setToday).catch((e) => setError(errorMessage(e)));
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, []);

  if (error) return <main className="container"><p className="error">{error}</p></main>;
  if (!today) return <main className="container"><p className="muted">Loading…</p></main>;

  return (
    <main className="container">
      <nav className="app-bar">
        <button className="brand" onClick={() => setView({ page: "dashboard" })}>
          <span className="brand-mark" aria-hidden>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={3} strokeLinecap="round" strokeLinejoin="round">
              <path d="M4 18 10 12l4 4 6-8" />
            </svg>
          </span>
          <span className="brand-text">Habit Levelling</span>
        </button>
        <div className="nav-tabs">
          <button
            className={view.page === "dashboard" || view.page === "details" ? "active" : ""}
            aria-current={view.page === "dashboard" || view.page === "details" ? "page" : undefined}
            onClick={() => setView({ page: "dashboard" })}
          >
            Today
          </button>
          <button
            className={view.page === "analysis" ? "active" : ""}
            aria-current={view.page === "analysis" ? "page" : undefined}
            onClick={() => setView({ page: "analysis" })}
          >
            Analysis
          </button>
          <button
            className={view.page === "data" ? "active" : ""}
            aria-current={view.page === "data" ? "page" : undefined}
            onClick={() => setView({ page: "data" })}
          >
            Data
          </button>
        </div>
      </nav>
      {view.page === "data" ? (
        <Data />
      ) : view.page === "analysis" ? (
        <Analysis key={today} />
      ) : view.page === "dashboard" ? (
        <Dashboard
          key={today}
          today={today}
          onOpen={(id) => setView({ page: "details", id })}
        />
      ) : (
        <HabitDetails
          key={`${view.id}-${today}`}
          id={view.id}
          today={today}
          onBack={() => setView({ page: "dashboard" })}
        />
      )}
    </main>
  );
}

export default App;
