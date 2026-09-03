---
id: E6-S1-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S3-T1
  - E2-S1-T1
---

# E6-S1-T1: Implement task comments

## Parent story

[E6-S1: Add and view comments](../epics/E6-S1-add-and-view-comments.md)

## Objective

Implement atomic comment creation and deterministic read-only comment listing for active and archived tasks, with stable actor attribution and human/JSON CLI contracts.

## Readiness

Planning is complete. E1-S3-T1 provides registered agent identity, E2-S1-T1 provides the task model, and both dependencies are implemented. This task is ready for implementation.

## Deliverables

- A compatible SQLite migration for durable task comments.
- Core comment models, validation, actor resolution, atomic insertion, and snapshot listing.
- `tbtm task comment add` and `tbtm task comment list` CLI handlers.
- Stable human output, JSON envelopes, typed errors, and exit mappings.
- Migration, core, CLI, rollback, concurrency, linked-worktree, and output tests.

## Proposed structure

Extend the current core/CLI layout without moving domain behavior into the process boundary:

```text
crates/tbtm-core/
├── migrations/
│   └── 0008_task_comments.sql
└── src/
    └── task.rs                 # or a focused task/comment module

crates/tbtm-cli/src/
└── main.rs                     # or the existing task command modules
```

Exact module boundaries may follow the code present at implementation time. Core owns comment validation, UUID/timestamp generation, actor resolution, database constraints, transactions, ordering, and typed errors. CLI owns Clap syntax, rendering, the JSON envelope, and process exit codes.

## Technical choices

- Resolve the canonical database through the E1-S5-aware repository resolver already used by task operations.
- Add the next ordered schema-version-1-compatible migration and keep config `schemaVersion: 1` unchanged.
- Use UUID v4 text as the complete public comment ID, with a database primary-key constraint and bounded collision regeneration.
- Store one tagged author representation equivalent to task actor metadata: `user` with no agent ID, or `agent` with a foreign key to the authoritative agent UUID.
- Store the caller's content unchanged after validating `!content.trim().is_empty()`.
- Store creation time as RFC 3339 UTC.
- Do not update the parent task row when adding a comment.
- Use `created_at ASC, id ASC` for deterministic list ordering.
- Keep comments out of `FullTask`; expose focused add/list results.
- Keep all behavior synchronous, repository-local, and network-free.

## Data model

Add a `task_comments` table with equivalent logical fields:

```text
id                 UUID text, primary key
task_id            text, required FK to tasks(id)
content            text, required and non-empty under trimmed domain validation
author_actor_type  `user` or `agent`
author_agent_id    nullable FK to agents(id)
created_at         RFC 3339 UTC timestamp, required
```

Add a database check ensuring `user` has no agent UUID and `agent` has one. Index `(task_id, created_at, id)` for chronological reads. Do not add update metadata, soft-delete state, archive-specific fields, or a placeholder relation in the full-task aggregate.

The migration must apply transactionally to fresh and existing compatible repositories, preserve all task/agent data, participate in the existing migration ledger, and avoid changing repository/config compatibility version.

## Implementation flow

### 1. Define focused contracts

Create a serializable comment result with camelCase fields:

```text
id, taskId, content, author, createdAt
```

Serialize `author` as `"user"` or the agent UUID. Retain sufficient internal/hydrated agent information for human rendering without changing the JSON shape.

Define add input as task ID, content, and optional agent UUID. Define list input as task ID only. Reuse `TASK_NOT_FOUND`, `AGENT_NOT_FOUND`, malformed UUID, and shared repository errors; add `INVALID_COMMENT_CONTENT` for empty trimmed content.

### 2. Parse and validate CLI input

Expose:

```text
tbtm task comment add <task-id> --content <markdown> [--agent <uuid>] [--json]
tbtm task comment list <task-id> [--json]
```

- Require `--content` for add and reject empty or whitespace-only values.
- Preserve the accepted string exactly rather than storing the trimmed value.
- Parse optional `--agent` as UUID before mutation; omission selects `user`.
- Keep both commands non-interactive when explicit arguments are supplied.

### 3. Apply the migration on mutation paths

Use the established read-write resolver and compatible pending-migration mechanism before adding a comment. A migration failure prevents insertion; a successfully committed migration may remain applied if the later add operation fails, matching existing mutation behavior.

The read-only list command must use the established no-migration read path. It may report inherited repository or migration compatibility failures but must not modify the database to resolve them.

### 4. Insert one comment atomically

After migrations, execute one add attempt in a SQLite transaction:

1. Resolve the target task without filtering archived rows; return `TASK_NOT_FOUND` if absent.
2. Resolve the supplied agent UUID when present; return `AGENT_NOT_FOUND` if absent.
3. Generate a UUID v4 and one RFC 3339 UTC timestamp.
4. Insert the complete comment row with valid tagged author fields.
5. Hydrate the created result and agent display data in the same transaction.
6. Commit, then render the result.

On a comment-ID uniqueness collision, roll back that attempt and retry with a new UUID up to a small implementation constant. Do not retry foreign-key, validation, or unrelated database failures. Any failure leaves no comment and never updates the parent task.

### 5. List comments from one snapshot

