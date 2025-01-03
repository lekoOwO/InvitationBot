-- Add migration script here
CREATE TABLE IF NOT EXISTS invites (
    id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    used_at DATETIME,
    used_by TEXT,
    discord_invite_code TEXT
); 