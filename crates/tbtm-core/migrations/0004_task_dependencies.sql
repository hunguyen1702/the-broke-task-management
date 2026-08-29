CREATE TABLE task_dependencies (
    downstream_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    upstream_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    PRIMARY KEY (downstream_task_id, upstream_task_id),
    CHECK (downstream_task_id <> upstream_task_id)
);

CREATE INDEX task_dependencies_upstream_idx
    ON task_dependencies (upstream_task_id, downstream_task_id);
