# Habit Levelling: notes for Claude

Local-first habit tracker for Windows and Android: Tauri 2 + Rust + SQLite (SQLx) + React/TypeScript. The user-facing details are in README.md; this file is the working guide.

## Layout

- `crates/core/` (`habit_core`): the game rules. These are pure functions with no DB, UI, clock or platform code, and depend only on `chrono` and `serde`. sqlx derives are behind the optional `sqlx` feature.
  - stats/XP/levels: `stats.rs`
  - streaks: `streaks.rs`
  - decay: `decay.rs`
  - luck: `luck.rs`
  - charts: `history.rs`
  - heatmap: `activity.rs`
  - stat suggestions: `recommend.rs`
  - shared types: `models.rs`
- `src-tauri/`: the app.
  - `habits.rs`: SQLite queries
  - `backup.rs`: JSON export/restore and daily auto-backups
  - `commands.rs`: the only API the UI calls, and the place that decides "today"
  - `db.rs` and `migrations/`
- `src-tauri/gen/android/`: the Android shell, committed. `MainActivity.kt` handles the status bar and keyboard insets and Android Back.
- `src/`: the React UI, shared by both platforms.
  - `src/platform/` holds everything platform-specific: file pickers, dialogs, Back and `isAndroid`. Pages must not import `@tauri-apps/plugin-*` or sniff the platform directly.
- The root `Cargo.toml` is the workspace. It holds the shared version, dependency versions and the release profile. Build output goes to the root `target/`.

## Rules that must hold

- **Business logic lives in Rust.** Put new rules in `habit_core`, with unit tests beside them. React only renders and calls commands.
- **Derived, never stored.** XP, levels, streaks, decay and history are recomputed from completion logs. The only stored randomness is `luck_rolls`.
- **Dates are local calendar dates** (`YYYY-MM-DD`), with one completion per habit per day. In TypeScript, never `new Date("YYYY-MM-DD")`; use `parseIsoDate` / `toIsoDate`.
- **Schema changes are new migration files.** Never edit an applied migration, because SQLx checksums them. Keep `*.sql` LF-only (see `.gitattributes`).
- **Stat colours** come from a validated categorical palette (comment in `App.css`). Don't change them casually; text and identity must never rely on colour alone.
- **Dark theme only.**

## Commands (run from the repo root)

```bash
cargo test -p habit_core     # game rules only, fast
cargo test --workspace       # everything (76+ tests)
npx tsc --noEmit             # frontend type-check
npm run tauri dev            # desktop dev app
npm run tauri build -- --no-bundle   # target/release/habit_levelling.exe
```

Android: `npm run tauri android build -- --apk --target aarch64 --target armv7 --target x86_64`. On Windows this needs **Developer Mode** for a symlink step. Without it, the build fails after compiling. The workaround is to copy `target/<triple>/release/libhabit_levelling_lib.so` into `gen/android/app/src/main/jniLibs/<abi>/` and run `gradlew assemble<Flavor>Release -x rustBuild<Flavor>Release`.

## Environment quirks (Windows)

- **Run cargo from PowerShell, not Git Bash.** Git Bash's `/usr/bin/link` shadows the MSVC linker.
- **Inline scripts in Bash heredocs lose backslashes.** Write scripts to a file, or use the Edit tool instead.
- **Release signing** uses `src-tauri/gen/android/key.properties` (git-ignored) and the keystore in `%USERPROFILE%\.android-keys\`. Never print, commit or upload these.
- **Data locations:**
  - Windows: `%APPDATA%\com.habitlevelling.app\habits.db`
  - Android: app-private storage
- `graphify-out/` is a stale generated graph. Ignore it.

## Releases

Bump the version in the root `Cargo.toml`, `src-tauri/tauri.conf.json` and `package.json`, then push a `v*` tag. The Release workflow builds the exe and APK into a draft GitHub release; write the notes, then publish. CI runs the tests on every push.

## Working style

- **Commit and push only when asked.** Keep unrelated changes out of commits, and check for edits made outside the session before committing.
- **Be token-efficient.** Use targeted reads and greps and small edits. Verify with tests, logs or DB queries before taking screenshots, and keep screenshots small.
- **Verify before claiming.** Run the tests and type-check after changes, and check real builds when packaging is affected.
