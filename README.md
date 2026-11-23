# Habit Levelling

**Turn your daily habits into an RPG character sheet.** Log what you do each day, keep your streaks alive, and level up six D&D-style stats, but let them slide and they start to fade.

Habit Levelling is a local-first app for **Windows and Android**, built with **Rust + Tauri**, **SQLite**, and a **React + TypeScript** interface. It runs entirely on your own device: no account, no server, no internet connection.

![The Today page: habits, stat radar and levels, and a year of activity (sample data)](docs/screenshots/today.png)

## Features

- **Daily habits.** Tick a habit off for today, log how long you spent and how hard it felt, and undo it if you mis-click.
- **Quick time logging.** Build up the time with +5, +10 and +30 minute buttons, or tap the total to type an exact time. Did a second session later? Add its time to today's log.
- **Six stats.** Each habit trains a primary stat (full XP) and optionally a secondary stat (half XP): Strength, Dexterity, Constitution, Intelligence, Wisdom and Charisma.
- **Smart stat suggestions.** Type a habit like "Study maths" or "Weightlifting" and the app suggests stats for it (Intelligence + Wisdom, Strength + Constitution).
- **Levelling that gets harder.** Your first completion levels a stat up straight away, each level after costs a little more, and level 100 takes about a year of serious daily work.
- **Streaks everywhere.** Each habit and each stat has a current and longest streak, and longer streaks earn more XP.
- **Use it or lose it.** Stats you stop training slowly lose XP, then levels. Habits you haven't done for a week get a warning.
- **A secret stat.** Luck can't be trained. It may appear when a streak hits a milestone…
- **Character overview.** A radar chart of your six stats, with an XP bar and streak for each one.
- **Activity heatmap.** A GitHub-style year of activity at a glance.
- **Analysis.** Stat levels or total XP over time for the last 30 days, 90 days, year or all time, plus a table of what changed.
- **Habit history.** A monthly calendar, recent activity and lifetime totals for every habit.
- **Backups.** Export your data to a file, restore it on another PC, and get an automatic backup every day.

![The Analysis page: stat levels over time with a hover readout (sample data)](docs/screenshots/analysis.png)

## How the game works

### Earning XP

Each day you log a habit, its stats earn:

> **XP = (10 + minutes spent) × difficulty × streak bonus**

| Difficulty | Easy | Moderate | Hard | Very hard | Brutal |
| --- | --- | --- | --- | --- | --- |
| Multiplier | ×1 | ×1.5 | ×2 | ×2.5 | ×3 |

- **Streak bonus:** +5% for every day the habit's streak has run past the first, reaching ×2 on day 21. After that it keeps growing slowly, +1% for every full week, up to ×2.5 after about a year (day 371).
- **Primary and secondary stats:** the primary stat gets the full amount and the secondary stat gets half.

For example, 30 minutes of Hard studying on day 7 of a streak gives **(10 + 30) × 2 × 1.3 = 104 Intelligence** and **52 Wisdom**.

### Logging time

Each habit has one log per day. When you tick a habit off, choose how long you spent with the **+5 min**, **+10 min** and **+30 min** buttons, or tap the total to type an exact number of minutes (up to 24 hours), then pick how hard it was.

If you come back to a habit later the same day, press **+ Time** on its card and choose **Add time** to add the new session to today's total, or **Edit total** to correct it. XP is worked out from the day's total, so splitting a session into several logs never earns extra.

Forgot to log a day? Open the habit and pick the day on its calendar to fill it in, change it or remove it. Streaks and XP update as if you'd logged it on time, including the streak bonus on later days. Filled-in days never roll for Luck.

A day with less than **10 minutes** logged earns no XP for stats or the habit's level, but it still counts as done, so the streak carries on. Adding time later that day to reach 10 minutes earns the XP after all.

### Levels

Level 1 → 2 costs just 10 XP, so the first completion of 10 minutes or more levels its stat up. After that, each level costs 40 XP more than the one before:

| Level up | 1 → 2 | 2 → 3 | 5 → 6 | 10 → 11 | 50 → 51 | 99 → 100 |
| --- | --- | --- | --- | --- | --- | --- |
| XP needed | 10 | 50 | 170 | 370 | 1,970 | 3,930 |

Level 100 takes 195,030 XP in total, which is about 11 months of a Hard 2-hour habit every day without breaking the streak. The XP bar resets after every level up. Levels stop at 100.

### Daily quests

