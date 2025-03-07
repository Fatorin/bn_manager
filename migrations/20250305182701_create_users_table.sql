CREATE TABLE users
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    discord_id TEXT    NOT NULL UNIQUE,
    username   TEXT    NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);