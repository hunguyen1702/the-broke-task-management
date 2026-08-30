---
id: E4-S1
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E2-S1
---

# E4-S1: Manage task hierarchy

## Outcome

Users and agents can organize active tasks into a valid single-parent scope hierarchy and inspect parent, child, and recursive descendant context without conflating containment with execution dependencies.

## User story

As a user or agent, I want to assign valid parents so that epics and stories organize work.

## Commands

```text
tbtm task parent set <task-id> --parent <parent-id> [--agent <uuid>] [--json]
tbtm task parent remove <task-id> [--agent <uuid>] [--json]
tbtm task hierarchy <task-id> [--recursive] [--json]
```

Parent mutation is deliberately child-oriented: `<task-id>` is always the child. There is no duplicate `add-child` or `remove-child` command surface.

## Product decisions

### Relationship model

- Hierarchy represents scope or containment and never implies a dependency, blocking, completion, availability, or claim relationship.
- A task has zero or one direct parent. A parent may have any number of direct children.
- The fixed type matrix is authoritative: epics have no parent; stories may have only an epic parent; tasks, improvements, refactors, bugs, spikes, testing tasks, and POCs may have an epic or story parent.
- Parent assignment is optional. A task may remain independent regardless of type.
- The graph is acyclic. Cycle validation traverses the complete hierarchy, including archived tasks and preserved edges.

### Mutation behavior

- The logical `user` or a registered agent selected through `--agent` may set, replace, or remove a parent.
- Both endpoints of an edge must be active when it is created, replaced, or removed. `set` requires the child and proposed parent to be active; `remove` requires the child and its current parent to be active.
- Archiving a task preserves every hierarchy edge. If either endpoint is archived, it must be unarchived before that edge can be changed.
- `set` rejects self-parenting, an invalid child/parent type pair, and any direct or indirect cycle.
- Setting the already-current parent is a fully validated no-op: it succeeds without changing relationship rows, `updatedAt`, or `updatedBy`.
- Removing a task with no parent is not an idempotent no-op; it returns `PARENT_NOT_FOUND`.
- A successful effective mutation updates only the child task's `updatedAt` and `updatedBy`. It does not mutate parent metadata, status, archive state, dependencies, claims, or availability.

### Validation and concurrency

- `set` validation precedence is: command syntax, actor, child existence, child archive state, parent existence, parent archive state, self-parent, type matrix, cycle, then mutation.
- `remove` validation precedence is: command syntax, actor, child existence, child archive state, current edge existence, current parent archive state, then mutation.
- Actor resolution, endpoint reads, edge checks, cycle validation, mutation, child metadata, and result construction occur in one SQLite immediate write transaction.
- Database constraints enforce singular parentage, valid endpoint references, and unequal endpoints in addition to core validation.
- E4-S1 exposes reusable transaction/connection-borrowing queries for current parent, direct children, and recursive descendants. E2-S3 reuses the same type-matrix validation against a task's current direct edges.

### Read and output contract

- `task view` and the shared full-task aggregate populate `hierarchy.parent` and `hierarchy.children` with direct E2-S2 related-task summaries. Children sort by task ID.
- `task hierarchy <task-id>` returns the selected task, its direct parent, and direct children. `--recursive` additionally returns all descendants with a positive `depth` relative to the selected task.
- Recursive descendants are deduplicated and ordered by depth ascending, then task ID ascending. Archived tasks remain present in hierarchy reads.
- Hierarchy reads are read-only, apply no pending migrations, and use one SQLite snapshot. E4-S5 remains responsible for combined hierarchy/dependency map directions and graph-oriented state.
- Mutation JSON uses the shared success envelope and returns `{taskId, parentId}`; removal returns `parentId: null`. Human mutation output is one concise confirmation.
- Hierarchy JSON uses stable camelCase fields for `task`, `parent`, `children`, and `descendants`. Without `--recursive`, `descendants` is omitted rather than ambiguously returning an empty complete traversal.
- Compact `task list` remains unchanged. Newly created independent tasks continue to return `parent: null` and `children: []`.

