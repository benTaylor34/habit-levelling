import { useEffect, useState } from "react";
import { api, errorMessage } from "../lib/tauri";
import type { BackupInfo, BackupSummary } from "../lib/types";
import { parseIsoDate } from "../lib/dates";
import { canReveal, displayName, isAndroid, openTextFile, revealInFolder, saveTextFile } from "../platform";
import { CheckIcon, DownloadIcon, FolderIcon, UploadIcon, WarningIcon } from "../components/Icons";

const BACKUP_FILTER = [{ name: "Habit Levelling backup", extensions: ["json"] }];

const plural = (n: number, word: string) => `${n.toLocaleString()} ${word}${n === 1 ? "" : "s"}`;

function shortDate(iso: string) {
  return parseIsoDate(iso).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
}

/** "2026-11-16 21:30:00" (local time) -> "16 Nov 2026, 21:30". */
function formatExportedAt(s: string) {
  const [date, time] = s.split(" ");
  return time ? `${shortDate(date)}, ${time.slice(0, 5)}` : s;
}

function describe(s: BackupSummary) {
  const range =
    s.firstDate && s.lastDate ? ` from ${shortDate(s.firstDate)} to ${shortDate(s.lastDate)}` : "";
  return `${plural(s.habits, "habit")} and ${plural(s.completions, "daily log")}${range}`;
}

function RevealButton({ path, label = "Show in folder" }: { path: string; label?: string }) {
  if (!canReveal(path)) return null;
  return (
    <button type="button" className="btn ghost small-btn" onClick={() => revealInFolder(path)}>
      <FolderIcon size={15} /> {label}
    </button>
  );
}

export function Data() {
  const [info, setInfo] = useState<BackupInfo | null>(null);
  /** Where the backup went, plus the name we suggested (Android's content:// paths hide it). */
  const [exported, setExported] = useState<{ path: string; name: string } | null>(null);
  const [pending, setPending] = useState<{ json: string; fileName: string; summary: BackupSummary } | null>(
    null,
  );
  const [restored, setRestored] = useState<{ summary: BackupSummary; safetyBackup: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadInfo = () => api.getBackupInfo().then(setInfo).catch((e) => setError(errorMessage(e)));
  useEffect(() => {
    loadInfo();
  }, []);

  // Rust builds the backup; the system Save dialog decides where it goes.
  async function exportNow() {
    setBusy(true);
    setError(null);
    try {
      const { fileName: suggested, json } = await api.exportBackup();
      const path = await saveTextFile(suggested, json, BACKUP_FILTER);
      if (!path) return; // cancelled
      setExported({ path, name: suggested });
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  // Pick a file and let Rust check it before anything is replaced.
  async function chooseFile() {
    setError(null);
    setRestored(null);
    try {
      const file = await openTextFile(BACKUP_FILTER);
      if (!file) return; // cancelled
      // Android doesn't reveal the file's name; it's then just "This backup".
      setPending({ json: file.text, fileName: file.name ?? "", summary: await api.inspectBackup(file.text) });
    } catch (err) {
      setPending(null);
      setError(errorMessage(err));
    }
  }

  async function restoreNow() {
    if (!pending) return;
    setBusy(true);
    setError(null);
    try {
      setRestored(await api.restoreBackup(pending.json));
      setPending(null);
      loadInfo();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <header className="page-header">
        <div>
          <p className="eyebrow">Data</p>
          <h1>Backup & restore</h1>
          <p className="muted description">
            Your habits are stored only on this device. Back them up to keep them safe or move them
            to another device.
          </p>
        </div>
      </header>

      {error && (
        <p className="error" onClick={() => setError(null)}>
          {error}
        </p>
      )}

      <div className="data-grid">
        <section className="panel data-card">
          <span className="stat-icon accent">
            <DownloadIcon size={18} />
          </span>
          <div className="data-body">
            <h2>Export a backup</h2>
            <p className="muted small">
              Saves all your habits and daily logs as one file, wherever you choose. Your XP,
              levels and streaks are rebuilt from it.
            </p>
            <div className="data-actions">
              <button type="button" className="btn primary" disabled={busy} onClick={exportNow}>
                <DownloadIcon size={16} /> Export backup…
              </button>
            </div>
            {exported && (
              <div className="data-result success-result">
                <CheckIcon size={15} strokeWidth={2.6} />
                <span className="path">
                  Saved {canReveal(exported.path) ? `to ${exported.path}` : `as ${exported.name}`}
                </span>
                <RevealButton path={exported.path} />
              </div>
            )}
          </div>
        </section>

        <section className="panel data-card">
          <span className="stat-icon gold">
            <UploadIcon size={18} />
          </span>
          <div className="data-body">
            <h2>Restore from a backup</h2>
            <p className="muted small">
              Replaces everything in the app with a backup's contents. Your current data is saved
              first, so you can always undo it.
            </p>
            {!pending && (
              <div className="data-actions">
                <button type="button" className="btn" disabled={busy} onClick={chooseFile}>
                  <UploadIcon size={16} /> Choose backup file…
                </button>
              </div>
            )}

            {pending && (
              <div className="confirm-box" role="alertdialog" aria-label="Confirm restore">
                <div className="confirm-title">
                  <WarningIcon size={16} strokeWidth={2.4} /> Replace all current data?
                </div>
                <p className="small">
                  {pending.fileName ? <strong>{pending.fileName}</strong> : "This backup"} was made on{" "}
                  {formatExportedAt(pending.summary.exportedAt)} and holds {describe(pending.summary)}.
                </p>
                <div className="data-actions">
                  <button type="button" className="btn ghost" disabled={busy} onClick={() => setPending(null)}>
                    Cancel
                  </button>
                  <button type="button" className="btn danger-solid" disabled={busy} onClick={restoreNow}>
                    Replace my data
                  </button>
                </div>
              </div>
            )}

            {restored && (
              <div className="data-result success-result">
                <CheckIcon size={15} strokeWidth={2.6} />
                <span>
                  Restored {describe(restored.summary)}. Your previous data was saved as a safety
                  backup.
                </span>
                <RevealButton path={restored.safetyBackup} />
              </div>
            )}
          </div>
        </section>

        <section className="panel data-card wide">
          <span className="stat-icon success">
            <FolderIcon size={18} />
          </span>
          <div className="data-body">
            <h2>Automatic backups</h2>
            <p className="muted small">
              A backup is saved automatically the first time you open the app each day. The last{" "}
              {info?.autoBackupsKept ?? 14} are kept, along with the safety copies made before each
              restore.
              {isAndroid &&
                " On Android these stay inside the app, so export a backup to keep a copy you can move."}
            </p>
            {info && (
              <dl className="data-facts">
                <div>
                  <dt>Latest</dt>
                  <dd>{info.latestAuto ? (displayName(info.latestAuto) ?? info.latestAuto) : "None yet (made once you have a habit)"}</dd>
                </div>
                <div>
                  <dt>Kept</dt>
                  <dd>{plural(info.autoBackups, "backup")}</dd>
                </div>
                <div>
                  <dt>Your data</dt>
                  <dd className="path">{info.dataFile}</dd>
                </div>
              </dl>
            )}
            {info && (
              <div className="data-actions">
                <RevealButton path={info.latestAuto ?? info.dataFile} label="Open backups folder" />
              </div>
            )}
          </div>
        </section>
      </div>
    </>
  );
}
