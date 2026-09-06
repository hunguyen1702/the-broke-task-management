---
id: E3-S4
kind: epic
planning_status: done
implementation_status: ready
contract_depends_on:
  - E3-S2
---

# E3-S4: Delete an unused custom status

## Outcome

Users can remove obsolete custom workflow columns without deleting or orphaning task data.

## User story

As a user, I want to remove an unused custom status so that obsolete board columns disappear safely.

## Command

```text
tbtm status delete <code> [--json]
```

Status deletion is a logical-user operation and does not accept `--agent`.

## Product decisions

### Eligibility and identity

- The source is resolved by the exact immutable status code defined by E3-S2.
- Only custom statuses may be deleted. Every default status is rejected even when no task uses it.
- A custom status is unused only when no task row references its UUID. Both active and archived tasks count as usage.
- Completion semantics do not affect deletion eligibility. E3-S3 is therefore not a contract dependency.
- An eligible delete needs no confirmation because it cannot remove or rewrite task data.

### Ordering and data preservation

- Successful deletion removes exactly the selected status row.
- Remaining statuses are compacted to contiguous zero-based `displayOrder` values while preserving their relative order.
- Task rows, task status assignments, claims, relationships, comments, and all other repository data remain unchanged.
- Failed deletion leaves the status collection, its ordering, and all task data unchanged.

### Reads, writes, and concurrency

- Lookup, default/custom validation, usage counting, deletion, order compaction, and result capture occur in one immediate SQLite transaction.
- The transaction prevents a concurrent supported writer from assigning a task to the status between the usage check and deletion. A concurrent task creation or status change either commits before the check and blocks deletion, or proceeds after the deletion and can no longer resolve the removed code.
- The mutation uses the canonical shared repository across linked worktrees and applies compatible pending migrations before beginning normal status mutation behavior.
- Busy-timeout exhaustion and repository, database, migration, permission, and unexpected failures retain shared operational errors and exit behavior.

### Output

Successful JSON output uses the shared envelope and returns the deleted status object directly as `data`:

```json
{
  "id": "UUID",
  "code": "review",
  "name": "Review",
  "completed": false,
  "displayOrder": 2,
  "isDefault": false
}
```

This is the status snapshot captured immediately before deletion, so `displayOrder` is its former position. The response does not include the resulting status list. Human output concisely identifies the deleted code and name.

### Errors and exits

| Code | Exit | Stable details and condition |
|---|---:|---|
| `STATUS_NOT_FOUND` | 3 | `{code, role: "source"}`; the exact source code does not exist. |
| `STATUS_DEFAULT_IMMUTABLE` | 5 | `{code}`; the source is a default status. |
| `STATUS_IN_USE` | 4 | `{code, taskCount}`; one or more active or archived tasks reference the source status. |

The human message for `STATUS_DEFAULT_IMMUTABLE` is command-neutral: `default status is immutable: <code>`. This wording also applies to E3-S2 rename failures without changing their stable code, details, or exit. Clap handles a missing `<code>` as exit `2`. Validation precedence is source lookup, default-status rejection, then usage rejection. Every error is non-mutating.

## Functional acceptance criteria

1. A user can delete an unused custom status by exact code in human or JSON mode without confirmation.
2. Success returns the complete pre-delete status snapshot; human output identifies its code and name.
3. Deletion removes exactly that status and compacts remaining positions to a contiguous zero-based sequence without changing relative order.
4. An unknown source fails with `STATUS_NOT_FOUND` and `{code, role: "source"}`.
5. Every default status fails with `STATUS_DEFAULT_IMMUTABLE`, including an otherwise-unused default status.
6. A custom status referenced by any active or archived task fails with `STATUS_IN_USE` and an exact count across both scopes.
7. Failed deletion preserves all statuses, ordering, tasks, assignments, and related repository data.
8. Outputs, errors, details, and exits match the documented human and JSON contracts.

## Non-functional acceptance criteria

1. Core owns lookup, eligibility, usage counting, transactionality, compaction, and typed errors; CLI owns parsing, rendering, envelopes, and exit mapping.
2. Delete is atomic and safe across concurrent processes and linked Git worktrees sharing the canonical database.
3. The write path applies compatible pending migrations and retains the shared busy-timeout policy; no read-only delete preview is added.
4. Stable status identity and camelCase JSON conventions are preserved, and the operation remains local and network-free.
5. Ordinary deletion remains immediate for a personal repository with a small workflow and task collection.

## Verification

- Delete unused custom statuses from the beginning, middle, and end of the board; verify the returned pre-delete snapshot and compact remaining order.
- Reject unknown, default, active-task-used, archived-task-used, and mixed-use statuses with exact errors, counts, details, and exits.
- Prove every rejection and injected database failure rolls back status deletion and order updates without changing task data.
- Race deletion against task creation or task status assignment from the main and a linked worktree; verify a coherent winner and no dangling status reference.
- Verify compatible-migration behavior, busy-timeout handling, public help, human/JSON output, and absence of `--agent` or confirmation prompts.

## Out of scope

- Reassigning or deleting tasks to make a status unused.
- Force deletion, replacement-status selection, bulk deletion, or deletion confirmation.
- Deleting default statuses or changing status codes, names, ordering, or completion semantics through this command.
- Status deletion history, undo, remote synchronization, authentication, and multi-user permissions.
- Acceptance-scenario creation, review, approval, or execution in this workflow.

## Implementation task

See [E3-S4-T1: Implement safe custom status deletion](../tasks/E3-S4-T1-implement-safe-custom-status-deletion.md).
