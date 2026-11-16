-- Skills become D&D-style stats. Each habit trains a primary stat and,
-- optionally, a secondary stat. XP and levels are still derived, never stored.
ALTER TABLE habits ADD COLUMN primary_stat TEXT
    CHECK (primary_stat IN ('strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'));
ALTER TABLE habits ADD COLUMN secondary_stat TEXT
    CHECK (secondary_stat IN ('strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'));

-- Carry over the old free-text skills where there is an obvious match.
UPDATE habits SET primary_stat = CASE lower(trim(skill))
    WHEN 'fitness'    THEN 'strength'
    WHEN 'strength'   THEN 'strength'
    WHEN 'dexterity'  THEN 'dexterity'
    WHEN 'discipline' THEN 'constitution'
    WHEN 'health'     THEN 'constitution'
    WHEN 'knowledge'  THEN 'intelligence'
    WHEN 'technical'  THEN 'intelligence'
    WHEN 'mental'     THEN 'wisdom'
    WHEN 'social'     THEN 'charisma'
    WHEN 'creativity' THEN 'charisma'
    ELSE NULL
END;

ALTER TABLE habits DROP COLUMN skill;

-- The secret Luck stat. Luck can't be derived from history because it is
-- random, so each roll is recorded once per habit per milestone day. That
-- also stops undo/redo from re-rolling the same milestone.
CREATE TABLE luck_rolls (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_id INTEGER NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    date     TEXT    NOT NULL,
    streak   INTEGER NOT NULL, -- the streak length that triggered the roll
    xp       INTEGER NOT NULL, -- 0 when the roll missed
    UNIQUE (habit_id, date)
);
