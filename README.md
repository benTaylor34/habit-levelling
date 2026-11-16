# Habit Levelling

**Turn your daily habits into an RPG character sheet.** Log what you do each day, keep your streaks alive, and level up six D&D-style stats, but let them slide and they start to fade.

Habit Levelling is a local-first desktop app built with **Rust + Tauri**, **SQLite**, and a **React + TypeScript** interface. It runs entirely on your computer: no account, no server, no internet connection.

![The Today page: habits, stat radar and levels, and a year of activity (sample data)](docs/screenshots/today.png)

## Features

- **Daily habits.** Tick a habit off for today, log how long you spent and how hard it felt, and undo it if you mis-click.
- **Six stats.** Each habit trains a primary stat (full XP) and optionally a secondary stat (half XP): Strength, Dexterity, Constitution, Intelligence, Wisdom and Charisma.
- **Smart stat suggestions.** Type a habit like "Study maths" or "Weightlifting" and the app suggests stats for it (Intelligence + Wisdom, Strength + Constitution).
- **Levelling that gets harder.** Low levels come quickly; high levels take real commitment, using Pokémon's "medium fast" XP curve.
- **Streaks everywhere.** Each habit and each stat has a current and longest streak, and longer streaks earn more XP.
- **Use it or lose it.** Stats you stop training slowly lose XP, then levels. Habits you haven't done for a week get a warning.
- **A secret stat.** Luck can't be trained. It may appear when a streak hits a milestone…
- **Character overview.** A radar chart of your six stats, with an XP bar and streak for each one.
- **Activity heatmap.** A GitHub-style year of activity at a glance.
- **Analysis.** Stat levels or total XP over time for the last 30 days, 90 days, year or all time, plus a table of what changed.
- **Habit history.** A monthly calendar, recent activity and lifetime totals for every habit.

![The Analysis page: stat levels over time with a hover readout (sample data)](docs/screenshots/analysis.png)

## How the game works

### Earning XP

Each day you log a habit, its stats earn:

> **XP = (10 + minutes spent) × difficulty × streak bonus**

| Difficulty | Easy | Moderate | Hard | Very hard | Brutal |
| --- | --- | --- | --- | --- | --- |
| Multiplier | ×1 | ×1.5 | ×2 | ×2.5 | ×3 |

- **Streak bonus:** +5% for every day the habit's streak has run past the first, up to ×2 from day 21 onwards.
- **Primary and secondary stats:** the primary stat gets the full amount and the secondary stat gets half.

For example, 30 minutes of Hard studying on day 7 of a streak gives **(10 + 30) × 2 × 1.3 = 104 Intelligence** and **52 Wisdom**.

### Levels

Reaching level *n* takes 10 × (*n*³ − 1) XP in total, so each level costs more than the last:

| Level up | 1 → 2 | 2 → 3 | 5 → 6 | 10 → 11 | 50 → 51 |
| --- | --- | --- | --- | --- | --- |
| XP needed | 70 | 190 | 910 | 3,310 | 76,510 |

The XP bar resets after every level up. Levels stop at 100.

### Decay

| Days since a stat last earned XP | What happens each day |
| --- | --- |
| 0–6 | Nothing |
| 7–20 | It loses 10% of its current level's XP bar, down to 0. The level is kept. |
| 21+ | Once the bar is empty it drops a level (landing at 90% of that bar) and keeps draining. It never goes below level 1. |

Training the stat again stops the decay straight away.

### Luck

Whenever a habit's streak reaches a multiple of 7 days there's a 3% chance of +25 Luck XP. Every 30 days there's a 15% chance of +100. Luck stays hidden until you first win some.

## Download

Habit Levelling currently runs on **64-bit Windows 10 and 11**. Grab `habit_levelling.exe` from the [Releases](../../releases) page and run it. There's nothing to install.

> **"Windows protected your PC"?** The app isn't code-signed yet, so Windows warns about it the first time. Choose **More info → Run anyway**.

## Your data

Everything is saved in a single SQLite file on your own computer:

- Windows: `%APPDATA%\com.habitlevelling.app\habits.db`

Nothing is sent anywhere. To back up your progress, copy that file. Your data isn't synced between computers.

## Building from source

**You'll need:**

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/) (stable, MSVC toolchain on Windows)
- On Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload (Rust needs its linker)

**Run in development mode:**

```bash
npm install
npm run tauri dev
```

**Build a standalone `.exe`** (output: `src-tauri/target/release/habit_levelling.exe`):

```bash
npm run tauri build -- --no-bundle
```

Leave out `--no-bundle` to also build `.msi` and setup `.exe` installers under `src-tauri/target/release/bundle/`.

**Run the tests:**

```bash
cd src-tauri
cargo test
```

> **Windows tip:** run Cargo from PowerShell or cmd, not Git Bash. Git Bash's `/usr/bin/link` shadows the MSVC linker and the build fails with `link: extra operand`.

## Architecture

All the rules live in Rust; React only displays data and calls Tauri commands.

```text
React UI (src/)  ──invoke──▶  Tauri commands (commands.rs)  ──▶  habits.rs  ──▶  SQLite (SQLx)
                                                                  │
                                         pure logic: streaks · stats · decay · luck · history · activity · recommend
```

Two design rules keep the data trustworthy:

- **Derived, not stored.** XP, levels, streaks and decay are recalculated from your completion history every time, so they can never drift out of sync. The only exception is Luck rolls: they're random, so each roll is recorded once.
- **Local calendar dates.** Completions are stored as `YYYY-MM-DD` in your local time, with one completion per habit per day.

| Path | What it does |
| --- | --- |
| `src-tauri/src/commands.rs` | Tauri commands: the frontend's only API. Decides what "today" is. |
| `src-tauri/src/habits.rs` | Habit CRUD, completions, summaries and stat queries |
| `src-tauri/src/stats.rs` | Stats, XP per completion, level curve |
| `src-tauri/src/streaks.rs` | Current, longest and per-day streaks |
| `src-tauri/src/decay.rs` | Idle stats fading, replayed day by day |
| `src-tauri/src/luck.rs` | Streak milestones and Luck rolls |
| `src-tauri/src/history.rs` | Day-by-day stat XP and levels for the Analysis page |
| `src-tauri/src/activity.rs` | A year of daily activity for the heatmap |
| `src-tauri/src/recommend.rs` | Keyword-based stat suggestions |
| `src-tauri/src/db.rs` | SQLite connection and embedded migrations |
| `src-tauri/migrations/` | Database schema |
| `src/pages/` | Today (dashboard), habit details and Analysis pages |
| `src/components/` | Habit cards, forms, stat panel, charts and heatmap |
| `src/lib/` | Typed command wrappers, shared types and date helpers |
