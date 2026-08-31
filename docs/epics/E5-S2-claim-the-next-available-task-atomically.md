---
id: E5-S2
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E4-S3
  - E5-S1
---

# E5-S2: Claim the next available task atomically

## Outcome

A registered coding agent can atomically select and claim the highest-ranked currently available task, including across linked worktrees, without relying on a stale prior query.

## User story

As a coding agent, I want selection and claim in one operation so that a prior availability query cannot become stale.

## Command

```text
tbtm task claim-next --agent <uuid> [--status <code>]... [--type <type>]... [--tag <tag>]... [--json]
```

The command is non-interactive and requires a registered agent UUID. It does not accept the logical `user`, an agent display name, or an implicit current agent.

## Product decisions

### Selection and filters

- Candidate selection reuses E4-S3's complete live availability predicate: non-archived, incomplete status, no active claim, and every direct upstream dependency effectively completed by completed status or archive.
- Candidates use the fixed ordering of priority descending, creation time ascending, then task ID ascending. The first remaining candidate is selected.
- Repeatable status, type, and tag filters have exactly the E4-S3 semantics: OR within a group, AND across groups, exact case-sensitive tag matching, and no ability to weaken availability.
- Every distinct status code is validated. Unknown statuses return `STATUS_NOT_FOUND`; a known completed status is valid and naturally produces an empty result. Invalid task types return `INVALID_TASK_TYPE`.
- Priority ranges, estimates, hierarchy filters, full-text search, custom sorting, pagination, fallback filter sets, and retry loops are outside the MVP.

### Validation precedence

- Validation precedence is command syntax and UUID/type parsing, registered-agent lookup, validation of every distinct status code, then candidate selection.
- If the agent does not exist and a status filter is also unknown, `AGENT_NOT_FOUND` wins because agent resolution precedes status lookup.
- An empty candidate set is a successful result, not an error. It does not produce `TASK_NOT_AVAILABLE` or `CLAIM_CONFLICT` because no specified task was promised.

### Atomic concurrency behavior

- Core applies compatible pending migrations, starts one `TransactionBehavior::Immediate` transaction, resolves the agent, validates filters, selects the first candidate, generates the claim timestamp, inserts the claim, hydrates the result, and commits.
- Selection occurs only after the immediate write lock is acquired. A waiter recalculates availability and ordering from committed current state rather than retaining a candidate selected before the lock.
- With two contenders and at least two eligible tasks, the first transaction claims the highest-ranked task and the second claims the next current candidate. With one eligible task, exactly one transaction claims it and the other returns empty success.
- `CLAIM_CONFLICT` is not an outcome of this command. Busy-timeout exhaustion remains an operational database failure because the command could not safely observe current state.
- SQLite's one-writer behavior is intentional. All linked worktrees use the E1-S5 canonical database, so immediate transactions serialize writers across processes and worktrees.
- The MVP retains `journal_mode = DELETE`. WAL is not required for correctness and would not permit simultaneous write transactions. The transaction contains no prompt, network call, or slow external work after lock acquisition.
- Claim-table uniqueness remains a defensive invariant. Any validation, selection, insertion, hydration, or commit failure leaves no partial claim and does not mutate task data.

### Claim and task lifecycle

- A successful operation creates the same authoritative active-claim row as E5-S1 with the supplied agent and one RFC 3339 UTC `claimedAt` timestamp.
- Claiming does not change status, archive state, task content, relationships, `updatedAt`, or `updatedBy`.
- No candidate is an observed success with no writes. The command does not reserve work, automatically change status, transfer claims, or retry with weakened filters.

### Output

- Non-empty JSON uses the shared success envelope containing the E2-S2 normalized full-task aggregate with the new claim populated, including direct relationship context.
- Non-empty human output reuses E5-S1's exact claim-success renderer: `Claimed task: <id>`, `Agent: <display-name>`, and `Claimed at: <timestamp>` on separate lines.
- Empty JSON uses the shared success envelope with `data: null`. Empty human output is exactly `No available task to claim.` Both exit `0`.

### Errors and exits

| Code | Exit | Condition |
|---|---:|---|
| `INVALID_TASK_TYPE` | 2 | Any supplied type token is invalid |
| `AGENT_NOT_FOUND` | 3 | The supplied UUID is not registered |
| `STATUS_NOT_FOUND` | 3 | Any distinct supplied status code does not exist |

Malformed or missing arguments and malformed UUIDs exit `2`. Database, busy-timeout, permission, repository, and unexpected failures retain shared behavior. Exit `4` is not produced. Every failure returns the shared error envelope and leaves claims and task state unchanged.

## Functional acceptance criteria

1. The command selects only a task satisfying the complete current E4-S3 availability predicate and all supplied filters.
2. It selects the first candidate by priority descending, creation time ascending, and task ID ascending.
3. Status, type, and tag filters preserve E4-S3 validation and OR/AND semantics without weakening availability.
4. Validation follows the documented precedence, including `AGENT_NOT_FOUND` before an unknown status when both are present.
5. Selection and claim creation form one atomic outcome; no candidate returns the documented successful empty result without writing.
6. Two serialized contenders claim distinct ordered candidates when at least two exist; with one candidate, one claims it and the other returns empty success rather than a conflict.
7. Success creates the E5-S1 claim identity and timestamp while preserving status, archive state, relationships, and task mutation metadata.
8. Non-empty and empty human/JSON results use the documented shared projections, exact messages, and exits.

## Non-functional acceptance criteria

1. Agent/filter validation, candidate selection, timestamp generation, claim insertion, response hydration, and commit occur in one immediate SQLite transaction.
2. Every failure rolls back; claim uniqueness and database integrity hold across concurrent processes and linked worktrees.
3. The operation uses the canonical shared database with `journal_mode = DELETE`, keeps the write-lock scope free of prompts and external work, and treats busy timeout as an operational failure.
4. Core reuses E4-S3's caller-owned selector rather than duplicating availability, filtering, or ordering rules.
5. The command is local and network-free, uses bounded database queries, and feels immediate for a personal repository under ordinary contention.
6. JSON fields remain camelCase, timestamps remain RFC 3339 UTC, and all row and collection ordering is deterministic.
7. Core owns validation, selection, transaction, and claim rules; CLI owns parsing and rendering.

## Verification

- Cover every availability branch, all filter combinations and validation failures, every ordering tie-break, and empty-after-filter behavior.
- Verify the selected claim contains the requested agent and one timestamp while all task fields and mutation metadata remain unchanged.
- Race two connections/processes with multiple candidates and with one candidate; assert the documented distinct-claim and success-plus-empty outcomes.
- Repeat contention from linked worktrees and inspect the canonical database for unique claim rows and integrity.
- Exhaust the busy timeout separately and verify an operational error rather than conflict or empty success.
- Inject insertion and post-insert hydration failures and verify rollback leaves no claim.
- Snapshot exact non-empty and empty human/JSON output and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Querying without ownership or explaining why a particular excluded task is unavailable; E4-S3 and E4-S4.
- Claiming a specified task and returning claim conflicts; E5-S1.
- Claim transfer, expiry, heartbeats, force-unclaim, or implicit agent state.
- Automatic status changes, persisted availability caches, custom sorting, pagination, or retrying with different filters.
- Switching the repository to WAL mode or adding daemon-based coordination.

## Implementation task

See [E5-S2-T1: Implement atomic next-available-task claiming](../tasks/E5-S2-T1-implement-atomic-next-available-task-claiming.md).
