// Android's Back button (platform edge). MainActivity.kt calls `window.__habitBack()` when
// Back is pressed: the most recently registered handler runs (close the open
// popup, leave the current page) and Android exits only if there is none.
import { useEffect, useRef } from "react";

const handlers: Array<() => void> = [];

declare global {
  interface Window {
    /** Returns true if Back was handled inside the app. */
    __habitBack?: () => boolean;
  }
}

window.__habitBack = () => {
  const top = handlers[handlers.length - 1];
  if (!top) return false;
  top();
  return true;
};

/**
 * While mounted (and `enabled`), Back runs `handler` instead of exiting.
 * Later registrations (e.g. a popup over a page) take priority.
 */
export function useBackHandler(handler: () => void, enabled = true) {
  const latest = useRef(handler);
  latest.current = handler;

  useEffect(() => {
    if (!enabled) return;
    const run = () => latest.current();
    handlers.push(run);
    return () => {
      const i = handlers.lastIndexOf(run);
      if (i >= 0) handlers.splice(i, 1);
    };
  }, [enabled]);
}
