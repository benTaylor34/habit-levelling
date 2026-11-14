interface Props {
  days: number;
  label?: string;
}

export function StreakBadge({ days, label = "day streak" }: Props) {
  return (
    <span className={`streak-badge ${days > 0 ? "active" : ""}`}>
      <span aria-hidden>🔥</span> {days} {label}
    </span>
  );
}
