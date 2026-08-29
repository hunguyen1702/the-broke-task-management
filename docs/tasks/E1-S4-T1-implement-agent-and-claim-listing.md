---
id: E1-S4-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S3-T1
  - E2-S1-T1
  - E3-S1-T1
  - E4-S2-T1
  - E5-S1-T1
---

# E1-S4-T1: Implement agent and claim listing

## Parent story

[E1-S4: List agents and claims](../epics/E1-S4-list-agents-and-claims.md)

## Objective

Implement a read-only command that returns all registered agents and their current claimed tasks from one consistent SQLite snapshot with stable human and JSON output.

## Deliverables

- Agent-with-claims query result models in `tbtm-core`.
- A read-only listing service that integrates with the completed agent, task, status, and active-claim persistence models.
- `tbtm agent list [--json]` CLI handler.
- Stable human and JSON renderers.
- Unit, integration, snapshot-consistency, output, and error-mapping tests.

## Proposed structure

Extend the implemented workspace without duplicating repository resolution or persistence models:

```text
crates/
├── tbtm-core/
│   └── src/
│       └── agent/
│           ├── model.rs
│           └── list.rs
└── tbtm-cli/
    └── src/
        └── commands/agent_list.rs
```

Exact module names should follow the layout produced by prerequisite implementation. `tbtm-core` owns the query, aggregation, ordering, and typed results; `tbtm-cli` owns argument parsing, rendering, and process exit codes.

## Technical choices

- Reuse the E1-S2 resolver with read-only, no-create access.
- Do not run `PRAGMA quick_check`, full repository-health validation, or migrations as part of listing.
- Start one SQLite read transaction and build the complete response before ending that transaction.
- Query the authoritative tables and relationships produced by E1-S3, E2-S1, E3-S1, and E5-S1. Do not add duplicate agent, task, status, or claim storage for this story.
- Use a set-based query or a small fixed number of queries inside the transaction. Do not issue one claim query per agent.
- Preserve agents without claims through outer-join semantics or equivalent aggregation.
- Return only active claims. Use current task titles and current status names from the same snapshot.
- Perform no network access and no filesystem writes.

## Result contract

Core result models have the following logical shape:

```text
AgentList
  agents: AgentWithClaims[]

AgentWithClaims
  id: UUID
  baseName: string
  displayName: string
  createdAt: RFC 3339 UTC timestamp
  claims: CurrentClaimSummary[]

CurrentClaimSummary
  taskId: stable task identifier
  title: current task title
  status: current status name
  claimedAt: RFC 3339 UTC timestamp
```

Sort agents by `createdAt`, then `id`, both ascending. Sort each claim collection by `claimedAt`, then `taskId`, both ascending. Apply ordering explicitly in the persistence query and/or core aggregation rather than relying on incidental SQLite row order.

## Implementation flow

### 1. Parse command

- Add `tbtm agent list` with optional `--json`.
- Accept no filter, pagination, alternate ordering, or agent selector options in this story.
- Reject unsupported arguments through the shared CLI validation behavior.

### 2. Resolve repository read-only

- Resolve from the current working location through E1-S2.
- Open the configured existing SQLite database read-only and without implicit creation.
- Preserve shared repository, configuration, database, and permission errors.
- Do not apply a pending migration. A repository lacking required prerequisite migrations follows the shared unsupported/incomplete database-state behavior established by the migration framework.

### 3. Read one consistent snapshot

Within one SQLite read transaction:

1. Read every registered agent in deterministic order.
2. Read active claims joined to their current task and status data.
3. Aggregate claim summaries by authoritative agent UUID.
4. Retain an empty claim collection for every agent with no matching active claim.
5. Complete all row decoding and result construction before the transaction ends.

Treat a persisted active claim referencing a missing agent, task, or status according to the integrity-error contract defined by the prerequisite model. Do not silently omit contradictory ownership data or repair it during listing.

### 4. Render output

Human output groups current tasks beneath each agent and clearly renders an agent with no claims. Avoid locale-dependent ordering or timestamp transformations that would destabilize snapshots.

JSON uses the shared envelope:

