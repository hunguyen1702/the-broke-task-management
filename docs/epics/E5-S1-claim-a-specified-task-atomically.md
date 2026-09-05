---
id: E5-S1
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E1-S3
  - E2-S1
  - E3-S1
  - E4-S2
---

# E5-S1: Claim a specified task atomically

## Outcome

A registered coding agent can acquire one specified available task without another local process or linked worktree claiming the same task concurrently.

## User story

As a coding agent, I want to atomically claim an available task so that another agent cannot claim it simultaneously.

## Command

```text
tbtm task claim <task-id> --agent <uuid> [--json]
```

`--agent` is required. The logical `user` actor does not acquire claims through this story, and display names are not accepted as agent identity.

## Product decisions

### Availability and validation

- A claim succeeds only when the task is non-archived, its current status is not completed, it has no active claim, and every direct upstream dependency is effectively completed by completed status or archive.
- Availability is derived from authoritative current task, status, claim, and dependency state. No available or blocked flag is persisted.
- Validation precedence is command syntax and UUID parsing, registered agent, task existence, existing claim, archive state, status completion, unresolved dependencies, then claim insertion.
- Checking an existing claim before other availability reasons ensures any attempt against currently owned work returns the dedicated conflict contract. This includes the current owner trying to claim the same task again and tasks that are also completed or dependency-blocked.
- A malformed UUID is command validation. An unknown UUID and unknown task retain the shared `AGENT_NOT_FOUND` and `TASK_NOT_FOUND` behavior.

### Claim identity and lifecycle

- An active claim contains the authoritative registered agent UUID and one UTC `claimedAt` timestamp.
- A task has at most one active claim. Claiming never transfers or overwrites an existing claim.
- Repeating claim as the existing owner is not an idempotent success; it returns `CLAIM_CONFLICT` and preserves the original timestamp.
- Claim is independent of task status and mutation attribution. A successful claim does not change task status, `updatedAt`, or `updatedBy`.
- Claims do not expire automatically. Unclaim, force-unclaim, archive release, next-task selection, and claim/status lifecycle behavior remain owned by later stories.

### Atomic concurrency behavior

- A sequential compatible migration introduces the authoritative active-claim relation with one row at most per task and a required registered-agent reference.
- Claim uses the shared canonical SQLite database, including from linked worktrees.
- After applying pending compatible migrations, core starts an immediate write transaction. Agent resolution, task lookup, the complete availability recheck, claim insertion, response construction, and commit occur under that transaction.
- A contender waiting behind a committed winner rechecks current state after acquiring the write lock and returns `CLAIM_CONFLICT`; it never overwrites the winner.
- SQLite constraints independently protect the one-claim-per-task invariant. Any failed validation or insert rolls back without a partial claim.
- If lock acquisition exceeds the configured busy timeout, the operation returns the inherited operational database error. It does not claim that another agent won when current ownership could not be read safely.

### Success output

- JSON returns the shared full-task aggregate with `claim` populated as `{agent: {id, displayName}, claimedAt}`. Task fields, direct relationship summaries, and collection shapes remain compatible with E2-S2 and E4-S2.
- Human output confirms the task ID, agent display name, and claim time.
- Claim success does not add a separate availability projection or recursively explain relationships.

### Errors and exits

| Code | Exit | Condition and stable details |
|---|---:|---|
| `CLAIM_CONFLICT` | 4 | The task already has a claim; details contain `taskId`, `agent: {id, displayName}`, and `claimedAt` |
| `TASK_NOT_AVAILABLE` | 2 | The unclaimed task is archived, completed, or dependency-blocked; details contain `taskId` and `reason` |
| `AGENT_NOT_FOUND` | 3 | The supplied UUID is not registered |
| `TASK_NOT_FOUND` | 3 | The supplied task ID does not exist |

`TASK_NOT_AVAILABLE.reason` is exactly `archived`, `completed`, or `dependencies_blocked`. Dependency-blocked details additionally contain `unresolvedUpstreamIds`, sorted by task ID ascending. The other two reasons omit that field. Syntax and malformed UUID failures exit `2`; repository, database, permission, and unexpected failures retain shared behavior. Every failure leaves claim and task state unchanged.

## Functional acceptance criteria

1. A registered agent can claim a specified task only when the complete current availability predicate is true.
2. Claim stores exactly one agent reference and one UTC claim timestamp without changing task status or mutation metadata.
3. A task can have at most one active claim, and neither another agent nor the current owner can overwrite or refresh it.
4. Concurrent claims against the same initially available task produce one persisted winner; a contender that observes the winner returns `CLAIM_CONFLICT` with the preserved winner identity and timestamp.
5. Archived, completed, and dependency-blocked unclaimed tasks return deterministic `TASK_NOT_AVAILABLE` reasons; unresolved upstream IDs are complete and sorted.
6. Existing claim takes precedence over archive, completion, and dependency-blocking errors.
7. JSON success returns the existing full-task contract with a populated claim, while human success identifies the task, agent display name, and claim time.
8. Malformed identity, missing agent, missing task, claim conflict, unavailable state, and operational failures use the documented codes, details, exits, and no-write guarantees.

## Non-functional acceptance criteria

1. Availability validation and claim creation are one SQLite transaction and cannot leave a partial claim after failure or interruption.
2. Database constraints and serialized write validation preserve claim uniqueness across concurrent local processes and linked worktrees.
3. Claim is local and network-free and feels immediate for a personal repository under ordinary contention.
4. Timestamps use RFC 3339 UTC, JSON fields remain stable camelCase, and unresolved upstream ordering is deterministic.
5. Core owns claim rules, transactions, and typed errors; CLI owns parsing and rendering.

## Verification

- Claim tasks with zero dependencies and with completed-status, archived, mixed-resolved, and unresolved upstream sets.
- Reject archived, completed, dependency-blocked, already-claimed, and same-agent re-claim paths with exact precedence, details, exits, and no state changes.
- Verify claimed-plus-completed and claimed-plus-blocked tasks return the existing-claim conflict.
- Query SQLite and task output to verify one claim row, correct agent and timestamp, populated detail/list claim summaries, and unchanged task status, `updatedAt`, and `updatedBy`.
- Inject claim insertion failure and verify transaction rollback.
- Race two separate connections, processes, and linked worktrees against one task; assert one winner, one stored claim, one deterministic conflict after winner observation, and no database corruption.
- Exercise busy-timeout exhaustion separately and verify it remains an operational database failure rather than a fabricated claim conflict.
- Snapshot human and JSON success/error output and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Querying available tasks or explaining arbitrary blockers; E4-S3 and E4-S4.
- Selecting and claiming the next available task atomically; E5-S2.
- Owner unclaim and user force-unclaim; E5-S3 and E5-S4.
- Automatic expiry, heartbeats, claim transfer, or implicit current-agent state.
- Claim creation by the logical `user` actor.
- Automatic status changes caused by claim creation.

## Implementation task

See [E5-S1-T1: Implement atomic specified-task claiming](../tasks/E5-S1-T1-implement-atomic-specified-task-claiming.md).
