CREATE TABLE IF NOT EXISTS vaults (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS env_vars (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    vault_id   INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(vault_id, key)
);

CREATE TABLE IF NOT EXISTS dir_vaults (
    dir        TEXT PRIMARY KEY,
    vault_name TEXT NOT NULL
);

INSERT OR IGNORE INTO vaults (name) VALUES ('default');
