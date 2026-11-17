// Platform edge of the frontend.
//
// Everything that behaves differently on desktop and Android lives in this
// folder; pages and components import from here and never talk to Tauri
// plugins or sniff the platform themselves. The Android shell's native side
// is in src-tauri/gen/android/app/src/main/java/.../MainActivity.kt.
//
// - files.ts   Save/Open pickers, reading and writing the picked file, "Show in folder"
// - dialogs.ts Native confirmation dialogs
// - back.ts    Android's Back button (called from MainActivity.kt)

export { isAndroid } from "./env";

export { canReveal, displayName, openTextFile, revealInFolder, saveTextFile } from "./files";
export type { FileFilter } from "./files";
export { confirm } from "./dialogs";
export { useBackHandler } from "./back";
