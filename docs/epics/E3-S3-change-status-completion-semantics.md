---
id: E3-S3
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E3-S2
  - E4-S3
---

# E3-S3: Change status completion semantics

## Outcome

The logical user can change whether a repository status represents completed work, with a deterministic warning about current task and dependency impact before any consequential change commits.

## User story

As a user, I want to mark a status completed or incomplete so that dependency behavior matches its meaning.

## Command

```text
tbtm status set-completed <code> --completed <true|false> [--yes] [--json]
```

Status configuration is a logical-user operation. The command does not accept `--agent`.

## Product decisions

### Repository-scoped status definition

- Completion is a property of a status definition stored in the current canonical repository, not a value copied onto each task and not a machine-global setting.
- Every task refers to a status by stable UUID. Changing `completed` therefore immediately changes completion semantics for every task in the same repository that references that status.
- Main and linked worktrees share the change through their canonical repository database. Statuses with the same code in another repository are independent and unaffected.
- Both default and custom statuses may change completion semantics. UUID, code, name, display order, and default/custom identity remain unchanged.
- The returned `status` is the shared repository-level status object after mutation. Task output continues to embed the current projection of its referenced repository status.

### Effective completion and claims

- For an active task, effective completion follows its status `completed` value. An archived task remains effectively completed regardless of status, so archived tasks are excluded from impact.
- A completion change is visible immediately to dependency resolution, `task available`, specified claiming, and next-task claiming through the authoritative derived predicates.
- No availability, blocking, or effective-completion cache is written.
- Active claims are never created, released, transferred, or otherwise changed. A claimed task may become completed or dependency-blocked while retaining its claim.

### Impact model

Impact compares the repository state immediately before and after the proposed boolean change and contains two task-ID-ordered arrays:

```json
{
  "statusTasks": [
    {
      "taskId": "TBTM-task-1234abcd",
      "title": "Implement review",
      "claim": null,
      "availableBefore": true,
      "availableAfter": false
    }
  ],
  "downstreamTasks": [
    {
      "taskId": "TBTM-task-5678efab",
      "title": "Ship feature",
      "claim": null,
      "availableBefore": false,
      "availableAfter": true,
      "unresolvedUpstreamTaskIdsBefore": ["TBTM-task-1234abcd"],
      "unresolvedUpstreamTaskIdsAfter": []
    }
  ]
}
```

- `statusTasks` contains every active task that directly references the changed status. Each item reports its shared nullable claim summary and authoritative availability before and after the proposal.
- `downstreamTasks` contains each active task that directly depends on at least one `statusTask` and is incomplete before or after the transition. A downstream task is excluded only when archived or completed on both sides.
- Claimed downstream tasks and tasks that remain blocked by another unresolved upstream are retained because their dependency resolution changes even when overall availability stays false.
- A downstream item reports its shared nullable claim summary, authoritative availability on both sides, and complete task-ID-ordered unresolved direct-upstream IDs on both sides.
- A task may appear in both arrays when it uses the changed status and directly depends on another affected status task. Downstream items are deduplicated even when they depend on multiple affected status tasks.
- Arrays and unresolved-upstream arrays are ordered by task ID. Impact never includes recursive-only descendants.
- Both transition directions use the same model: incomplete to completed may make source tasks unavailable and downstream tasks unblocked; completed to incomplete may make source tasks available and downstream tasks blocked.

### Confirmation and concurrency

- Setting an existing status to its current value is a successful no-op: no prompt, empty impact, no database write, and unchanged status data.
- A real change with empty impact commits without prompting. Impact is non-empty when either task array is non-empty.
- Interactive human execution renders the proposed old/new completion state and both impact groups, then prompts unless `--yes` is supplied. Declining prints exactly `Status completion change cancelled.`, exits `0`, and changes nothing.
- `--yes` is explicit advance confirmation and is accepted with empty or non-empty impact. JSON and non-interactive execution never prompt; a non-empty impact requires `--yes`.
- The write transaction recalculates the current impact. Without confirmation, a latest non-empty impact returns `CONFIRMATION_REQUIRED` and commits nothing, including an empty-to-non-empty race.
- Once any non-empty impact has been confirmed, the MVP accepts that details may change before the transaction. It does not use a snapshot token or request repeated confirmation. Success returns actual transactional impact.
- Lookup, current-value detection, impact calculation, confirmation enforcement, status mutation, and result hydration are consistent within one immediate SQLite transaction. Concurrent supported writers cannot expose a partially changed definition.

### Output

Successful JSON data is:

