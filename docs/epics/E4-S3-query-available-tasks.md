---
id: E4-S3
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1
  - E4-S2
  - E5-S1
---

# E4-S3: Query available tasks

## Outcome

Users and coding agents can inspect the repository's currently available work through a deterministic read-only query without mistaking the result for ownership.

## User story

As a coding agent, I want to query unblocked and unclaimed tasks so that I do not select unavailable work.

## Command

```text
tbtm task available [--status <code>]... [--type <type>]... [--tag <tag>]... [--json]
```

The command is informative only. Its result may become stale immediately; agents that need ownership must use an atomic claim command rather than relying on query results.

## Product decisions

### Authoritative availability

- A task is returned only when it is non-archived, its current status is not completed, it has no active claim, and every direct upstream dependency is effectively completed through completed status or archive.
- Zero upstream dependencies are satisfied. Every upstream must be satisfied when one or more exist.
- Availability is derived from current task, status, claim, and dependency state. No available, blocked, or dependency-satisfaction flag is persisted.
- Type, hierarchy position, estimate, and other task attributes do not independently affect availability.
- This story lists available tasks but does not explain why excluded tasks are unavailable; blocker explanation remains E4-S4 scope.

### Filters

- Status, type, and tag filters are repeatable. Values within one filter group use OR; different groups use AND.
- Filters narrow the available set and can never weaken or bypass the complete availability predicate.
- Status values are immutable machine codes. Every distinct supplied code is validated; any unknown code returns `STATUS_NOT_FOUND` rather than an empty result.
- A known completed status is a valid filter and naturally produces an empty successful result because completed tasks cannot be available.
- Type uses the fixed task-type tokens. An invalid type returns `INVALID_TASK_TYPE`.
- Tag matching is exact and case-sensitive.
- Priority ranges, estimate ranges, hierarchy filters, full-text search, pagination, and caller-selected sorting are outside the MVP command.

### Ordering and output

- Results always order by priority descending, creation time ascending, then stable task ID ascending after predicate and filters are applied.
- JSON uses the shared success envelope and the existing E2-S2 compact `TaskListItem` projection. Available items therefore retain `archived: false`, `claim: null`, and a status with `completed: false`; no separate `available` field is added.
- Human output reuses the compact task-list table. An empty human result is exactly `No available tasks found.`
- An empty JSON result is `data: []`. Empty results succeed with exit code `0` and are distinct from failures.
- Output does not include blocker explanations, recursive relationships, or a claim guarantee.

### Read and reuse semantics

- The query opens the canonical repository database read-only, applies no migration, creates no repository artifacts, and performs no mutation.
- Each invocation reads from one SQLite snapshot so task rows, statuses, claims, dependencies, tags, filters, and ordering are internally consistent even when another local process commits concurrently.
- Core owns one reusable selection primitive that accepts a caller-supplied SQLite connection or transaction. The standalone command uses it in a read snapshot; E5-S2 can later use the identical predicate, filters, and ordering inside its own immediate write transaction without a separate stale read.
- Filtering and ordering are performed by SQLite where practical, and claim/tag hydration uses bounded queries rather than one query per task.
- The command remains local, network-free, and consistent across linked worktrees that resolve the same canonical database.

### Errors and exits

| Code | Exit | Condition |
|---|---:|---|
| `INVALID_TASK_TYPE` | 2 | Any supplied type token is invalid |
| `STATUS_NOT_FOUND` | 3 | Any distinct supplied status code does not exist |

Conflicting or malformed arguments exit `2`. Repository, incomplete-database, database, permission, and unexpected failures retain the shared error and exit contracts. Failures never return a partial result or modify repository state.

## Functional acceptance criteria

1. The command returns only non-archived, incomplete, unclaimed tasks whose every direct upstream dependency is effectively completed.
2. Zero-upstream tasks qualify when their other availability conditions hold; a mix of completed-status and archived upstreams satisfies dependencies only when no unresolved upstream remains.
3. Repeatable status, type, and tag filters use OR within a group and AND across groups without admitting any otherwise-unavailable task.
4. Unknown statuses and invalid types return the documented stable errors, while a known completed-status filter returns a successful empty result.
5. Every result is ordered by priority descending, creation time ascending, then task ID ascending.
6. JSON returns the E2-S2 compact task-list projection without an availability or blocking extension; human output uses the compact table.
7. Empty human and JSON results use the documented forms and exit `0`, distinct from validation and operational failures.
8. Query results are explicitly informative and do not reserve, claim, or guarantee continued availability.

## Non-functional acceptance criteria

1. The command is read-only, applies no migrations, creates no artifacts, and reads all result components from one SQLite snapshot.
2. Core exposes the exact selection behavior through a caller-owned connection/transaction API reusable by E5-S2.
3. Query count is bounded rather than growing once per returned task, with predicate, filters, and ordering executed by SQLite where practical.
4. Common queries feel immediate for a personal repository containing thousands of tasks and perform no network access.
5. JSON fields remain stable camelCase, timestamps remain RFC 3339 UTC, and row and collection ordering are deterministic.
6. Core owns availability/filter rules and database reads; CLI owns argument parsing and human/JSON rendering.

## Verification

- Exercise each availability condition independently and in combination: active versus archived, incomplete versus completed, unclaimed versus claimed, zero upstreams, one unresolved upstream, mixed upstream states, and all upstreams resolved by status or archive.
- Verify each filter independently, repeated OR values, cross-group AND behavior, exact tag case, unknown statuses, invalid types, and a known completed-status filter.
- Cover every ordering key and tie-break combination after filtering.
- Snapshot compact human/JSON output, including invariant `archived`, `claim`, and status completion fields, exact empty output, errors, and exit codes.
- Change claims, upstream completion/archive state, and dependency edges between invocations to prove availability is derived rather than cached.
- Coordinate concurrent writes with an open query to prove each result is one internally consistent snapshot while acknowledging that the completed result may immediately become stale.
- Inspect query behavior for bounded query count and no per-result task/tag/claim lookup growth.
- Run from the main and a linked worktree and verify both observe the same committed canonical state.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Explaining why an arbitrary task is unavailable or returning unresolved upstream IDs; E4-S4.
- Selecting and claiming the next task atomically; E5-S2.
- Claiming a specified task, releasing claims, or changing claim ownership; E5.
- Persisted availability caches, background recomputation, polling, or refresh notifications.
- Custom sorting, pagination, priority/estimate ranges, hierarchy filters, search, or board-specific projections.

## Implementation task

See [E4-S3-T1: Implement available-task querying](../tasks/E4-S3-T1-implement-available-task-querying.md).
