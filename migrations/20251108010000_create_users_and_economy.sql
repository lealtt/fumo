-- Create users table with proper economy columns
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    discord_id INTEGER NOT NULL UNIQUE,
    dollars INTEGER NOT NULL DEFAULT 0,
    diamonds INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_discord_id ON users(discord_id);

-- Create reward_states table for tracking daily/weekly/monthly rewards
CREATE TABLE IF NOT EXISTS reward_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    reward_type TEXT NOT NULL,
    last_claimed_at TEXT,
    next_reset_at TEXT,
    total_claims INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, reward_type)
);

CREATE INDEX IF NOT EXISTS idx_reward_states_user_id ON reward_states(user_id);
CREATE INDEX IF NOT EXISTS idx_reward_states_user_reward ON reward_states(user_id, reward_type);
