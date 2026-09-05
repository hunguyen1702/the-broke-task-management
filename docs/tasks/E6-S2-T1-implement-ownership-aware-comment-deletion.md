---
id: E6-S2-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E6-S1-T1
---

# E6-S2-T1: Implement ownership-aware comment deletion

## Parent story

[E6-S2: Delete comments under ownership rules](../epics/E6-S2-delete-comments-under-ownership-rules.md)

## Objective

Implement atomic hard deletion of one task-scoped comment, allowing logical-user authority to delete any comment and a declared registered agent to delete only its own comment, without changing parent-task state.

## Readiness

Planning is complete. E6-S1-T1 provides the authoritative comment schema, actor representation, comment output model, shared-store behavior, and add/list commands, so this task is ready.

## Deliverables

- Core delete input/result behavior, scoped lookup, ownership validation, exact deletion, transaction handling, and typed errors.
- `tbtm task comment delete` parsing, actor selection, human rendering, JSON envelope, and exit mapping.
- Core, CLI, rollback, concurrency, linked-worktree, output, and task-metadata regression tests.
- Public help and documentation that describe actor selection as a trust convention rather than caller authentication.

## Proposed structure

Extend the E6-S1 implementation without a new migration:

```text
crates/tbtm-core/src/comment.rs
crates/tbtm-core/src/lib.rs
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/task_comment.rs
```

Keep core behavior testable without invoking a process. Exact test factoring may follow the repository structure present during implementation.

## Technical choices

- Reuse `task_comments`, `TaskComment`, actor columns, canonical resolver, compatible write-path migrations, busy timeout, and E6-S1 author hydration.
- Define delete input with task ID, comment UUID, and optional agent UUID. Omission selects logical `user`; a supplied UUID must resolve to a registered agent.
- Add `CommentNotFound { task_id, comment_id }` and `CommentDeleteForbidden { task_id, comment_id, author }` typed errors with stable details and exits.
- Query comments by both `task_id` and `id`; a cross-task comment ID is indistinguishable from an absent comment.
- Capture the original comment before deletion and return it after commit using the existing E6-S1 shape.
- Use an immediate transaction and delete by exact task/comment identity with affected-row checking. Never authorize from a lookup outside the transaction.
- Do not update the task row or introduce tombstones, audit rows, or a migration.

## Implementation flow

### 1. Extend core and error contracts

Add a delete request and core operation returning `TaskComment`. Reuse the existing scalar author representation in JSON and any internal display information needed only for human rendering.

- `CommentNotFound` -> `COMMENT_NOT_FOUND`, details `{taskId, commentId}`, exit `3`.
- `CommentDeleteForbidden` -> `COMMENT_DELETE_FORBIDDEN`, details `{taskId, commentId, author}`, exit `5`.

Preserve established malformed UUID, `AGENT_NOT_FOUND`, `TASK_NOT_FOUND`, repository, migration, database, and permission behavior.

### 2. Parse the CLI command

Expose:

```text
tbtm task comment delete <task-id> <comment-id> [--agent <uuid>] [--json]
```

- Require both IDs and parse comment/agent UUID syntax before mutation.
- Omitted `--agent` selects logical-user authority; supplied `--agent` selects the exact registered identity.
- Keep the command non-interactive with no confirmation.
- Help text must not imply that the CLI authenticates whether the caller is physically a human or an agent.

### 3. Delete under one immediate transaction

1. Resolve the canonical repository read-write and apply compatible pending migrations.
2. Start an immediate SQLite transaction.
3. Resolve a supplied agent or return `AGENT_NOT_FOUND`; omission needs no agent lookup.
4. Resolve the task regardless of archive state or return `TASK_NOT_FOUND`.
5. Load and hydrate the comment by exact task ID and comment ID; otherwise return `COMMENT_NOT_FOUND`.
6. If acting as an agent, require the stored author to be that exact agent UUID; otherwise return `COMMENT_DELETE_FORBIDDEN`.
7. Delete by exact task ID and comment ID and require one affected row.
8. Commit, then render the captured deleted comment.