1. Open the canonical database read-only without applying migrations.
2. Start the repository's established deferred read transaction/snapshot.
3. Resolve the task regardless of archive state; unknown task returns `TASK_NOT_FOUND` rather than an empty list.
4. Query all matching comments and their optional agent display names ordered by `created_at ASC, id ASC`.
5. Commit/close the read scope and render the complete list.

An existing task with no comments returns an empty vector. Listing never writes migration state, comment data, task metadata, or persistent journal artifacts beyond the repository's established read-only guarantees.

### 6. Render stable output

Add JSON returns one focused comment under the shared success envelope; list returns an array of the identical shape. For example:

```json
{
  "ok": true,
  "data": {
    "id": "UUID",
    "taskId": "project-task-a1b2c3d4",
    "content": "Implemented parser decision.",
    "author": "UUID",
    "createdAt": "RFC 3339 UTC timestamp"
  },
  "error": null
}
```

Human add output and each list entry show comment ID, creation time, content, and `user` or `<display-name> (<uuid>)`. Empty human list output is exactly one concise success message chosen and snapshot-tested during implementation. JSON mode writes only the envelope to stdout.

### 7. Map errors and exits

| Exit | Conditions |
|---:|---|
| 0 | Comment added, non-empty list returned, or empty list returned |
| 1 | Database, busy-timeout, migration, or unexpected operational failure |
| 2 | Missing/malformed arguments, malformed UUID, or `INVALID_COMMENT_CONTENT` |
| 3 | `TASK_NOT_FOUND` or `AGENT_NOT_FOUND` |
| 5 | Permission denied |

Errors use the shared envelope. `INVALID_COMMENT_CONTENT` needs no echo of the full Markdown input. `TASK_NOT_FOUND` details include `{taskId}`; `AGENT_NOT_FOUND` follows the existing UUID detail contract. Every error path preserves comment and task state.

## Test plan

### Unit and model tests

- Empty and whitespace-only content rejection while non-empty content is stored byte-for-byte.
- Comment UUID generation, serialization, RFC 3339 UTC timestamps, and actor scalar rendering.
- Human author rendering for `user` and agent display name plus UUID.
- Deterministic comparison/order by creation time then ID.
- Typed errors, detail shapes, and exit mapping.

### Migration and persistence tests

- Fresh initialization applies the comment migration; an existing schema-version-1 repository upgrades on add without config version changes or lost task/agent data.
- Migration transaction and ledger ordering are correct; constraints reject invalid tagged actors and dangling task/agent references.
- Successful user/agent insert persists exactly one complete row.
- Unknown task, unknown agent, insert failure, hydration failure, and retry exhaustion leave no partial comment.
- Comment add leaves every parent task field, timestamp, actor, claim, hierarchy, and dependency unchanged.

### Integration and concurrency tests

- Add and list as user and registered agent on active and archived tasks.
- List zero, one, and several comments; force equal timestamps and verify ascending ID tie-break.
- Reject missing/empty content, malformed/unknown agent, and unknown task with exact output, exit, and no mutation.
- Force a UUID collision and verify bounded regeneration; run concurrent additions from the same and linked worktrees and verify distinct durable rows and a usable database.
- Verify list is read-only, uses one snapshot, does not apply pending migrations, and creates no unintended persistent state.
- Snapshot exact human and JSON add, list, empty, and error output.
- Regression-test that `task view` and other full-task responses have no new comments field.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test in a temporary repository by registering an agent, creating active and archived tasks, adding comments as both actors, and listing from the main and a linked worktree.

## Acceptance scenario impact

`add`: E6-S1 introduces new public CLI, persistence, actor, archived-task, output, and concurrency behavior. After implementation, revalidate the approved catalog and add independently executable E6-S1 scenarios; do not rewrite existing approved scenario history.

## Definition of done

- Every functional and non-functional criterion in E6-S1 passes.
- Migration and constraints preserve existing schema-version-1 repositories and comment referential integrity.
- User and registered-agent comments are inserted atomically on active and archived tasks without changing task mutation metadata.
- Listing is deterministic, snapshot-consistent, read-only, and distinguishes empty success from task-not-found.
- Human and JSON output, actor representation, errors, details, and exits are stable and tested.
- Collision, rollback, concurrent-process, and linked-worktree tests pass.
- No edit/delete operation or `FullTask` comment field is introduced.
- Scenario impact is revalidated and new approved behavior scenarios are added through the acceptance workflow.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E6-S1 epic](../epics/E6-S1-add-and-view-comments.md)
- [PRD comment model](../PRD.md#710-comment)
- [PRD comment requirements](../PRD.md#fr-9-comments)
- [PRD agent work lifecycle](../PRD.md#102-agent-work-lifecycle)
- [PRD E6-S1 story](../PRD.md#story-e6-s1-add-and-view-comments)
- [E1-S3 agent identity](../epics/E1-S3-register-an-agent.md)
- [E1-S3 implementation task](E1-S3-T1-implement-agent-registration.md)
- [E2-S1 task and actor model](../epics/E2-S1-create-a-task.md)
- [E2-S1 implementation task](E2-S1-T1-implement-task-creation.md)
- [E2-S2 full-task output contract](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S5 archive reason contract](../epics/E2-S5-archive-a-task.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [SQLite foreign keys](https://www.sqlite.org/foreignkeys.html)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
- [UUID crate](https://docs.rs/uuid/latest/uuid/)
