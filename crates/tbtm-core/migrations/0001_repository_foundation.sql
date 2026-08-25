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
    code TEXT NOT NULL UNIQUE CHECK (
        length(code) > 0
        AND substr(code, 1, 1) BETWEEN 'a' AND 'z'
        AND code NOT GLOB '*[^a-z0-9_]*'
    ),
    name TEXT NOT NULL UNIQUE,
    completed INTEGER NOT NULL CHECK (completed IN (0, 1)),
    display_order INTEGER NOT NULL UNIQUE,
    is_default INTEGER NOT NULL CHECK (is_default IN (0, 1))
);
