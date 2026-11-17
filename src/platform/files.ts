// Files the user picks, the same way on desktop and Android.
//
// Desktop pickers return normal paths. Android's return content:// URIs
// (e.g. "content://.../document/14"), which the fs plugin can read and write
// but which carry no usable file name and can't be shown in a folder.
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { isAndroid } from "./env";

export interface FileFilter {
  name: string;
  extensions: string[];
}

const isContentUri = (path: string) => path.startsWith("content:");

/**
 * Ask where to save `contents` (system Save dialog), then write it.
 * Returns where it went, or null if the user cancelled.
 */
export async function saveTextFile(
  suggestedName: string,
  contents: string,
  filters: FileFilter[],
): Promise<string | null> {
  const path = await save({ defaultPath: suggestedName, filters });
  if (!path) return null;
  await writeTextFile(path, contents);
  return path;
}

/**
 * Let the user pick a file (system Open dialog) and read it as text.
 * `name` is the file's name, or null when the platform doesn't reveal one.
 */
export async function openTextFile(
  filters: FileFilter[],
): Promise<{ path: string; name: string | null; text: string } | null> {
  const path = await open({ multiple: false, directory: false, filters });
  if (!path) return null;
  const text = await readTextFile(path);
  return { path, name: displayName(path), text };
}

/** A file's name for display, or null for Android content:// references. */
export function displayName(path: string): string | null {
  if (isContentUri(path)) return null;
  return path.split(/[\\/]/).pop() || null;
}

/** Whether `path` can be shown in a file explorer (desktop paths only). */
export const canReveal = (path: string) => !isAndroid && !isContentUri(path);

/** Open the folder containing `path`, with the file selected. */
export function revealInFolder(path: string): Promise<void> {
  return revealItemInDir(path);
}