No task state is validated beyond existence. Any lookup, authorization, delete, affected-row, or commit failure leaves the comment present and the task unchanged.

### 4. Define concurrency results

Immediate transactions serialize competing writers. For two permitted callers targeting one comment, the first committed deletion succeeds and the later transaction returns `COMMENT_NOT_FOUND`. Busy-timeout exhaustion remains an operational error.

The scoped predicate and affected-row check prevent deletion of a comment associated with another task or any unrelated comment. Linked worktrees observe the same canonical state.

### 5. Render stable output

JSON success returns the existing E6-S1 comment object under the shared envelope:

```json
{
  "ok": true,
  "data": {
    "id": "UUID",
    "taskId": "project-task-a1b2c3d4",
    "content": "Removed Markdown content",
    "author": "user or agent UUID",
    "createdAt": "RFC 3339 UTC timestamp"
  },
  "error": null
}
```

Human success identifies the deleted comment ID and task ID. JSON mode writes only the envelope to stdout. No response adds comment data to the full-task aggregate.

## Test plan

### Core and persistence tests

- Logical user deletes user- and agent-authored comments from active and archived tasks.
- An agent deletes its own comment; the same agent UUID is matched exactly.
- An agent is forbidden from deleting logical-user and foreign-agent comments, with typed details and no mutation.
- Unknown task, absent comment, cross-task comment ID, malformed agent UUID, and unknown agent follow exact precedence and preserve state.
- Successful deletion returns the complete original comment and removes exactly that row.
- Task content, status, archive state/reason, claim, hierarchy, dependencies, `updated_at`, and `updated_by` remain unchanged.
- Inject deletion, affected-row, and commit failures and prove rollback preserves the comment.

### CLI, concurrency, and regression tests

- Snapshot exact help, human success, JSON success, and all stable error envelopes/exits.
- Verify omission of `--agent` selects logical-user authority and test this as actor selection, not physical-caller authentication.
- Coordinate two deletes of one comment and assert one success plus one `COMMENT_NOT_FOUND` without corruption.
- Repeat ownership and race coverage from a linked worktree sharing the canonical database.
- Verify listing omits only the deleted comment and retains deterministic ordering of survivors.
- Regression-test add/list behavior, archived-task parity, and absence of a `comments` field in task responses.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test user, owner-agent, and foreign-agent deletion on active and archived tasks in an isolated temporary repository, including one linked-worktree attempt.

## Acceptance scenario impact

`add`: E6-S2 adds a common public deletion workflow with ownership denial and archived-task behavior. After implementation, revalidate the E6 comment catalog and add independently executable E6-S2 scenarios through the acceptance workflow.

## Definition of done

- Every functional and non-functional E6-S2 criterion passes.
- Logical-user deletion and exact agent-owner deletion behave as documented; foreign-agent deletion fails without mutation.
- Scoped lookup, authorization, result capture, exact deletion, and commit are one immediate transaction.
- Task state and mutation metadata remain unchanged on success and failure.
- Human/JSON output, stable errors/details/exits, active/archived parity, rollback, concurrency, and linked-worktree tests pass.
- Help and contract describe the actor trust model without claiming caller authentication.
- Acceptance impact is recorded for handoff; scenario creation, revalidation, approval, and execution are not implementation completion criteria.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E6-S2 epic](../epics/E6-S2-delete-comments-under-ownership-rules.md)
- [PRD human actor](../PRD.md#61-human-user)
- [PRD coding-agent actor](../PRD.md#62-coding-agent)
- [PRD comment model](../PRD.md#710-comment)
- [PRD archive model](../PRD.md#711-archive)
- [PRD E6-S2 story](../PRD.md#story-e6-s2-delete-a-comment-under-ownership-rules)
- [E6-S1 authoritative comment contract](../epics/E6-S1-add-and-view-comments.md)
- [E6-S1 implementation task](E6-S1-T1-implement-task-comments.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
