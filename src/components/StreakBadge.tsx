import { FlameIcon } from "./Icons";

interface Props {
  days: number;
  /** Compact form shows just the flame and number. */
  compact?: boolean;
}

export function StreakBadge({ days, compact = false }: Props) {
  return (
    <span
      className={`streak-badge ${days > 0 ? "active" : ""}`}
      title={`${days} day streak`}
    >
      <FlameIcon size={14} />
      <span className="streak-num">{days}</span>
      {!compact && <span className="streak-label">{days === 1 ? "day" : "days"}</span>}
    </span>
  );
}
