-- Stores which Discord users are blocked from interacting with the bot
CREATE TABLE IF NOT EXISTS blacklist_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    discord_id INTEGER NOT NULL UNIQUE,
    moderator_id INTEGER NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blacklist_entries_discord_id ON blacklist_entries(discord_id);
CREATE INDEX IF NOT EXISTS idx_blacklist_entries_created_at ON blacklist_entries(created_at);
