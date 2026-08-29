---
id: E4-S2-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E2-S1-T1
  - E2-S2-T1
---

# E4-S2-T1: Implement dependency management

## Parent story

[E4-S2: Manage dependencies](../epics/E4-S2-manage-dependencies.md)

## Objective

Implement transactional directed dependency mutations, cycle prevention, reusable dependency-satisfaction queries, and direct relationship hydration in the existing full-task view.

## Readiness

Planning is complete. E2-S1-T1 and E2-S2-T1 are implemented, so this task is ready and will provide the authoritative dependency model required by E4-S3 and E5-S1.

## Deliverables

- Sequential SQLite migration for directed task-dependency edges and supporting indexes.
- Core add/remove inputs, results, typed errors, graph validation, mutation logic, and effective-completion predicate.
- Core query support that hydrates direct upstream/downstream summaries in full task detail.
- CLI dependency add/remove commands with actor selection, concise human output, stable JSON envelopes, and exit mapping.
- Core, migration, integration, concurrency, output, and regression tests.

## Proposed structure

Extend the implemented task module while preserving the core/CLI boundary. A focused dependency submodule is preferred if it keeps graph SQL and traversal separate from task content:

```text
crates/tbtm-core/
├── migrations/
│   └── 0004_task_dependencies.sql
└── src/
    ├── lib.rs
    └── task.rs                 # or task/dependency.rs

crates/tbtm-cli/src/
└── main.rs                     # commands may be split as the CLI grows
```

Exact file boundaries may follow the current implementation. Core APIs must remain testable without invoking the process boundary.

## Technical choices

- Add a `task_dependencies` table with `downstream_task_id` and `upstream_task_id`, foreign keys to `tasks(id)`, a composite primary key, and a self-edge check. Add an upstream-oriented index for reverse traversal and downstream summary loading where the primary-key order is insufficient.
- Add migration version 4 and advance the internal latest migration while leaving repository `schemaVersion: 1` unchanged.
- Use recursive CTE traversal or an equivalent bounded graph query to reject an add when the proposed upstream already reaches the downstream through existing upstream edges.
- Use `BEGIN IMMEDIATE` through the existing Rust transaction path so competing graph writers serialize validation and insertion.
- Do not persist derived availability or dependency-satisfaction state.
- Reuse existing task IDs, status completion, archive flags, actors, timestamps, `RelatedTask`, and shared error envelopes rather than creating parallel models.

## Implementation flow

### 1. Add the relationship migration

Create the directed-edge table with:

- both endpoint IDs required and foreign-keyed to tasks;
- uniqueness of `(downstream_task_id, upstream_task_id)`;
- rejection of equal endpoints at the database boundary;
- indexes supporting traversal and bulk direct-summary queries.

Apply it through the existing compatible pending-migration path for write commands. Read-only commands retain existing migration behavior and must not mutate an outdated repository. In particular, `task view` against a schema-version-compatible repository that still has migration 4 pending returns the established stable incomplete-database error before querying dependency tables; it must not expose a raw missing-table error or pretend that dependencies are empty. After any supported mutation applies migration 4, the same view succeeds normally.

### 2. Define typed core contracts

Represent an add or remove request with downstream task ID, upstream task ID, and optional agent UUID. Return a concise edge result serialized as:

```json
{
  "taskId": "project-task-a1b2c3d4",
  "dependsOn": "project-task-b2c3d4e5"
}
```

Add typed errors for self-dependency, duplicate edge, missing edge, and cycle. Reuse `TaskArchived`, `TaskNotFound`, and `AgentNotFound`, enriching endpoint details at the rendering boundary where needed.

### 3. Implement add and remove atomically

For both operations:

1. Validate parsed command inputs.
2. Resolve the repository read-write, configure the existing busy timeout, and apply compatible pending migrations.
3. Begin an immediate transaction.
4. Resolve actor, downstream, downstream archive state, and upstream in the documented precedence.
5. Validate self-link and operation-specific edge existence.
6. For add, traverse the current graph to reject any cycle before insert.
7. Insert or delete exactly one edge.
8. Update only the downstream task's `updated_at`, `updated_actor_type`, and `updated_agent_id` once.
9. Construct the edge result and commit.

Every validation or database failure rolls back both relationship and metadata changes. A completed but active downstream follows the same path; an archived downstream is rejected. Upstream archive state never prevents the relationship mutation.

### 4. Expose shared graph predicates

Provide transaction/connection-borrowing core queries for:

