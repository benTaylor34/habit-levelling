import { useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { DailyQuest } from "../lib/types";
import { QuestPanel } from "../components/QuestPanel";

/** The Quests tab: today's three quests. Stats refresh when you go back to Today. */
export function Quests() {
  const [quests, setQuests] = useState<DailyQuest[] | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.getDailyQuests().then(setQuests).catch((e) => setError(errorMessage(e)));
  }, []);

  async function toggle(quest: DailyQuest) {
    setBusyId(quest.id);
    setError(null);
    try {
      setQuests(await (quest.completed ? api.undoQuest(quest.id) : api.completeQuest(quest.id)));
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <>
      <header className="page-header">
        <div>
          <p className="eyebrow">Quests</p>
          <h1>Daily quests</h1>
          <p className="muted description">
            Three new quests every day, each for a different stat and scaled to its level. Do them
            any time today; each one gives that stat XP.
          </p>
        </div>
      </header>

      {error && (
        <p className="error" onClick={() => setError(null)}>
          {error}
        </p>
      )}

      {quests === null ? (
        <p className="muted">Loading…</p>
      ) : (
        <div className="quests-page">
          <QuestPanel quests={quests} busyId={busyId} onToggle={toggle} />
        </div>
      )}
    </>
  );
}
