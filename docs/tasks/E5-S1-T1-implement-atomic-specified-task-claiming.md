---
id: E5-S1-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E1-S3-T1
  - E1-S5-T1
  - E2-S1-T1
  - E2-S2-T1
  - E3-S1-T1
  - E4-S2-T1
---

# E5-S1-T1: Implement atomic specified-task claiming

## Parent story

[E5-S1: Claim a specified task atomically](../epics/E5-S1-claim-a-specified-task-atomically.md)

## Objective

Implement an agent-only specified-task claim command backed by a unique SQLite claim relation, an atomic current-state availability recheck, stable conflict/unavailable errors, and the existing full-task output contract.

## Readiness

Planning is complete. Agent identity, shared linked-worktree repository resolution, task/status models, full-task claim placeholders, and authoritative dependency satisfaction are implemented, so this task is ready.

## Deliverables

- Sequential SQLite migration for active claims and supporting indexes or constraints.
- Core claim input/result behavior, availability validation, transaction logic, and typed errors.
- Full-task and compact-list claim hydration using the E2-S2 claim shape.
- `tbtm task claim` CLI parsing, human rendering, JSON envelopes, and exit mapping.
- Migration, core, integration, concurrency, linked-worktree, output, and regression tests.

## Proposed structure

Extend the existing task module while preserving the core/CLI boundary. A focused claim submodule may be introduced if it keeps transaction and hydration queries coherent:

```text
crates/tbtm-core/
├── migrations/
│   └── 0005_task_claims.sql
└── src/
    ├── lib.rs
    └── task.rs                 # or task/claim.rs

crates/tbtm-cli/src/
└── main.rs
```

Exact file boundaries may follow the current implementation. Core APIs must remain testable without invoking the process boundary.

## Technical choices

- Add a `task_claims` table keyed by `task_id`, foreign-keyed to `tasks(id)`, with required `agent_id` referencing `agents(id)` and required `claimed_at`. The primary key enforces at most one active claim per task. Add only indexes justified by claim-owner/list hydration queries.
- Add internal migration version 5 and advance the latest migration while leaving repository/config `schemaVersion: 1` unchanged.
- Reuse the E1-S5 canonical shared database, existing busy-timeout configuration, `TransactionBehavior::Immediate`, E3-S1 completion data, and E4-S2 all-upstreams-effectively-completed logic.
- Keep availability derived. Do not add cached `available`, `blocked`, or dependency-satisfaction columns.
- Reuse `FullTask`, `TaskListItem`, `TaskClaim`, and `ClaimAgent`. Do not create a competing success projection.
- Claim insertion does not update the task row. `updated_at`, actor columns, status, archive state, and dependencies remain unchanged.

## Implementation flow

### 1. Add the claim migration

Create the claim table with:

- exactly one row permitted for each task;
- valid task and registered-agent foreign keys;
- non-null claim timestamp;
- referential behavior compatible with the MVP, which exposes neither permanent task deletion nor agent deletion.

Apply migration 5 through the existing pending-compatible-migration path used by write commands. Read-only task view/list retain their established behavior: they do not apply migrations, and a compatible repository with migration 5 pending returns the stable incomplete-database error rather than treating every claim as absent or leaking a missing-table error.

### 2. Define core and error contracts

Represent a request with specified task ID and required agent UUID. A successful core operation returns the hydrated shared `FullTask`.

Add typed errors:

- `ClaimConflict` carrying task ID, owner UUID/display name, and original claim timestamp;
- `TaskNotAvailable` carrying task ID, stable reason, and unresolved upstream IDs only for dependency blocking.

Serialize unavailable reasons exactly as `archived`, `completed`, and `dependencies_blocked`. Sort unresolved upstream IDs by task ID ascending. Reuse `AgentNotFound` and `TaskNotFound`.

### 3. Claim under one immediate transaction

1. Parse task ID and the required agent UUID at the CLI boundary.
2. Resolve the canonical repository read-write, configure the existing busy timeout, and apply pending compatible migrations.
3. Start an immediate write transaction.
4. Resolve the registered agent or return `AGENT_NOT_FOUND`.
5. Load the specified task or return `TASK_NOT_FOUND`.
6. If a claim exists, return `CLAIM_CONFLICT` before inspecting other availability conditions. This includes a same-agent re-claim.
7. Reject archive, completed status, then unresolved direct upstream dependencies in that order.
8. Generate one UTC timestamp and insert the claim row.
9. Hydrate the full task, including claim and direct relationships, from transaction-owned queries.
10. Commit and return the result.

Every typed rejection and database failure rolls back. The unique task key remains a final integrity guard if a future write path violates the supported serialization contract.