- whether every direct upstream of a task is effectively completed;
- direct upstream and downstream related-task summaries.

The satisfaction predicate must treat zero upstreams as satisfied and each upstream as satisfied when `archived = true OR status.completed = true`. It must not consider the downstream task's own status, archive, or claim; E4-S3/E5 combine it with the rest of the availability predicate.

### 5. Retrofit full-task reads

Replace placeholder dependency arrays in `task view` with direct related-task summaries loaded in the same read snapshot as the task. Sort both collections by related task ID and retain the established summary shape:

```json
{
  "id": "project-task-b2c3d4e5",
  "title": "Prepare schema",
  "type": "task",
  "status": {"code": "done", "name": "Done", "completed": true}
}
```

Newly created tasks still return empty arrays. Compact lists remain unchanged. Avoid per-related-task query growth by loading each direction with bounded queries.

### 6. Add CLI parsing and rendering

Expose the two documented commands with repeat invocation for multiple edges. Human success is exactly one concise confirmation describing the directed edge; failures use the shared renderer and include the relevant endpoint IDs.

JSON success returns the edge result under the shared success envelope. Do not include full task detail, dependency satisfaction, availability, or a cycle path.

Map errors as follows:

| Exit | Errors |
|---:|---|
| 0 | Successful add or remove |
| 1 | Database or other operational failure |
| 2 | `SELF_DEPENDENCY`, `DEPENDENCY_EXISTS`, `DEPENDENCY_CYCLE`, `TASK_ARCHIVED`, or argument validation |
| 3 | `TASK_NOT_FOUND`, `AGENT_NOT_FOUND`, or `DEPENDENCY_NOT_FOUND` |
| 5 | Permission denied |

Exit 4 remains reserved for claim conflicts.

## Test plan

### Migration and core tests

- Apply migration 4 once and through the normal pending-migration path; reject duplicate and self edges at the database boundary.
- Add one/many upstreams, share an upstream across downstreams, and remove one of several edges.
- Verify zero dependencies are satisfied; mixed upstream completion is not; all completed-status, archived, and mixed completed/archived upstream sets are satisfied.
- Reject duplicate, absent remove, self, direct-cycle, and multi-level indirect-cycle operations with exact typed errors.
- Prove validation precedence for unknown actor, missing downstream, archived downstream, missing upstream, self, edge state, and cycle.
- Verify active-completed downstream mutations, archived-upstream relationships, downstream-only actor/timestamp changes, and unchanged metadata on every failure.
- Race graph additions from separate connections/worktrees that would collectively form a cycle; assert committed state remains acyclic and unique.
- Inject mutation and metadata failures to prove transaction rollback.

### Query and CLI integration tests

- View upstream and downstream tasks and assert exact E2-S2 summary shape, direction, task-ID ordering, and one-snapshot consistency.
- Verify a newly created task still returns empty dependency arrays and compact list JSON remains unchanged.
- Snapshot add/remove human confirmations and JSON edge envelopes.
- Verify every error code, endpoint detail, exit, and no-write guarantee in both human and JSON modes.
- Run commands as logical user and registered agent from canonical and linked worktrees.
- Confirm read-only task view does not apply migrations or create persistent SQLite side artifacts.
- Open a migration-3 repository with the new binary and verify `task view` returns the stable incomplete-database error without writes; apply migration 4 through a supported mutation and verify the same view then succeeds.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test a temporary repository containing a branched acyclic graph, completed and archived upstreams, and commands issued from two linked worktrees.

## Definition of done

- Every functional and non-functional acceptance criterion in E4-S2 passes.
- Migration 4 establishes the authoritative directed dependency model without changing repository compatibility version 1.
- Add/remove operations follow the stable validation, actor, metadata, error, exit, and output contracts atomically.
- Cycles and duplicate edges remain impossible under concurrent supported writers.
- The shared satisfaction predicate exactly implements effective completion across every mandatory upstream.
- Full-task detail returns deterministic direct dependencies while create and compact-list contracts remain compatible.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E4-S2 epic](../epics/E4-S2-manage-dependencies.md)
- [PRD dependency model](../PRD.md#78-dependency)
- [PRD effective completion](../PRD.md#76-effective-completion)
- [PRD availability definition](../PRD.md#81-available-task-definition)
- [PRD dependency management requirement](../PRD.md#fr-5-dependency-management)
- [E2-S1 task creation](../epics/E2-S1-create-a-task.md)
- [E2-S2 task detail and listing](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S5 safe archive](../epics/E2-S5-archive-a-task.md)