### Errors and exits

| Code | Exit | Condition |
|---|---:|---|
| `SELF_PARENT` | 2 | Child and proposed parent IDs are equal |
| `INVALID_TASK_HIERARCHY` | 2 | The proposed edge violates the task-type matrix |
| `HIERARCHY_CYCLE` | 2 | The proposed edge would create a direct or indirect cycle |
| `TASK_ARCHIVED` | 2 | The child task is archived |
| `PARENT_ARCHIVED` | 2 | The proposed or current parent is archived |
| `TASK_NOT_FOUND` | 3 | The child, parent, or hierarchy-read target does not exist |
| `AGENT_NOT_FOUND` | 3 | The selected agent does not exist |
| `PARENT_NOT_FOUND` | 3 | Parent removal is requested for a task with no parent |

Task-not-found details identify the unresolved ID. Parent errors identify the child and relevant parent when known. Repository, database, permission, and unexpected failures preserve the shared contracts; exit code 4 remains reserved for claim conflicts. Every failure leaves hierarchy rows and task metadata unchanged.

## Functional acceptance criteria

1. A user or registered agent can assign, replace, and remove the single parent of an active task through child-oriented commands.
2. The complete fixed type matrix is enforced, while every type remains valid without a parent.
3. Self-parenting, direct cycles, indirect cycles, and concurrent mutations that would collectively form a cycle are rejected without partial writes.
4. A hierarchy edge can change only when both endpoints are active; archive preserves existing edges and hierarchy reads include archived tasks.
5. Setting the current parent is a valid metadata-preserving no-op, while removing a missing parent returns `PARENT_NOT_FOUND`.
6. Effective mutations update only the child's actor and timestamp metadata and return the documented concise human or JSON result.
7. Full task detail exposes deterministic direct parent and child summaries, and the hierarchy query exposes direct context plus optional deterministic recursive descendants.
8. All validation failures follow the documented precedence, stable codes, details, exits, and no-write guarantee.

## Non-functional acceptance criteria

1. Endpoint validation, cycle prevention, edge mutation, child metadata, and response construction are atomic under concurrent local processes and linked worktrees.
2. Hierarchy reads use one read-only SQLite snapshot, do not apply migrations, and do not create persistent database side artifacts.
3. Core owns typed hierarchy rules, traversal, transactions, and reusable queries; CLI owns parsing and rendering.
4. SQLite constraints preserve referential integrity and singular parentage, while parameterized operations protect data handling.
5. Operations are local, network-free, deterministic, and responsive for a personal repository-sized hierarchy; JSON remains camelCase and timestamps RFC 3339 UTC.

## Verification

- Exercise every valid and invalid child/parent type combination, parent replacement, removal, and independent tasks.
- Verify self, direct-cycle, indirect-cycle, missing endpoint, archived endpoint, unknown actor, and missing-edge failures with exact precedence, codes, exits, and unchanged metadata.
- Verify the same-parent no-op and effective user/agent mutations, including precise child-only metadata changes.
- Race hierarchy writes from separate connections and linked worktrees that would otherwise produce a cycle or invalidate a concurrent E2-S3 type update.
- Verify direct full-task hydration and hierarchy reads with branched, deep, archived, and empty hierarchies, including deterministic depth/ID ordering and one-snapshot consistency.
- Verify pending-migration behavior, human/JSON output, compact-list compatibility, and formatting/lint/workspace tests.

## Out of scope

- Task type mutation, which E2-S3 implements using E4-S1 validation primitives.
- Dependency relationships, blocking, availability, or claims; E4-S2 through E5.
- Combined or multi-direction relationship maps and graph visualization; E4-S5.
- Mutating relationships while either endpoint is archived.
- Multiple parents, hierarchy-derived dependencies, cross-repository hierarchy, bulk moves, or permanent task deletion.

## Implementation task

See [E4-S1-T1: Implement task hierarchy management](../tasks/E4-S1-T1-implement-task-hierarchy-management.md).
