import type { StatKey } from "../lib/types";
import { STATS, statInfo } from "../lib/types";

interface Props {
  label: string;
  hint: string;
  value: StatKey | null;
  onChange: (stat: StatKey | null) => void;
  /** A stat that can't be picked here (the other slot's choice). */
  exclude?: StatKey | null;
  disabled?: boolean;
}

/** Row of the six stats as toggle buttons, with the chosen stat's description. */
export function StatPicker({ label, hint, value, onChange, exclude, disabled }: Props) {
  return (
    <div className={`field ${disabled ? "is-disabled" : ""}`}>
      <span className="field-label">
        {label} <span className="muted">{hint}</span>
      </span>
      <div className="stat-picker" role="radiogroup" aria-label={label}>
        {STATS.map((s) => (
          <button
            key={s.key}
            type="button"
            role="radio"
            aria-checked={value === s.key}
            className={`stat-option stat-${s.key} ${value === s.key ? "selected" : ""}`}
            disabled={disabled || exclude === s.key}
            title={s.description}
            // Clicking the selected stat again clears it.
            onClick={() => onChange(value === s.key ? null : s.key)}
          >
            <span className="stat-abbr">{s.abbr}</span>
            <span className="stat-option-name">{s.name}</span>
          </button>
        ))}
      </div>
      <p className="muted tiny stat-help">
        {value ? statInfo(value).description : disabled ? "Pick a primary stat first." : "None selected."}
      </p>
    </div>
  );
}
