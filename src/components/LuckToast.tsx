import { useEffect } from "react";
import type { LuckWin } from "../lib/types";
import { CloverIcon } from "./Icons";

interface Props {
  win: LuckWin;
  habitName: string;
  onClose: () => void;
}

/** Celebration shown when a streak milestone's Luck roll comes up lucky. */
export function LuckToast({ win, habitName, onClose }: Props) {
  useEffect(() => {
    const t = setTimeout(onClose, 7000);
    return () => clearTimeout(t);
  }, [onClose]);

  return (
    <div className="toast luck-toast" role="status" onClick={onClose}>
      <span className="toast-icon">
        <CloverIcon size={20} />
      </span>
      <div>
        <div className="toast-title">Lucky! +{win.xp} Luck XP</div>
        <div className="muted small">
          Your {win.streak}-day {habitName} streak caught fortune's eye.
        </div>
      </div>
    </div>
  );
}
