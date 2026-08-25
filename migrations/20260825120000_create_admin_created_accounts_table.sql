CREATE TABLE admin_created_accounts
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    username         TEXT    NOT NULL,
    admin_discord_id TEXT    NOT NULL,
    admin_username   TEXT    NOT NULL,
    created_at       INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_admin_created_accounts_admin_discord_id ON admin_created_accounts (admin_discord_id);
CREATE INDEX idx_admin_created_accounts_username ON admin_created_accounts (username);
