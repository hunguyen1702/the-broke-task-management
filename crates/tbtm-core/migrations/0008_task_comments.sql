CREATE TABLE task_comments (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    author_actor_type TEXT NOT NULL CHECK (author_actor_type IN ('user', 'agent')),
    author_agent_id TEXT REFERENCES agents(id),
    created_at TEXT NOT NULL,
    CHECK (
        (author_actor_type = 'user' AND author_agent_id IS NULL)
        OR (author_actor_type = 'agent' AND author_agent_id IS NOT NULL)
    )
);

CREATE INDEX task_comments_chronological
ON task_comments(task_id, created_at, id);
