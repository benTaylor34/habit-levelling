// Native dialogs. The web view's own window.confirm() isn't reliable on
// Android, so confirmations go through the dialog plugin everywhere.
import { ask } from "@tauri-apps/plugin-dialog";

/** Ask a yes/no question in a native dialog; resolves true if confirmed. */
export function confirm(
  message: string,
  options: { title: string; okLabel: string; cancelLabel?: string; warning?: boolean },
): Promise<boolean> {
  return ask(message, {
    title: options.title,
    kind: options.warning ? "warning" : "info",
    okLabel: options.okLabel,
    cancelLabel: options.cancelLabel ?? "Cancel",
  });
}
