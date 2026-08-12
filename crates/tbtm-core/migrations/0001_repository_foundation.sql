CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE repository_metadata (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    repository_id TEXT NOT NULL,
    prefix TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE statuses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    completed INTEGER NOT NULL CHECK (completed IN (0, 1)),
    display_order INTEGER NOT NULL UNIQUE,
    is_default INTEGER NOT NULL CHECK (is_default IN (0, 1))
);
