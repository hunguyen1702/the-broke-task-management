CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    base_name TEXT NOT NULL,
    display_name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);
