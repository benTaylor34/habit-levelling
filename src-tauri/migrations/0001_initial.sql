-- Habits: what the user wants to do. Streaks/XP are NOT stored here;
-- they are always derived from habit_completions so they can never go stale.
CREATE TABLE habits (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT,
    skill       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    archived    INTEGER NOT NULL DEFAULT 0
);

-- One row per habit per *local calendar day* (YYYY-MM-DD).
CREATE TABLE habit_completions (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_id INTEGER NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    date     TEXT    NOT NULL,
    UNIQUE (habit_id, date) -- completing twice on one day is still one completion
);

CREATE INDEX idx_habit_completions_habit_id ON habit_completions(habit_id);