Each day brings **3 quests, each for a different stat**: small, at-home challenges such as push-ups, a walk, journaling, reading or starting a conversation. Some quests are about healthy basics (sleep, water, screen-free time before bed, reaching out to people), not just training.

- **Targets scale with that stat's level** at the start of the day, and vary a little from day to day. For example, push-ups are about 5–11 at level 1, 21–49 at level 10, and up to around 150 at high levels. Every quest has a cap, so all three stay doable in one day. Sleep and water quests stay at healthy amounts at every level.
- **Completing one gives its stat XP** worth 1–20% of what that stat needs for its next level: a bigger share at low levels and more for harder targets, so quests stay worthwhile as you level up. Quest XP counts as training, so it also holds off decay.
- **The same 3 quests stay for the whole day**, and a new set arrives tomorrow. Find them on the **Quests** tab: tick one to complete it, and tick again to undo.

### Habit levels

Each habit also has its own level, shown next to its name. It's a slow measure of how much time you've given the habit:

- **XP:** each completed day of 10 minutes or more earns 10 XP plus 1 per minute spent, the same base as a stat. Difficulty and streaks don't affect it.
- **Cost:** each habit level costs twice the XP of the same stat level.
- **Visibility:** the XP itself isn't shown.

For a typical habit (daily, Moderate, streak kept) that makes habit levels take roughly **4× as long** as its stat's levels. A 30-minute daily habit reaches level 10 in about 2½ months and level 20 in about a year.

### Decay

| Days since a stat last earned XP | What happens each day |
| --- | --- |
| 0–6 | Nothing |
| 7–20 | It loses 10% of its current level's XP bar, down to 0. The level is kept. |
| 21+ | Once the bar is empty it drops a level (landing at 90% of that bar) and keeps draining. It never goes below level 1. |

Training the stat again stops the decay straight away. Stats that are fading are marked in the stat panel.

**Habit levels fade the same way** when you stop doing the habit: the same timings, measured from the last day you did it, on the habit's own XP bar. Doing it again, even for less than 10 minutes, stops the fade. A fading habit shows a warning on its card and its page. A habit you've never done has nothing to lose, so it shows no warning.

### Luck

Whenever a habit's streak reaches a multiple of 7 days there's a 3% chance of +25 Luck XP. Every 30 days there's a 15% chance of +100. Luck stays hidden until you first win some.

## Download

Grab the latest version from the [Releases](../../releases) page.

**Windows** (64-bit Windows 10 and 11): download `habit_levelling.exe` and run it. There's nothing to install.

> **"Windows protected your PC"?** The app isn't code-signed yet, so Windows warns about it the first time. Choose **More info → Run anyway**.

**Android** (Android 7.0 or newer): download `habit-levelling.apk` on your phone and open it. Android asks you to allow installing apps from your browser or file manager the first time; allow it, then tap **Install**. Updates install over the top and keep your data.

## Your data

Everything is saved in a single SQLite file on your own device:

- Windows: `%APPDATA%\com.habitlevelling.app\habits.db`
- Android: inside the app's private storage (removed if you uninstall the app, so export a backup first)

Nothing is sent anywhere, and your data isn't synced between devices.

Everything about backups is on the **Data** page:

- **Export a backup** saves all your habits and daily logs as one readable `.json` file, wherever you choose.
- **Restore from a backup** replaces the app's data with a backup's contents, for example to move to a new PC or phone. Backups work across Windows and Android. It shows what the file contains and asks first. Your current data is saved as a safety backup before anything is replaced, so a restore can always be undone.
- **Automatic backups** are saved the first time you open the app each day, into a `backups` folder next to your data. The last 14 are kept.

XP, levels, streaks and decay aren't stored in backups; they're rebuilt from your logs, so a restore brings them all back exactly.

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

**Build a standalone `.exe`** (output: `target/release/habit_levelling.exe`):

```bash
npm run tauri build -- --no-bundle
```

Leave out `--no-bundle` to also build `.msi` and setup `.exe` installers under `target/release/bundle/`.

**Run the tests** from the repo root. The game rules can be tested on their own, with no database or Tauri:

```bash
cargo test -p habit_core
```

```bash
cargo test --workspace
```

### Android

You'll also need a JDK 17, the Android SDK with an NDK (r27 works), and Rust's Android targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Set `JAVA_HOME`, `ANDROID_HOME` and `NDK_HOME`, then build a debug APK for the emulator or run it on a connected device:

```bash
npm run tauri android build -- --debug --target x86_64 --apk
```

