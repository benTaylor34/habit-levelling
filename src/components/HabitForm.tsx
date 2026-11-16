import { useEffect, useState } from "react";
import type { Habit, HabitInput } from "../lib/types";
import { SUGGESTED_SKILLS } from "../lib/types";

interface Props {
  initial?: Habit;
  onSubmit: (input: HabitInput) => Promise<void>;
  onCancel: () => void;
}

export function HabitForm({ initial, onSubmit, onCancel }: Props) {
  const [name, setName] = useState(initial?.name ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [skill, setSkill] = useState(initial?.skill ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSubmit({
        name,
        description: description || null,
        skill: skill || null,
      });
    } catch (err) {
      setError(String(err));
      setSaving(false);
    }
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onCancel();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <form className="modal" onSubmit={submit} onClick={(e) => e.stopPropagation()}>
        <div>
          <h2>{initial ? "Edit habit" : "New habit"}</h2>
          <p className="muted small">
            {initial ? "Update the details of this habit." : "Something you want to do every day."}
          </p>
        </div>

        <label>
          <span>Name</span>
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Read 20 pages"
            required
          />
        </label>

        <label>
          <span>Description <span className="muted">(optional)</span></span>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
          />
        </label>

        <label>
          <span>Skill <span className="muted">(optional)</span></span>
          <input
            list="skill-options"
            value={skill}
            onChange={(e) => setSkill(e.target.value)}
            placeholder="Knowledge"
          />
          <datalist id="skill-options">
            {SUGGESTED_SKILLS.map((s) => (
              <option key={s} value={s} />
            ))}
          </datalist>
        </label>

        {error && <p className="error">{error}</p>}

        <div className="modal-actions">
          <button type="button" className="btn ghost" onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" className="btn primary" disabled={saving || !name.trim()}>
            {initial ? "Save" : "Create"}
          </button>
        </div>
      </form>
    </div>
  );
}
