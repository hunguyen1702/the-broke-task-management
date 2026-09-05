---
id: E6-S2
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E6-S1
---

# E6-S2: Delete comments under ownership rules

## Outcome

Humans can remove any unwanted task comment, while a registered agent acting under its declared identity can remove only comments authored by that same identity.

## User story

As an agent, I want to delete my own comment, and as a user I want to delete any comment, so that incorrect or unwanted content can be removed.

## Command

```text
tbtm task comment delete <task-id> <comment-id> [--agent <uuid>] [--json]
```

Omitting `--agent` selects the logical `user`. Supplying `--agent <uuid>` selects that exact registered agent.

## Product decisions

### Actor authority and trust boundary

- The logical `user` may delete any comment on the specified task.
- A registered agent may delete a comment only when the stored comment author is the exact same agent UUID.
- An agent cannot delete a logical-user comment or a comment authored by another agent identity.
- Actor selection follows E6-S1: omitting `--agent` selects logical-user authority; supplying the option requires a registered UUID.
- This is repository-level actor attribution and authorization, not authentication of the physical caller. Any process can omit `--agent`; trusted automation is expected to identify itself. Credentials, OS/process identity, and prevention of user-mode invocation by an agent are outside this story.

### Targeting and deletion

- Both task ID and comment ID are required so the command is explicit and consistent with the task-comment command group.
- The task must exist, and the comment must belong to that task. A missing comment or a comment belonging to another task returns `COMMENT_NOT_FOUND` and does not disclose its other task.
- Successful deletion permanently removes the comment row. There is no confirmation, trash, tombstone, or soft-delete state in the MVP.
- The same behavior applies to active and archived tasks without changing archive reason or state.
- Deletion does not change the task's content, status, relationships, claim, `updatedAt`, or `updatedBy`.

### Atomic concurrency behavior

- After compatible pending migrations are applied, actor resolution, task lookup, scoped comment lookup, ownership validation, deletion, and deleted-result capture execute in one immediate SQLite transaction.
- The delete is constrained to the exact comment and task observed in that transaction.
- Two concurrent permitted attempts cannot both succeed. The first committed deletion succeeds; a later contender observes `COMMENT_NOT_FOUND`.
- Busy-timeout exhaustion remains an operational database error. Every failure rolls back without deleting a comment or changing its task.

### Success output

- JSON returns the deleted comment using the unchanged E6-S1 comment shape under the shared success envelope: `id`, `taskId`, `content`, `author`, and `createdAt`.
- Human output concisely confirms the deleted comment ID and task ID.
- Returning the captured deleted record does not retain a tombstone or add comments to the full-task aggregate.

### Errors and exits

| Code | Exit | Condition and stable details |
|---|---:|---|
| `COMMENT_NOT_FOUND` | 3 | No comment with the supplied ID belongs to the supplied task; details are `{taskId, commentId}` |
| `COMMENT_DELETE_FORBIDDEN` | 5 | The selected agent does not own the comment; details identify `{taskId, commentId, author}` |
| `AGENT_NOT_FOUND` | 3 | The supplied valid agent UUID is not registered |
| `TASK_NOT_FOUND` | 3 | The supplied task ID does not exist |

Malformed UUID and CLI syntax failures exit `2`. Repository, database, migration, permission, busy-timeout, and unexpected failures retain shared behavior. Errors use the shared envelope and leave comment and task state unchanged.

Validation precedence is command syntax and UUID parsing, registered agent when supplied, task existence, scoped comment existence, then ownership.

## Functional acceptance criteria

1. The logical user can delete a user-authored or agent-authored comment from an active or archived task.
2. A registered agent can delete its own comment from an active or archived task.
3. A registered agent cannot delete a logical-user comment or another agent's comment; the command returns `COMMENT_DELETE_FORBIDDEN` without mutation.
4. Unknown tasks, comments absent from the specified task, malformed agents, and unknown agents produce the documented precedence, error details, exits, and no-write behavior.
5. Successful deletion permanently removes exactly one scoped comment and preserves every field and relationship of its parent task.
6. JSON success returns the deleted E6-S1 comment shape; human success identifies its comment and task IDs.
7. Concurrent delete attempts yield at most one success, never delete an unrelated comment, and leave the database usable.
8. Actor selection and its non-authenticating trust boundary are documented consistently in help-facing behavior and tests.

## Non-functional acceptance criteria

1. Lookup, ownership validation, deleted-record capture, and exact deletion are atomic in one SQLite transaction.
2. The operation is safe across concurrent local processes and linked worktrees sharing the canonical database.
3. Core owns actor resolution, authorization, persistence, transactions, and typed errors; CLI owns parsing, rendering, and exit mapping.
4. JSON remains stable camelCase, and the deleted record preserves its original RFC 3339 UTC timestamp and author representation.
5. Deletion is local, network-free, and feels immediate for a personal repository under ordinary contention.

## Verification

- Delete user- and agent-authored comments as logical user on active and archived tasks.
- Delete an owned agent comment, then reject user-authored and foreign-agent comments with exact output, exit, and no mutation.
- Exercise malformed/unknown agents, unknown tasks, absent comments, and cross-task comment IDs according to the validation precedence.
- Verify successful and failed deletion leave task fields, archive reason, claim, hierarchy, dependencies, and mutation metadata unchanged.
- Race permitted deletes from the main and a linked worktree; verify exactly one success and one current-state `COMMENT_NOT_FOUND`.
- Snapshot human and JSON success/error output and run format, lint, and workspace tests.

## Out of scope

- Editing comment content; E6-S3 reinforces comment immutability.
- Authentication, credentials, operating-system identity, or preventing a process from selecting logical-user authority.
- Soft deletion, recovery, deletion history, audit tombstones, batch deletion, and confirmation prompts.
- Deleting a task as a consequence of deleting its comments.
- Adding comments to task view or other full-task responses.

## Implementation task

See [E6-S2-T1: Implement ownership-aware comment deletion](../tasks/E6-S2-T1-implement-ownership-aware-comment-deletion.md).
