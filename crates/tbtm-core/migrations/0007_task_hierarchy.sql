CREATE TABLE task_hierarchy (
    child_task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    parent_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    CHECK (child_task_id <> parent_task_id)
);

CREATE INDEX task_hierarchy_parent_idx
    ON task_hierarchy (parent_task_id, child_task_id);
