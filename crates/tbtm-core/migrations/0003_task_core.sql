CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    short_suffix TEXT NOT NULL UNIQUE CHECK (
        length(short_suffix) = 8 AND short_suffix NOT GLOB '*[^0-9a-f]*'
    ),
    task_type TEXT NOT NULL CHECK (task_type IN (
        'epic', 'story', 'task', 'improvement', 'refactor', 'bug', 'spike', 'testing', 'poc'
    )),
    title TEXT NOT NULL CHECK (length(title) > 0),
    description TEXT NOT NULL,
    goal TEXT NOT NULL,
    acceptance_criteria TEXT NOT NULL,
    status_id TEXT NOT NULL REFERENCES statuses(id),
    priority INTEGER NOT NULL CHECK (priority BETWEEN 0 AND 1000000),
    estimate_hours REAL CHECK (estimate_hours >= 0),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_actor_type TEXT NOT NULL CHECK (created_actor_type IN ('user', 'agent')),
    created_agent_id TEXT REFERENCES agents(id),
    updated_actor_type TEXT NOT NULL CHECK (updated_actor_type IN ('user', 'agent')),
    updated_agent_id TEXT REFERENCES agents(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK ((created_actor_type = 'user' AND created_agent_id IS NULL) OR (created_actor_type = 'agent' AND created_agent_id IS NOT NULL)),
    CHECK ((updated_actor_type = 'user' AND updated_agent_id IS NULL) OR (updated_actor_type = 'agent' AND updated_agent_id IS NOT NULL))
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    value TEXT NOT NULL CHECK (length(value) > 0),
    PRIMARY KEY (task_id, ordinal),
    UNIQUE (task_id, value)
);

CREATE TABLE task_external_urls (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    url TEXT NOT NULL CHECK (length(url) > 0),
    PRIMARY KEY (task_id, ordinal),
    UNIQUE (task_id, url)
);

CREATE TABLE task_code_references (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    path TEXT NOT NULL CHECK (length(path) > 0),
    start_line INTEGER CHECK (start_line > 0),
    end_line INTEGER CHECK (end_line > 0),
    description TEXT,
    PRIMARY KEY (task_id, ordinal),
    UNIQUE (task_id, path, start_line, end_line, description),
    CHECK (end_line IS NULL OR (start_line IS NOT NULL AND end_line >= start_line))
);