```json
{
  "status": {
    "id": "UUID",
    "code": "review",
    "name": "Review",
    "completed": true,
    "displayOrder": 2,
    "isDefault": false
  },
  "impact": {
    "statusTasks": [],
    "downstreamTasks": []
  }
}
```

- `status` is the shared E3-S2 status object from the current repository after the operation.
- `impact` is the actual transactionally calculated impact. A no-op returns the unchanged status and empty arrays.
- Human preflight and success output identify the status code/name, old and new completed states, and both impact groups with claim, availability, and unresolved-upstream information.
- JSON stdout contains only the shared envelope; prompts and warning prose never contaminate it.

### Errors and exits

| Code | Exit | Stable details and condition |
|---|---:|---|
| `CONFIRMATION_REQUIRED` | 2 | The exact latest impact payload; a non-empty change was not confirmed. |
| `STATUS_NOT_FOUND` | 3 | `{code, role: "source"}`; the source status does not exist. |

Clap accepts only its documented boolean syntax for `--completed`; missing or malformed arguments exit `2`. Status lookup precedes current-value handling, so an unknown code never becomes a no-op. Repository, database, migration, permission, contention, and unexpected failures retain shared envelopes and exit behavior. Every failure preserves statuses, tasks, claims, relationships, and task metadata.

## Functional acceptance criteria

1. The logical user can set either a default or custom repository status to completed or incomplete while preserving every other status field.
2. The change affects every task referencing that status in the current canonical repository and no task or status in another repository.
3. Effective completion, dependency resolution, available-task queries, and claim eligibility immediately use the changed value without persisted derived state.
4. Existing claims remain byte-for-byte unchanged even when their task becomes completed, incomplete, blocked, or otherwise available.
5. Impact deterministically reports all qualifying status tasks and direct downstream tasks with accurate before/after availability, claims, and unresolved direct upstreams; archived and recursive-only tasks are excluded.
6. Overlapping source/downstream membership, multiple affected upstreams, both transition directions, and tasks that remain unavailable are represented by the documented deduplicated arrays.
7. Non-empty impact is displayed or returned before an unconfirmed change can commit; TTY, `--yes`, JSON, non-interactive, cancellation, and concurrent impact changes follow the documented behavior.
8. A same-value request is a no-write success, while an unknown status and malformed boolean return stable failures.
9. Human and JSON success, warning, cancellation, error, ordering, and exit behavior are deterministic.

## Non-functional acceptance criteria

1. Core owns status lookup, impact calculation, authoritative predicates, confirmation enforcement, mutation, and transaction boundaries; CLI owns parsing, prompts, rendering, envelopes, and exit mapping.
2. Mutation and returned impact are atomic and consistent across concurrent processes and linked worktrees sharing the canonical database.
3. Shared status, task, claim, dependency, blocking, and availability models are reused instead of duplicated.
4. Queries are bounded rather than growing once per impacted task; ordinary operation remains immediate for a personal repository with thousands of tasks.
5. JSON remains camelCase, ordering is deterministic, SQLite access is parameterized, and the operation is local and network-free.

## Verification

- Exercise incomplete-to-completed and completed-to-incomplete changes for default and custom statuses, including no-use, single-task, and many-task cases.
- Cover source tasks that are available, claimed, dependency-blocked, and archived; verify archived exclusion and exact before/after availability.
- Cover downstream tasks that become available/unavailable, retain another blocker, retain a claim, overlap `statusTasks`, depend on multiple affected sources, are completed on one side, are completed on both sides, are archived, or are recursive-only.
- Exercise TTY accept/decline, `--yes`, JSON, non-interactive execution, same-value no-op, unknown code, malformed boolean, and exact outputs/exits.
- Race empty-to-non-empty and non-empty-to-empty/detail-changed impact through the main and a linked worktree; verify confirmation enforcement, actual returned impact, unchanged claims, rollback, and one coherent committed status value.
- Run formatting, lint, and workspace tests after implementation.

## Out of scope

- Per-task completion overrides or copying `completed` onto task rows.
- Machine-global, user-global, or cross-repository status configuration.
- Changing a task's assigned status, any status code/name/order/default identity, task claims, relationships, content, archive state, or comments.
- Persisted availability/blocking caches, recursive impact presentation, notifications, event history, snapshot tokens, or repeated confirmation.
- Status deletion; E3-S4 owns it. Visual Studio Code integration remains E8 scope.
- Acceptance-scenario creation, review, approval, or execution in this workflow.

## Implementation task

See [E3-S3-T1: Implement repository status completion changes](../tasks/E3-S3-T1-implement-repository-status-completion-changes.md).
