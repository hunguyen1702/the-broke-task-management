CREATE TABLE task_claims (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id),
    claimed_at TEXT NOT NULL
);

CREATE INDEX task_claims_agent_idx
    ON task_claims (agent_id, claimed_at, task_id);
