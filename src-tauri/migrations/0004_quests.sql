-- Completed daily quests. The day's offer is derived (seeded by the date),
-- but its target is random and scaled to the level at the time, so what was
-- achieved and the XP it earned are recorded once, like luck_rolls.
CREATE TABLE quest_completions (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    date     TEXT    NOT NULL,          -- local calendar day, YYYY-MM-DD
    quest_id TEXT    NOT NULL,          -- catalog id, e.g. 'str-pushups'
    stat     TEXT    NOT NULL CHECK (stat IN ('strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma')),
    amount   INTEGER NOT NULL,
    xp       INTEGER NOT NULL,
    UNIQUE (date, quest_id)
);