```json
{
  "ok": true,
  "data": {
    "agents": [
      {
        "id": "UUID",
        "baseName": "claude",
        "displayName": "claude-a1b2c3d4",
        "createdAt": "RFC 3339 UTC timestamp",
        "claims": [
          {
            "taskId": "project-story-a1b2c3",
            "title": "Implement parser",
            "status": "In progress",
            "claimedAt": "RFC 3339 UTC timestamp"
          }
        ]
      }
    ]
  },
  "error": null
}
```

JSON mode writes exactly one JSON document to stdout. Empty results serialize as `"agents": []`; an agent without claims serializes as `"claims": []`, never `null` or an omitted field.

### 5. Map errors and exit codes

| Exit | Condition |
|---:|---|
| 0 | List returned, including an empty registry |
| 1 | Database/integrity/operational or unexpected failure |
| 2 | Invalid repository configuration or CLI validation |
| 3 | Repository not initialized |
| 5 | Permission denied |

Use the shared stable error envelope. Do not expose internal SQL or silently convert a query failure into an empty successful list.

## Test plan

### Unit tests

- Aggregate zero agents, one agent without claims, and multiple agents with multiple claims.
- Sort equal timestamps with the documented UUID/task-ID tie breakers.
- Serialize empty collections as arrays.
- Serialize exact camelCase result fields and RFC 3339 UTC timestamps.
- Render stable human output for empty, unclaimed, and claimed-agent cases.
- Map typed errors to inherited exit codes.

### Integration tests

- List from repository root and a nested directory.
- Return exit code 0 for an empty agent table.
- List all registered agents when none has a claim.
- List a mixture of unclaimed agents and agents with one or several active claims.
- Verify task IDs, current titles, current status names, and original `claimedAt` values.
- Release a claim and verify the task disappears from ownership output while its agent remains.
- Verify deterministic agent and claim ordering across repeated invocations.
- Compare CLI data with direct SQLite fixture queries.
- Preserve inherited E1-S2 errors for repository/config/database/permission failures.
- Verify missing or inconsistent prerequisite data returns an operational/integrity error rather than partial success.
- Snapshot human and JSON success/error output.

### Consistency and performance tests

- Coordinate a concurrent local claim mutation while listing and verify a response reflects one committed snapshot, not mixed before/after rows.
- Use a representative personal repository with many agents and claims and verify the query count remains fixed rather than growing per agent.
- Verify listing does not change config/database content or leave application-created journal/WAL artifacts.

## Verification

After prerequisite implementations exist, provide or reuse `mise` tasks for:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Smoke verification:

1. Initialize a temporary repository and confirm `tbtm agent list --json` returns `agents: []`.
2. Register two agents and confirm both appear with empty claim arrays.
3. Create claimable tasks through prerequisite commands and assign multiple claims while leaving one agent unclaimed.
4. Confirm human and JSON output show the same identities, tasks, statuses, timestamps, and order.
5. Release one claim and confirm only current ownership changes.

## Definition of done

- Parent story functional and non-functional acceptance criteria pass.
- All agents remain discoverable regardless of claim count.
- Current claim summaries contain the documented task and status fields.
- Ordering, empty-array behavior, JSON envelope, and human rendering are stable.
- One read transaction provides a consistent snapshot without mutation.
- Implementation reuses prerequisite persistence models and avoids per-agent query growth.
- Shared errors and exit codes are preserved.
- Formatting, lint, and tests pass through `mise`.

## References

- [Product requirements](../PRD.md), especially actor model, claim model, FR-2, FR-8, FR-11, E1-S4, and E5-S1.
- [E1-S2 epic](../epics/E1-S2-resolve-repository-configuration.md)
- [E1-S2 implementation task](E1-S2-T1-implement-repository-configuration-resolution.md)
- [E1-S3 epic](../epics/E1-S3-register-an-agent.md)
- [E1-S3 implementation task](E1-S3-T1-implement-agent-registration.md)
- Future E2-S1, E3-S1, E4-S2, and E5-S1 plans and implementations are prerequisite references once available.
- [SQLite transactions](https://www.sqlite.org/lang_transaction.html)
- [SQLite query planner](https://www.sqlite.org/queryplanner.html)
