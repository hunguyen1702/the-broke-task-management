---
id: E4-S4
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E4-S2
  - E4-S3
---

# E4-S4: Explain blocking

## Outcome

Users and agents can inspect every current reason a task is unavailable and follow its unresolved dependency context without confusing dependency blocking with hierarchy.

## User story

As a user or agent, I want to know why a task is unavailable so that I can resolve its blockers.

## Command

```text
tbtm task blockers <task-id> [--json]
```

The command is read-only and informative. It does not accept an actor, mutate state, reserve work, or guarantee that the explanation remains current after the read completes.

## Product decisions

### Availability explanation

- The command applies the exact E4-S3 availability predicate and returns success for both available and unavailable tasks.
- `reasons` contains every current top-level reason rather than choosing one by validation precedence. Its fixed order is `archived`, `completed`, `claimed`, then `dependencies_blocked`.
- Archive, completed status, and an active claim are reported independently, so a task may have several simultaneous reasons.
- `dependencies_blocked` is present when at least one direct upstream is not effectively completed. Archived or completed-status upstreams satisfy their edges and are not reported.
- Availability and explanations are derived from current task, status, claim, and dependency state. No blocked, reason, or availability cache is persisted.

### Dependency context

- `direct` contains every unresolved direct upstream dependency of the requested task.
- From each direct blocker, recursive traversal follows only dependency edges to upstream tasks that are themselves not effectively completed. Traversal stops at archived or completed-status tasks.
- `recursive` contains unresolved tasks reached beyond the direct level and excludes every task already present in `direct`.
- Both collections are deduplicated and sorted by task ID. Cycles remain impossible under the E4-S2 graph invariant.
- Every dependency item reuses the E2-S2 related-task identity, type, title, and status summary and adds `blockedByTaskIds`, the sorted IDs of that item's immediate unresolved upstreams that are present in the explanation graph.
- Hierarchy is never traversed or represented. E4-S5 owns hierarchy and general relationship-map paths, directions, and presentation.

### Output

- JSON uses the shared success envelope with one explanation object containing `taskId`, `available`, `reasons`, `claim`, and `unresolvedDependencies`.
- `claim` is the shared nullable claim summary `{agent: {id, displayName}, claimedAt}`.
- An available task returns `available: true`, empty reasons and dependency collections, and `claim: null`.
- Human output for an available task is exactly `<task-id> is available.`
- Human output for an unavailable task starts with `<task-id> is unavailable.` followed by `Reasons: <comma-separated-reasons>` in fixed reason order.
- A present claim renders `Claim: <display-name> (<agent-id>) since <claimed-at>`; the line is omitted when `claim` is `null`.
- Non-empty groups render under `Direct unresolved dependencies:` and `Recursive unresolved dependencies:` respectively. Empty groups and headings are omitted.
- Each dependency line is `- <id> [<type>, <status-code>] <title>; blocked by: <comma-separated-ids>`; an empty `blockedByTaskIds` renders `blocked by: none`.
- Task IDs and collections use deterministic ordering; timestamps retain RFC 3339 UTC and JSON fields use camelCase.

Example unavailable data:

```json
{
  "taskId": "TBTM-task-1234abcd",
  "available": false,
  "reasons": ["claimed", "dependencies_blocked"],
  "claim": {
    "agent": {
      "id": "00000000-0000-0000-0000-000000000001",
      "displayName": "worker-1234abcd"
    },
    "claimedAt": "2026-08-30T08:00:00Z"
  },
  "unresolvedDependencies": {
    "direct": [
      {
        "id": "TBTM-task-2345bcde",
        "title": "Prepare schema",
        "type": "task",
        "status": {
          "code": "in_progress",
          "name": "In progress",
          "completed": false
        },
        "blockedByTaskIds": []
      }
    ],
    "recursive": []
  }
}
```

### Read consistency and reuse

- Each invocation opens the canonical repository database read-only, applies no migration, creates no repository artifacts, and reads the task, status, claim, and complete explanation graph from one SQLite snapshot.
- Core owns a typed explanation query that accepts a caller-owned connection or transaction. The standalone command owns the read snapshot; E2-S6 can reuse the same unresolved-dependency primitives inside its unarchive transaction.
- Recursive loading uses a recursive CTE or equivalent bounded graph query rather than one query per node and remains practical for personal repositories containing thousands of tasks.

### Errors and exits

| Code | Exit | Condition |
|---|---:|---|
| `TASK_NOT_FOUND` | 3 | The requested task does not exist |

Available and unavailable explanations both exit `0`. Malformed arguments exit `2`; repository, incomplete-database, database, permission, and unexpected failures retain shared codes and exits. Failures never return a partial explanation or modify repository state.

## Functional acceptance criteria

1. The command reports whether a requested task currently satisfies the exact authoritative availability predicate.
2. Every simultaneous archive, completion, claim, and unresolved-dependency reason is returned in fixed order without precedence hiding another reason.
3. Every unresolved direct upstream is identifiable, and recursive context includes only unresolved ancestors reachable through unresolved dependency edges.
4. Traversal stops at effectively completed tasks, excludes direct blockers from the recursive collection, deduplicates shared ancestors, and orders all IDs deterministically.
5. Dependency items expose the shared related-task summary plus their immediate unresolved `blockedByTaskIds` within the explanation graph.
6. Hierarchy never creates or appears as a blocker, while archived and completed upstreams correctly satisfy dependency edges.
7. Available tasks, unavailable tasks, claims, missing tasks, and human/JSON modes use the documented stable shapes, messages, codes, and exits.
8. The explanation is read-only and does not claim, release, mutate, or promise continued availability.

## Non-functional acceptance criteria

1. Every result is internally consistent within one read-only SQLite snapshot and applies no migration or repository write.
2. Core exposes reusable typed availability-explanation and unresolved-dependency queries that can run on a caller-owned connection or transaction.
3. Recursive traversal and hydration use bounded database work, remain cycle-safe under graph invariants, and feel immediate for thousands of tasks.
4. The command is local and network-free and resolves the same canonical state from main and linked worktrees.
5. Core owns availability and traversal semantics; CLI owns parsing and deterministic human/JSON rendering.

## Verification

- Explain tasks independently and jointly affected by archive, completion, active claim, and unresolved dependencies.
- Cover zero, one, branched, deep, diamond, and shared dependency graphs; verify stop conditions, direct/recursive separation, deduplication, `blockedByTaskIds`, and ordering.
- Include hierarchy-only relationships and prove they never affect explanation output.
- Change claims, dependency edges, archive state, and status completion between invocations and prove explanations are derived rather than cached.
- Coordinate a concurrent writer with an open query and prove each response comes from one snapshot while remaining informational after completion.
- Snapshot exact available human output, unavailable heading/reasons, optional claim and dependency sections, dependency lines, JSON shapes, missing-task errors, and exits.
- Verify read-only pending-migration behavior, bounded query growth, linked-worktree consistency, and no writes.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Changing task, status, claim, archive, dependency, or hierarchy state.
- General upstream/downstream relationship maps, paths, hierarchy traversal, or combined graph presentation; E4-S5.
- Filtering or listing multiple tasks; E2-S2 and E4-S3 own list queries.
- Claim acquisition guarantees; E5 claim operations own atomic acquisition.
- Persisted blocker or availability state, background refresh, polling, or notifications.

## Implementation task

See [E4-S4-T1: Implement blocking explanations](../tasks/E4-S4-T1-implement-blocking-explanations.md).
