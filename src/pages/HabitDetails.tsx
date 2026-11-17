import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import { confirm } from "../platform";
import type { CompletionInput, HabitDetails as Details, IsoDate, LuckWin } from "../lib/types";
import { difficultyLabel, statInfo } from "../lib/types";
import { formatMinutes, parseIsoDate } from "../lib/dates";
import { Calendar } from "../components/Calendar";
import { HabitForm } from "../components/HabitForm";
import { LogForm } from "../components/LogForm";
import { LuckToast } from "../components/LuckToast";
import {
  ArrowLeftIcon,
  BoltIcon,
  CheckIcon,
  ClockIcon,
  FlameIcon,
  PencilIcon,
  TrashIcon,
  TrophyIcon,
  UndoIcon,
  WarningIcon,
} from "../components/Icons";

interface Props {
  id: number;
  today: IsoDate;
  onBack: () => void;
}

export function HabitDetails({ id, today, onBack }: Props) {
  const [details, setDetails] = useState<Details | null>(null);
  const [editing, setEditing] = useState(false);
  const [logging, setLogging] = useState(false);
  const [luck, setLuck] = useState<LuckWin | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setDetails(await api.getHabitDetails(id));
    } catch (e) {
      setError(errorMessage(e));
    }
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  const back = (
    <button className="btn ghost back" onClick={onBack}>
      <ArrowLeftIcon size={16} /> All habits
    </button>
  );

  if (!details) {
    return (
      <>
        {back}
        {error ? <p className="error">{error}</p> : <p className="muted">Loading…</p>}
      </>
    );
  }

  const { summary, completions } = details;

  async function logToday(log: CompletionInput) {
    const result = await api.completeToday(id, log);
    setLogging(false);
    setLuck(result.luck);
    await load();
  }

  async function undo() {
    setBusy(true);
    try {
      await api.undoToday(id);
      await load();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    const ok = await confirm(
      `Delete "${summary.name}" and its entire completion history? This can't be undone.`,
      { title: "Delete habit", okLabel: "Delete", warning: true },
    );
    if (!ok) return;
    try {
      await api.deleteHabit(id);
      onBack();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  const recent = [...completions].reverse().slice(0, 8);

  return (
    <>
      {back}

      <header className="page-header">
        <div className="title-block">
          {summary.primaryStat && (
            <p className="eyebrow">
              {statInfo(summary.primaryStat).name}
              {summary.secondaryStat && (
                <span className="eyebrow-secondary"> · {statInfo(summary.secondaryStat).name}</span>
              )}
            </p>
          )}
          <h1>{summary.name}</h1>
          {summary.description && <p className="muted description">{summary.description}</p>}
          {summary.needsAttention && (
            <p className="idle-banner">
              <WarningIcon size={14} strokeWidth={2.4} />
              {summary.totalCompletions === 0
                ? `Not started in ${summary.idleDays} days.`
                : `Not done in ${summary.idleDays} days.`}{" "}
              {summary.primaryStat && "Its stats fade after a week unless another habit trains them."}
            </p>
          )}
        </div>
        <div className="actions">
          <button
            className="btn icon"
            onClick={() => setEditing(true)}
            title="Edit habit"
            aria-label="Edit habit"
          >
            <PencilIcon size={16} />
          </button>
          <button
            className="btn icon danger"
            onClick={remove}
            title="Delete habit"
            aria-label="Delete habit"
          >
            <TrashIcon size={16} />
          </button>
          {summary.todayLog ? (
            <>
              <button
                className="btn icon"
                disabled={busy}
                onClick={undo}
                title="Undo today"
                aria-label="Undo today"
              >
                <UndoIcon size={16} />
              </button>
              <button
                className="btn success"
                onClick={() => setLogging(true)}
                title="Edit today's log"
              >
                <CheckIcon size={16} strokeWidth={2.5} />
                Done · +{summary.todayLog.xp} XP
              </button>
            </>
          ) : (
            <button className="btn primary" onClick={() => setLogging(true)}>
              <CheckIcon size={16} strokeWidth={2.5} />
              Complete today
            </button>
          )}
        </div>
      </header>

      {error && <p className="error">{error}</p>}

      <div className="stats">
        <div className="stat">
          <span className="stat-icon fire">
            <FlameIcon size={18} />
          </span>
          <span className="stat-value">{summary.currentStreak}</span>
          <span className="stat-label">Current streak</span>
        </div>
        <div className="stat">
          <span className="stat-icon gold">
            <TrophyIcon size={18} />
          </span>
          <span className="stat-value">{summary.longestStreak}</span>
          <span className="stat-label">Longest streak</span>
        </div>
        <div className="stat">
          <span className="stat-icon accent">
            <BoltIcon size={18} />
          </span>
          <span className="stat-value">{summary.totalXp}</span>
          <span className="stat-label">XP earned</span>
        </div>
        <div className="stat">
          <span className="stat-icon success">
            <ClockIcon size={18} />
          </span>
          <span className="stat-value">{formatMinutes(summary.totalMinutes) || "0 min"}</span>
          <span className="stat-label">
            Time over {summary.totalCompletions} {summary.totalCompletions === 1 ? "day" : "days"}
          </span>
        </div>
      </div>

      <div className="details-grid">
        <section className="panel">
          <Calendar completions={completions.map((c) => c.date)} today={today} />
        </section>
        <section className="panel">
          <div className="panel-header">
            <h2>Recent activity</h2>
          </div>
          {recent.length === 0 ? (
            <p className="muted small">No completions yet.</p>
          ) : (
            <ul className="history">
              {recent.map((c) => (
                <li key={c.date}>
                  <span className="history-dot">
                    <CheckIcon size={12} strokeWidth={3} />
                  </span>
                  <span className="history-main">
                    <span>
                      {c.date === today
                        ? "Today"
                        : parseIsoDate(c.date).toLocaleDateString(undefined, {
                            weekday: "short",
                            day: "numeric",
                            month: "short",
                          })}
                    </span>
                    <span className="muted tiny">
                      {[formatMinutes(c.minutes), difficultyLabel(c.difficulty), c.streak > 1 && `day ${c.streak}`]
                        .filter(Boolean)
                        .join(" · ")}
                    </span>
                  </span>
                  <span className="xp-gain">+{c.xp} XP</span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>

      {logging && (
        <LogForm
          habit={summary}
          initial={summary.todayLog}
          onCancel={() => setLogging(false)}
          onSubmit={logToday}
        />
      )}

      {luck && <LuckToast win={luck} habitName={summary.name} onClose={() => setLuck(null)} />}

      {editing && (
        <HabitForm
          initial={summary}
          onCancel={() => setEditing(false)}
          onSubmit={async (input) => {
            await api.updateHabit(id, input);
            setEditing(false);
            await load();
          }}
        />
      )}
    </>
  );
}
