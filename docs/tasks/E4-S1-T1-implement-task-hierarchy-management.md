---
id: E4-S1-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E2-S1-T1
  - E2-S2-T1
---

# E4-S1-T1: Implement task hierarchy management

## Parent story

[E4-S1: Manage task hierarchy](../epics/E4-S1-manage-task-hierarchy.md)

## Objective

Implement the authoritative single-parent hierarchy model, transactional child-oriented parent mutations, cycle and type validation, direct full-task hydration, and deterministic recursive hierarchy reads.

## Readiness

Planning is complete. E2-S1-T1 and E2-S2-T1 are implemented, so this task is ready. It provides the authoritative hierarchy model required to unblock E2-S3-T1 and later support E4-S5.

## Deliverables

- Sequential SQLite migration for one optional parent per task and traversal indexes.
- Typed core inputs, outputs, errors, type-matrix rules, cycle prevention, mutations, and hierarchy read APIs.
- Full-task hydration for direct parent and children.
- CLI parent set/remove and hierarchy read commands with stable human/JSON output and exit mapping.
- Migration, core, integration, concurrency, snapshot, output, and regression tests.

## Proposed structure

Extend the existing task module and shared full-task models while preserving the core/CLI boundary. A focused hierarchy submodule is appropriate if it keeps traversal SQL isolated:

```text
crates/tbtm-core/
├── migrations/
│   └── 0007_task_hierarchy.sql
└── src/
    ├── lib.rs
    └── task.rs                 # or task/hierarchy.rs

crates/tbtm-cli/
├── src/main.rs
└── tests/task_hierarchy.rs
```

Exact file boundaries may follow the repository at implementation time. Core behavior must remain testable without invoking the CLI process.

## Technical choices

- Add a `task_hierarchy` table with `child_task_id` as the primary key and `parent_task_id` as a required foreign key to `tasks(id)`. Add a check rejecting equal endpoints and a parent-oriented index for direct-child and descendant traversal.
- Add migration version 7 after the currently implemented archive-reason migration 6; leave repository compatibility `schemaVersion: 1` unchanged.
- Centralize the fixed hierarchy type matrix in core and expose validation that accepts a child type and optional parent type. E2-S3 uses this model for current-parent and direct-child validation during type changes.
- Use a recursive CTE or equivalent bounded traversal to reject a set when the proposed parent is already a descendant of the child. Traverse all rows regardless of archive state.
- Use the existing immediate-transaction path so competing hierarchy writers and future task-type writers serialize validation and mutation.
- Reuse existing task IDs, actors, timestamps, `RelatedTask`, shared envelopes, repository resolution, migration handling, and error rendering.

## Implementation flow

### 1. Add the hierarchy migration

Create the edge table with:

- exactly one row at most per child;
- both endpoints required and foreign-keyed to tasks;
- unequal endpoint enforcement;
- an index beginning with `parent_task_id` for reverse lookup and recursive traversal.

Advance the internal latest migration to 7 and register the migration in sequence. Write commands apply compatible pending migrations through the established path. Read-only `task view` and `task hierarchy` do not migrate: a compatible repository with migration 7 pending returns the established incomplete-database error without querying a missing table or writing SQLite artifacts.

### 2. Define typed contracts

Define parent-set and parent-remove inputs containing child task ID and optional agent UUID; set additionally contains parent ID. Both mutations return:

```json
{"taskId":"project-task-a1b2c3d4","parentId":"project-story-b2c3d4e5"}
```

Remove returns `parentId: null`. Define a hierarchy read result with selected-task summary, optional direct parent, direct children, and optional recursive descendants. Each descendant combines the E2-S2 related-task summary with a positive integer `depth`.

Add typed errors for self-parent, invalid hierarchy, hierarchy cycle, archived parent, and missing parent edge. Reuse task-archived, task-not-found, and agent-not-found errors with endpoint details at the rendering boundary.

### 3. Implement parent set atomically

1. Validate command syntax and resolve the repository read-write with compatible pending migrations.
2. Begin an immediate transaction.
3. Resolve actor, child, child archive state, proposed parent, parent archive state, self-link, type matrix, and cycle in the documented precedence.
4. Read the current parent. If it is the proposed parent, return the validated no-op without touching rows or metadata.
5. Insert or replace exactly one child edge.
6. Update only the child task's `updated_at`, `updated_actor_type`, and `updated_agent_id` once.
7. Construct the result and commit.

Parent replacement must be one transaction: no observer may see a temporary parentless state, and any validation or metadata failure restores the previous edge and metadata.

### 4. Implement parent remove atomically

1. Resolve syntax, repository, transaction, actor, child, and child archive state.
2. Resolve the current edge; return `PARENT_NOT_FOUND` when absent.
3. Load the current parent and reject `PARENT_ARCHIVED` before deletion.
4. Delete exactly the current edge and update only child mutation metadata once.
5. Construct `{taskId, parentId: null}` and commit.

Every failure rolls back edge and metadata changes. Because archive preserves relationships, an archived endpoint must first be unarchived through E2-S6.

### 5. Expose shared validation and queries

Provide caller-owned connection/transaction helpers for:

