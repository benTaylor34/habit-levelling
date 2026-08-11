CREATE TABLE habits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    max_streak INTEGER,
    frequency TEXT NOT NULL DEFAULT 'daily'
);

CREATE TABLE habit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_id INTEGER NOT NULL,
    log_date TEXT NOT NULL,
    completion INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE, --deletes logs when habit is deleted
    UNIQUE (habit_id, log_date) --makes making as complete idempotent
);

CREATE INDEX idx_habit_logs_habit_id ON habit_logs(habit_id); --faster lookup