A contender blocked behind another immediate writer waits according to the existing busy timeout. After it acquires the lock it performs the full recheck, observes a committed winner, and returns `CLAIM_CONFLICT`. If lock acquisition itself exhausts the timeout, preserve the inherited operational database error because ownership was not safely observed.

### 4. Hydrate claims in existing reads

Replace the E2-S2 claim placeholders in full task detail and compact task list with:

```json
{
  "agent": {
    "id": "agent UUID",
    "displayName": "worker-a1b2c3d4"
  },
  "claimedAt": "RFC 3339 UTC timestamp"
}
```

Unclaimed tasks remain `null`. Load claims inside each command's existing read snapshot and avoid per-task query growth for lists. `task create` remains unclaimed. Existing hierarchy, dependency, context, archive, ordering, and projection contracts remain unchanged.

### 5. Add CLI parsing and rendering

Expose:

```text
tbtm task claim <task-id> --agent <uuid> [--json]
```

The command is non-interactive and requires `--agent`. Human success prints the task ID, owner display name, and claim time. JSON returns the full-task result under the shared envelope.

Map errors as follows:

| Exit | Errors |
|---:|---|
| 0 | Successful claim |
| 1 | Database, busy-timeout, or other operational failure |
| 2 | Malformed/missing arguments or `TASK_NOT_AVAILABLE` |
| 3 | `AGENT_NOT_FOUND` or `TASK_NOT_FOUND` |
| 4 | `CLAIM_CONFLICT` |
| 5 | Permission denied |

`CLAIM_CONFLICT` details are `{taskId, agent: {id, displayName}, claimedAt}`. `TASK_NOT_AVAILABLE` details are `{taskId, reason}` plus sorted `unresolvedUpstreamIds` only for `dependencies_blocked`. No failure mutates claim or task state.

## Test plan

### Migration and core tests

- Apply migration 5 once and through normal pending-migration handling; reject duplicate task claims and invalid task/agent references at the database boundary.
- Claim an active, incomplete, unclaimed task with zero upstreams and with completed-status, archived, and mixed-resolved upstream sets.
- Reject archived, completed, and each dependency-blocked combination with exact reason and sorted unresolved IDs.
- Verify validation precedence for unknown agent, missing task, existing claim, archived, completed, and blocked states, including claimed-plus-completed and claimed-plus-blocked tasks.
- Re-claim as the same agent and as another agent; assert conflict details and preservation of the original timestamp.
- Verify successful and failed claims never change task status, archive state, `updated_at`, actor metadata, or dependency rows.
- Inject an insert/hydration failure and prove the transaction leaves no partial claim.
- Exercise busy-timeout exhaustion and assert an operational error rather than `CLAIM_CONFLICT`.

### Concurrency and integration tests

- Start two connections or processes against the same initially available task, coordinate simultaneous attempts, and assert exactly one success, one conflict after winner observation, and one persisted owner/timestamp.
- Repeat the race from separate linked worktrees and assert they share the same winner and database without corruption.
- Snapshot exact human success, JSON full-task success, conflict, unavailable, malformed UUID, missing agent, missing task, permission, and operational outputs and exits.
- Verify task view and list show populated claim summaries without changing their other fields or deterministic ordering; unclaimed and create outputs remain `null`.
- Confirm read-only task view/list do not apply migration 5 or create persistent SQLite side artifacts; after a supported write applies migration 5, both reads succeed and hydrate claims.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test two registered agents claiming one task through separate linked worktrees, plus archived, completed, resolved-dependency, and blocked-dependency tasks.

## Definition of done

- Every functional and non-functional criterion in E5-S1 passes.
- Migration 5 establishes the authoritative one-active-claim model without changing repository compatibility version 1.
- Specified-task claim rechecks the complete current availability predicate and inserts under one immediate transaction.
- Concurrent supported contenders cannot overwrite a winner; observed winners produce stable conflict details and busy-timeout exhaustion remains an operational error.
- Full-task and list reads hydrate the established claim shape with no regression to unclaimed, relationship, ordering, or read-only migration behavior.
- Claim success and every failure preserve task status and mutation metadata.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E5-S1 epic](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [PRD claim model](../PRD.md#79-claim)
- [PRD availability definition](../PRD.md#81-available-task-definition)
- [PRD claim management](../PRD.md#fr-8-claim-management)
- [PRD concurrent claim workflow](../PRD.md#105-concurrent-claim)
- [E1-S3 agent identity](../epics/E1-S3-register-an-agent.md)
- [E1-S5 shared worktree state](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [E2-S1 task creation](../epics/E2-S1-create-a-task.md)
- [E2-S2 full-task and claim shape](../epics/E2-S2-view-and-list-tasks.md)
- [E3-S1 completion semantics](../epics/E3-S1-use-default-statuses.md)
- [E4-S2 dependency satisfaction](../epics/E4-S2-manage-dependencies.md)
