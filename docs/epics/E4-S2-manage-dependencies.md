---
id: E4-S2
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E2-S1
---

# E4-S2: Manage dependencies

## Outcome

Users and agents can record and remove mandatory execution-order relationships, while the repository rejects invalid graphs and derives blocking from current upstream completion.

## User story

As a user or agent, I want to add and remove mandatory dependencies so that execution order is explicit.

## Commands

```text
tbtm task dependency add <task-id> --depends-on <upstream-id> [--agent <uuid>] [--json]
tbtm task dependency remove <task-id> --depends-on <upstream-id> [--agent <uuid>] [--json]
```

`<task-id>` is always the downstream task. If task A is invoked with `--depends-on B`, B is upstream of A and blocks A until B is effectively completed.

## Product decisions

### Relationship model

- Dependencies are directed, many-to-many, and mandatory. A task may have zero or many upstream dependencies, and one upstream may block many downstream tasks.
- Any task types may be connected. Hierarchy neither creates nor restricts dependencies.
- A downstream task has all dependencies satisfied only when every direct upstream is effectively completed through either a completed status or archive.
- Dependency satisfaction and availability are derived from current repository state. No `blocked`, `satisfied`, or `available` cache is persisted.

### Mutation rules

- The logical `user` or a registered agent selected with `--agent` may add or remove a dependency.
- Only a non-archived downstream task may be mutated. A completed but non-archived downstream remains mutable.
- An upstream task may be active or archived. An archived upstream immediately satisfies its edge while preserving the relationship for history and graph views.
- Add rejects self-dependencies, duplicate directed edges, and any direct or indirect cycle.
- Remove rejects an absent directed edge; it is not an idempotent no-op.
- A successful mutation updates only the downstream task's `updatedAt` and `updatedBy`. Failed mutations change neither task metadata nor relationship rows.

### Validation and concurrency

- Validation precedence is: command syntax, actor, downstream task, downstream archive state, upstream task, self-link, edge existence or duplication, cycle, then mutation.
- Cycle detection follows dependency direction and considers the complete current graph. The error contract does not expose a chosen cycle path in the MVP.
- Actor resolution, task checks, edge validation, cycle detection, mutation, metadata update, and response construction occur under one SQLite immediate write transaction.
- The database enforces one directed edge per task pair and valid task references in addition to core validation.

### Output and read integration

- Human success output only confirms the directed edge added or removed. Human failure output reports the specific reason through the shared error renderer.
- JSON success uses the shared envelope and returns only `{taskId, dependsOn}` for the changed edge. It does not return a full task aggregate or an availability result.
- `task view` and the existing full-task aggregate are retrofitted to populate direct `dependencies.upstream` and `dependencies.downstream` summaries. Both collections use the E2-S2 related-task shape and sort by task ID.
- `task create` continues to return empty dependency collections for a newly created task. Compact `task list` remains unchanged because dependencies are not part of its projection.

### Errors and exits

| Code | Exit | Condition |
|---|---:|---|
| `SELF_DEPENDENCY` | 2 | Downstream and upstream IDs are equal |
| `DEPENDENCY_EXISTS` | 2 | The directed edge already exists during add |
| `DEPENDENCY_CYCLE` | 2 | Adding the edge would create a cycle |
| `TASK_ARCHIVED` | 2 | The downstream task is archived |
| `TASK_NOT_FOUND` | 3 | The downstream or upstream task does not exist |
| `AGENT_NOT_FOUND` | 3 | The selected agent does not exist |
| `DEPENDENCY_NOT_FOUND` | 3 | The directed edge does not exist during remove |

Repository, database, permission, and unexpected failures preserve the shared error and exit contracts. Task-not-found details identify the unresolved task ID; dependency errors identify both directed-edge endpoints.

## Functional acceptance criteria

1. A user or registered agent can add multiple mandatory upstream dependencies to a non-archived downstream task and remove them independently.
2. One upstream task may be shared by multiple downstream tasks, and dependency direction is preserved in every mutation and view.
3. Self-links, duplicate edges, direct cycles, and indirect cycles are rejected without changing relationships or task metadata.
4. Removing a missing edge fails with `DEPENDENCY_NOT_FOUND`; removing an existing edge updates only the downstream task's mutation metadata.
5. Archived downstream tasks reject dependency mutations, while completed active downstream tasks remain mutable and archived upstream tasks remain valid targets.
6. Dependency satisfaction remains false until every upstream is effectively completed and immediately reflects upstream status, archive, or unarchive state without cached recomputation.
7. Add and remove return the documented concise human or JSON result, and failures use stable codes, details, and exits.
8. Full task detail exposes deterministic direct upstream and downstream summaries without changing compact task-list output.

## Non-functional acceptance criteria

1. Graph validation and mutation are atomic under concurrent local processes and linked worktrees; no accepted write can introduce a cycle or duplicate edge.
2. Dependency operations are local and network-free and feel immediate for a personal repository-sized graph.
3. Core owns typed graph rules, actor attribution, transactions, and satisfaction queries; CLI code owns parsing and rendering.
4. SQLite constraints protect referential and directed-edge uniqueness invariants, while parameterized queries protect data handling.
5. Relationship output and validation precedence are deterministic, timestamps use RFC 3339 UTC, and JSON fields use camelCase.

## Verification

- Add zero, one, and multiple upstreams; share one upstream across multiple downstreams; and remove one edge from a many-edge task.
- Verify satisfaction with incomplete, completed-status, and archived upstream combinations, including the all-upstreams requirement.
- Exercise self, duplicate, missing-edge, direct-cycle, and indirect-cycle failures plus the documented validation precedence.
- Verify archived-downstream rejection, active-completed downstream mutation, archived-upstream acceptance, user/agent attribution, and failure metadata invariants.
- Run concurrent graph mutations that could jointly form a cycle and prove only a valid acyclic result commits.
- Verify create/view/list compatibility, deterministic direct-summary ordering, one-snapshot detail reads, human/JSON output, and linked-worktree behavior.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Hierarchy mutation or hierarchy-derived dependencies; E4-S1.
- Available-task listing and complete availability output; E4-S3.
- Blocking explanations and recursive relationship maps; E4-S4 and E4-S5.
- Claim acquisition or release; E5.
- Optional, weighted, typed, or cross-repository dependencies.
- Editing relationships whose downstream task is archived.

## Implementation task

See [E4-S2-T1: Implement dependency management](../tasks/E4-S2-T1-implement-dependency-management.md).
