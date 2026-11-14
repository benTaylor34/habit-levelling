import { useEffect, useState } from "react";
import { api, errorMessage } from "./lib/tauri";
import type { IsoDate } from "./lib/types";
import { Dashboard } from "./pages/Dashboard";
import { HabitDetails } from "./pages/HabitDetails";
import "./App.css";

type View = { page: "dashboard" } | { page: "details"; id: number };

function App() {
  const [view, setView] = useState<View>({ page: "dashboard" });
  const [today, setToday] = useState<IsoDate | null>(null);
  const [error, setError] = useState<string | null>(null);

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
      {view.page === "dashboard" ? (
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
