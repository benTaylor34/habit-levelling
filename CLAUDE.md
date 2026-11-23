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
- **The Claude desktop app virtualises AppData.** It's a packaged app, so anything launched from a Claude session (`tauri dev`, the exe, tests, Python) reads and writes `%APPDATA%` inside `%LOCALAPPDATA%\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\` instead.
  - Data created in an app started from Claude is invisible to the same app opened from Explorer, and the reverse.
  - From Claude you can't see the user's real data. Don't tell them their data is safe based on a read from inside Claude.
  - To move data across, export a backup to `Documents`, which isn't virtualised.
- `graphify-out/` is a stale generated graph. Ignore it.

## Releases

Bump the version in the root `Cargo.toml`, `src-tauri/tauri.conf.json` and `package.json`, then push a `v*` tag. The Release workflow builds the exe and APK into a draft GitHub release; write the notes, then publish. CI runs the tests on every push.

## Working style

- **Commit and push only when asked.** Keep unrelated changes out of commits, and check for edits made outside the session before committing.
- **Don't leave the Android emulator running.** Shut it down (`adb emu kill`) once testing is done; it holds about 1 GB of RAM. Start it again when needed.
- **Be token-efficient.** Use targeted reads and greps and small edits. Verify with tests, logs or DB queries before taking screenshots, and keep screenshots small.
- **Verify before claiming.** Run the tests and type-check after changes, and check real builds when packaging is affected.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships. It is local only (git-ignored) and built from code with no API cost. graphify is installed in its own venv, not on PATH, so call it by full path: `$env:USERPROFILE\.graphify-venv\Scripts\graphify.exe` (PowerShell). A personal PreToolUse hook in `.claude/settings.local.json` reminds you before searches and reads.

Rules:
- When graphify-out/graph.json exists, run `graphify query "<question>"` before your first grep, search or source-file read for a codebase question, and put this rule in every subagent prompt that explores code. Read raw files only for lines the graph or an earlier read already located. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- **Don't trust impact queries alone.** `graphify affected "<symbol>"` misses calls across crates. For example, it doesn't see `src-tauri` calling into `habit_core`. Before changing a function's behaviour or signature, confirm its callers with a Grep limited to `crates/`, `src-tauri/src` and `src/`. Never search the whole repo: `target/` and `node_modules/` are huge.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost). It isn't refreshed automatically; `graphify hook install` would add git hooks that do this after each commit.