- validating a child type against an optional current parent and direct children;
- loading one direct parent and all direct children;
- determining whether a proposed edge creates a hierarchy cycle;
- selecting recursive descendants with depth.

These helpers must not open nested transactions. E2-S3 can call the direct-edge validator inside its own write transaction, and E4-S5 can reuse traversal without copying graph rules.

### 6. Retrofit full-task reads

Hydrate `hierarchy.parent` and `hierarchy.children` in the same snapshot as the rest of `task view` and mutation-returned full aggregates. Use the existing related-task shape and sort direct children by ID. Avoid per-child query growth.

Newly created independent tasks retain `parent: null` and `children: []`. Compact list output remains unchanged. Ensure other commands returning full task aggregates receive the populated direct hierarchy without changing their established fields.

### 7. Implement hierarchy reads

Expose `task hierarchy <task-id> [--recursive] [--json]` as a read-only command. Default output includes the target summary, optional parent, and direct children. With `--recursive`, include every descendant once, ordered by depth ascending and task ID ascending. Archived nodes and their descendants remain visible.

Run the target, parent, children, and descendant queries in one SQLite read snapshot. Do not apply migrations or create persistent journal/WAL/shared-memory files. Human output must distinguish parent, direct children, and descendant depth; empty sections remain explicit and concise.

### 8. Map errors and output

Implement the epic's exact validation precedence and map:

| Exit | Errors |
|---:|---|
| 0 | Successful mutation, no-op, or hierarchy read |
| 1 | Database or unexpected operational failure |
| 2 | `SELF_PARENT`, `INVALID_TASK_HIERARCHY`, `HIERARCHY_CYCLE`, `TASK_ARCHIVED`, `PARENT_ARCHIVED`, or argument validation |
| 3 | `TASK_NOT_FOUND`, `AGENT_NOT_FOUND`, or `PARENT_NOT_FOUND` |
| 5 | Permission denied |

Exit 4 remains reserved for claim conflicts. Human mutations confirm the child-oriented action in one line. JSON uses the shared envelope and exact camelCase mutation/read shapes. Failures include relevant IDs and never expose SQL or a chosen cycle path.

## Test plan

### Migration and core tests

- Apply migration 7 once and through pending-migration handling; verify endpoint foreign keys, one-parent uniqueness, self-edge rejection, and the parent index.
- Set, replace, remove, and preserve independent tasks across every allowed type combination.
- Reject every invalid type edge plus self, direct-cycle, and multi-level indirect-cycle attempts.
- Verify actor/child/archive/parent/archive/self/type/cycle precedence for set and actor/child/archive/edge/current-parent-archive precedence for remove.
- Verify same-parent no-op retains timestamps, actors, and rows; effective mutations change only child metadata.
- Verify archived endpoints prevent mutation, archive itself preserves edges, and failures/injected database errors roll back completely.
- Race writes from separate connections/worktrees that could form a cycle or replace one child concurrently; assert one valid acyclic, single-parent result.
- Exercise shared validation under a caller-owned transaction with current parent and multiple direct children in preparation for E2-S3.

### Query and CLI integration tests

- Verify direct parent/children hydration across `task view` and every existing full-task-returning command, with exact E2-S2 summary shape and ID ordering.
- Query empty, one-level, branched, deep, and archived hierarchies in default and recursive modes; assert deduplication and depth/ID ordering.
- Snapshot concise human mutation output, human hierarchy sections, and exact JSON envelopes including omission of `descendants` without `--recursive`.
- Verify every stable error, detail, exit, validation precedence, and no-write guarantee as logical user and registered agent.
- Confirm compact task lists and new-task output remain compatible.
- Confirm both hierarchy reads remain read-only with migration 7 pending and create no persistent SQLite artifacts; apply migration through a supported write and retry successfully.
- Run canonical and linked-worktree commands and verify one-snapshot read consistency during concurrent hierarchy mutation.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test a temporary repository containing a branched hierarchy, archived endpoints with preserved edges, a replacement, and concurrent commands from linked worktrees.

## Definition of done

- Every functional and non-functional acceptance criterion in E4-S1 passes.
- Migration 7 establishes the authoritative single-parent hierarchy without changing repository compatibility version 1.
- Parent set/remove follows the agreed child-oriented, active-endpoint, actor, metadata, no-op, error, exit, and output contracts atomically.
- Type-invalid edges, multiple parents, and cycles remain impossible under supported concurrent writers.
- Direct full-task hydration and recursive hierarchy reads are deterministic, snapshot-consistent, and include archived context.
- Reusable validation and query primitives are sufficient for E2-S3 without duplicating hierarchy rules.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E4-S1 epic](../epics/E4-S1-manage-task-hierarchy.md)
- [PRD task types](../PRD.md#72-task-types)
- [PRD hierarchy model](../PRD.md#77-hierarchy)
- [PRD hierarchy management requirement](../PRD.md#fr-4-hierarchy-management)
- [PRD relationship-map requirement](../PRD.md#fr-10-relationship-map)
- [PRD validation rules](../PRD.md#11-validation-and-edge-case-rules)
- [E2-S1 task creation](../epics/E2-S1-create-a-task.md)
- [E2-S2 task detail and listing](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S3 task-content update](../epics/E2-S3-update-task-content.md)
