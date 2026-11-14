# Habit Levelling

A local-first desktop habit tracker. Complete habits daily, keep streaks alive,
and level up skills. Built with Tauri + Rust + SQLite (SQLx) and a React +
TypeScript frontend.

## Run

Prerequisites: Node, Rust (MSVC toolchain), and on Windows the
**Visual Studio Build Tools with the "Desktop development with C++" workload**
(Rust needs its `link.exe`).

```bash
npm install
npm run tauri dev
```

Rust tests (streaks, CRUD, completions, skill XP, persistence across restarts):

```bash
cd src-tauri && cargo test
```

> On Windows, run cargo from PowerShell/cmd, not Git Bash — Git Bash's
> `/usr/bin/link` shadows the MSVC linker.

Data lives in `habits.db` in the app data directory
(`%APPDATA%\com.ben.habit_levelling\` on Windows).

## Architecture

React only renders and calls Tauri commands; all logic lives in Rust.

| File | Role |
| --- | --- |
| `src-tauri/src/commands.rs` | Tauri commands (the frontend's only API); decides "today" in local time |
| `src-tauri/src/habits.rs` | Habit CRUD, completions, summaries, skill XP queries |
| `src-tauri/src/streaks.rs` | Pure current/longest streak calculation |
| `src-tauri/src/skills.rs` | XP → level curve |
| `src-tauri/src/db.rs` | SQLite pool + embedded migrations |
| `src-tauri/migrations/0001_initial.sql` | `habits`, `habit_completions` (`UNIQUE(habit_id, date)`) |
| `src/lib/tauri.ts` | Typed wrappers for the commands |
| `src/pages/` | Dashboard, HabitDetails |
| `src/components/` | HabitCard, HabitForm, Calendar, StreakBadge, SkillPanel |

Streaks and XP are never stored; they're derived from completion history.
Completions are stored as local calendar dates (`YYYY-MM-DD`). If today isn't
completed yet, the current streak still counts back from yesterday.
