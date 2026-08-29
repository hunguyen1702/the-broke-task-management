---
id: E5-S3
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E5-S1
---

# E5-S3: Unclaim owned work

## Outcome

A registered coding agent can explicitly release its own active claim so that repository state immediately reflects the task's current claimability without changing workflow status or task mutation metadata.

## User story

As a coding agent, I want to release my claim so that another agent can acquire the task.

## Command

```text
tbtm task unclaim <task-id> --agent <uuid> [--json]
```

`--agent` is required. The UUID must identify the current claim owner; the logical `user` and agent display names are not accepted by this command.

## Product decisions

### Ownership and validation

- Only the registered agent that owns the task's active claim may unclaim it normally.
- Validation precedence is command syntax and UUID parsing, registered agent, task existence, active-claim existence, then ownership.
- A task without an active claim returns `CLAIM_NOT_FOUND`; unclaim is not an idempotent no-op.
- A claim owned by another agent returns `CLAIM_NOT_OWNED` with the current owner and original claim time. The command never transfers or overwrites ownership.
- Malformed UUID, unknown agent, and unknown task retain the shared validation, `AGENT_NOT_FOUND`, and `TASK_NOT_FOUND` behavior.

### Claim and task lifecycle

- Successful unclaim deletes only the active claim row.
- Task status, archive state, content, relationships, `updatedAt`, and `updatedBy` remain unchanged.
- Unclaim is valid for an owned claim regardless of the task's current status or dependency state. Claim/status independence allows a task to become completed or blocked while still claimed.
- Availability remains derived from current archive, status, claim, and dependency state. Removing the claim immediately changes subsequent availability queries and claim attempts, but this command does not add an availability projection to its output.
- Force-unclaim, claim transfer, automatic expiry, and implicit current-agent selection remain outside this story.

### Atomic concurrency behavior

- After compatible pending migrations are applied, core performs agent resolution, task lookup, claim lookup, owner validation, claim deletion, response hydration, and commit in one immediate SQLite transaction.
- Deletion is constrained to the exact task and invoking owner observed in the transaction. It cannot delete a foreign or replacement claim.
- If another supported operation removes the claim before this transaction obtains the write lock, the command observes no claim and returns `CLAIM_NOT_FOUND` without changing task state.
- If a different owner is observed, the command returns `CLAIM_NOT_OWNED` with that current claim. Busy-timeout exhaustion remains an operational database failure rather than a fabricated ownership result.
- Every validation, persistence, or hydration failure rolls back without partial claim removal.

### Success output

- JSON returns the existing E2-S2 normalized full-task aggregate with `claim: null`; it does not wrap or extend the aggregate with availability data.
- Human output concisely confirms the task ID and released agent display name.
- Human success does not include the removed `claimedAt`; the timestamp is deleted with the claim and is not persisted elsewhere.

### Errors and exits

| Code | Exit | Condition and stable details |
|---|---:|---|
| `CLAIM_NOT_FOUND` | 3 | The task has no active claim; details are `{taskId}` |
| `CLAIM_NOT_OWNED` | 5 | Another agent owns the claim; details are `{taskId, agent: {id, displayName}, claimedAt}` |
| `AGENT_NOT_FOUND` | 3 | The supplied UUID is not registered |
| `TASK_NOT_FOUND` | 3 | The supplied task ID does not exist |

Syntax and malformed UUID failures exit `2`. Repository, database, permission, busy-timeout, and unexpected failures retain shared behavior. Every failure leaves claim and task state unchanged.

## Functional acceptance criteria

1. A registered agent can remove its own active claim from a specified task.
2. Unclaiming a task with no claim returns `CLAIM_NOT_FOUND`; unclaiming another agent's claim returns `CLAIM_NOT_OWNED` with the current claimant identity and claim time.
3. Validation follows the documented precedence and no rejected operation changes claim or task state.
4. Successful unclaim removes only the claim and preserves status, archive state, content, relationships, and task mutation metadata.
5. The committed claim removal immediately changes the authoritative derived predicate: a subsequent E5-S1 claim succeeds exactly when archive, completion, and dependency conditions also permit it. Direct availability-query output remains E4-S3 scope.
6. JSON success reuses the normalized full-task aggregate with `claim: null`; human success identifies the task and released owner without adding a separate availability result.
7. Concurrent removal or ownership observations produce current-state `CLAIM_NOT_FOUND` or `CLAIM_NOT_OWNED` results and never remove a claim belonging to another agent.
8. Human and JSON errors use the documented codes, details, exits, and no-write guarantees.

## Non-functional acceptance criteria

1. Owner validation, exact claim deletion, response hydration, and commit are one SQLite transaction and cannot leave a partial result.
2. The operation is safe across concurrent local processes and linked worktrees sharing the canonical database.
3. Unclaim is local and network-free and feels immediate for a personal repository under ordinary contention.
4. JSON fields remain stable camelCase and claimant timestamps retain RFC 3339 UTC representation.
5. Core owns ownership rules, transactions, and typed errors; CLI owns parsing and rendering.

## Verification

- Unclaim an owned active claim and verify the claim row is removed while all task fields and mutation metadata remain unchanged.
- Unclaim owned tasks that are incomplete, completed, dependency-blocked, and otherwise available; verify status and dependencies never prevent owner release.
- Exercise malformed UUID, missing agent, missing task, no claim, and foreign claim paths with exact precedence, details, exits, and no state changes.
- Verify a released otherwise-available task can be claimed through E5-S1, while released completed or dependency-blocked tasks remain unavailable for their existing reason. E4-S3 later adds direct availability-query regression coverage.
- Coordinate concurrent owner release with competing removal or claim-changing operations and verify no foreign or replacement claim is deleted.
- Inject deletion and response-hydration failures and verify transaction rollback preserves the original claim.
- Snapshot human and JSON success/error output and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- User force-unclaim of stale or foreign work; E5-S4.
- Selecting or claiming the next available task; E5-S2.
- Adding an availability projection to unclaim output; E4-S3 owns availability queries.
- Automatic claim expiry, heartbeats, transfer, or daemon-based coordination.
- Changing task status or mutation metadata as a consequence of claim release.
- Archive-driven claim release; E2-S5 consumes the authoritative release behavior in its own archive transaction.

## Implementation task

See [E5-S3-T1: Implement owner-controlled task unclaim](../tasks/E5-S3-T1-implement-owner-controlled-task-unclaim.md).
