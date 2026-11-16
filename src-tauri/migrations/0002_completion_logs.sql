-- Each completion now records how long was spent and how hard it felt.
-- XP is NOT stored; it is derived from these two values in Rust (skills.rs).
-- Completions logged before this migration count as 0 minutes at difficulty 1.
ALTER TABLE habit_completions ADD COLUMN minutes    INTEGER NOT NULL DEFAULT 0;
ALTER TABLE habit_completions ADD COLUMN difficulty INTEGER NOT NULL DEFAULT 1;