```bash
npm run tauri android dev
```

> **Windows:** turn on **Developer Mode** (Settings → System → For developers). The Android build links the compiled Rust library into the Android project, and Windows only lets normal accounts create those links in Developer Mode.

**Release builds are signed** with a key described in `src-tauri/gen/android/key.properties`, which is git-ignored and never committed:

```properties
storeFile=C:/path/to/your-release.jks
keyAlias=your-key-alias
password=your-keystore-password
```

Without that file, release APKs are built unsigned. Keep the keystore safe: Android only installs updates signed with the same key.

> **Windows tip:** run Cargo from PowerShell or cmd, not Git Bash. Git Bash's `/usr/bin/link` shadows the MSVC linker and the build fails with `link: extra operand`.

### Releases (GitHub Actions)

Pushing a version tag builds every platform from that same commit:

```bash
git tag v0.4.0
```

```bash
git push origin v0.4.0
```

The **Release** workflow creates a **draft** GitHub release and attaches the Windows exe and the Android APK. Review it, write the notes and publish. The **CI** workflow runs the tests and type-check on every push and pull request.

To sign the APK in CI, add three repository secrets (Settings → Secrets and variables → Actions): `ANDROID_KEYSTORE_BASE64` (the keystore file, base64-encoded), `ANDROID_KEY_ALIAS` and `ANDROID_KEY_PASSWORD`. Without them the APK is built unsigned.

## Architecture

One repo, organised so shared code sits in the middle and platform-specific code stays at the edges.

```text
crates/core/     habit_core: the game rules. Pure Rust, no database, UI or platform code.
src-tauri/       The Tauri app: SQLite storage, backups and the commands the UI calls.
  gen/android/   The Android shell (MainActivity.kt: status bar, keyboard, Back button).
src/             The React UI, shared by desktop and Android.
  platform/      Everything that differs by platform: file pickers, dialogs, Android Back.
```

```text
React UI (src/)  ──invoke──▶  Tauri commands (commands.rs)  ──▶  habits.rs  ──▶  SQLite (SQLx)
                                                                    │
                          habit_core: stats · streaks · decay · luck · history · activity · recommend
```

All the rules live in Rust; React only displays data and calls Tauri commands. A future web or server version would build on `habit_core` in the same way.

Two design rules keep the data trustworthy:

- **Derived, not stored.** XP, levels, streaks and decay are recalculated from your completion history every time, so they can never drift out of sync. The only exception is Luck rolls: they're random, so each roll is recorded once.
- **Local calendar dates.** Completions are stored as `YYYY-MM-DD` in your local time, with one completion per habit per day.

| Path | What it does |
| --- | --- |
| `crates/core/src/stats.rs` | Stats, XP per completion, level curve |
| `crates/core/src/streaks.rs` | Current, longest and per-day streaks |
| `crates/core/src/decay.rs` | Idle stats fading, replayed day by day |
| `crates/core/src/luck.rs` | Streak milestones and Luck rolls |
| `crates/core/src/history.rs` | Day-by-day stat XP and levels for the Analysis page |
| `crates/core/src/activity.rs` | A year of daily activity for the heatmap |
| `crates/core/src/recommend.rs` | Keyword-based stat suggestions |
| `crates/core/src/quests.rs` | Daily quest catalog, daily picks, scaled targets and quest XP |
| `crates/core/src/habit_levels.rs` | Each habit's own level, from time spent |
| `crates/core/src/models.rs` | Shared data types, serialised for the UI |
| `src-tauri/src/commands.rs` | Tauri commands: the frontend's only API. Decides what "today" is. |
| `src-tauri/src/habits.rs` | Habit CRUD, completions, summaries and stat queries (SQLite) |
| `src-tauri/src/backup.rs` | JSON export, checked all-or-nothing restore, daily automatic backups |
| `src-tauri/src/db.rs` | SQLite connection and embedded migrations |
| `src-tauri/migrations/` | Database schema |
| `src/pages/` | Today (dashboard), habit details, Quests, Analysis and Data pages |
| `src/components/` | Habit cards, forms, stat panel, charts and heatmap |
| `src/platform/` | Platform edge: file pickers, native dialogs, Android Back button |
| `src/lib/` | Typed command wrappers, shared types and date helpers |
| `.github/workflows/` | CI (tests on every push) and Release (builds on version tags) |

## Licence

[MIT](LICENSE): free to use, modify and share, including commercially, as long as the copyright and licence notice are kept.
