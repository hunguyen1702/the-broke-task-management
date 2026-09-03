---
id: E6-S1
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E1-S3
  - E2-S1
---

# E6-S1: Add and view comments

## Outcome

Humans and registered agents can add durable Markdown comments to active or archived tasks and read the complete comment history in deterministic chronological order.

## User story

As a user or agent, I want to comment on a task so that decisions and progress are retained.

## Commands

```text
tbtm task comment add <task-id> --content <markdown> [--agent <uuid>] [--json]
tbtm task comment list <task-id> [--json]
```

`add` uses the logical `user` actor when `--agent` is omitted. Supplying `--agent` selects an exact registered agent UUID. `list` is actor-independent and read-only.

## Product decisions

### Comment identity and content

- Every comment receives a stable UUID v4 as its public ID.
- A comment stores its task ID, Markdown content, author actor, and one RFC 3339 UTC creation timestamp.
- Content is trimmed only to determine whether it is empty. A non-empty value is stored byte-for-byte as supplied, including surrounding whitespace and Markdown formatting.
- Empty or whitespace-only content is rejected with `INVALID_COMMENT_CONTENT` and creates no comment.
- MVP input is the explicit `--content` option. Editor launch, stdin, and file-based content are outside this story.

### Actor attribution

- Without `--agent`, the author is the logical actor `user`.
- With `--agent <uuid>`, the UUID must identify a registered repository agent. Base name and display name are not accepted as identity.
- JSON represents `author` as the scalar string `"user"` or the authoritative agent UUID, matching existing task actor metadata.
- Human output renders `user` for the logical user and both display name and UUID for an agent.

### Task lifecycle and metadata

- Comments can be added to active and archived tasks without confirmation.
- Adding a comment is an independent child-record mutation. It does not change task content, status, archive state, relationships, claim, `updatedAt`, or `updatedBy`.
- A comment does not replace or modify an archived task's canonical archive reason.
- E6-S1 exposes no comment editing or deletion. Ownership-based deletion belongs to E6-S2, and the immutable-history contract is reinforced by E6-S3.

### Listing and ordering

- `comment list` returns every comment for the specified task, whether the task is active or archived.
- Comments sort by `createdAt` ascending and then comment `id` ascending, producing deterministic chronological order when timestamps tie.
- A task with no comments returns success with an empty JSON array and a concise human empty-state message.
- Comment history remains separate from the shared E2-S2 full-task aggregate. `task view`, task lists, and mutation results do not gain a `comments` field in this story.

### Persistence and consistency

- Add a compatible versioned SQLite migration for comments without changing repository/config `schemaVersion: 1`.
- Comment rows reference their task and, for agent authors, the registered agent. Database and domain constraints preserve the valid `user` versus agent-UUID actor representation.
- Task lookup, optional agent lookup, UUID generation, timestamp capture, and insert complete atomically for one add operation; any failure leaves no comment row.
- Concurrent local processes and linked worktrees may add comments safely. UUID uniqueness is database-enforced, with bounded regeneration for an identity collision.
- Listing uses one read-only SQLite snapshot, applies no migrations, and creates no persistent database side effects.

### Output and errors

Successful `add` JSON returns the created comment in the shared envelope. Successful `list` JSON returns an array of the same comment shape:

```json
{
  "id": "UUID",
  "taskId": "project-task-a1b2c3d4",
  "content": "Markdown content",
  "author": "user or agent UUID",
  "createdAt": "RFC 3339 UTC timestamp"
}
```

Human `add` and non-empty `list` output show comment ID, author, creation timestamp, and content. Agent authors include both display name and UUID.

Relevant stable failures are:

- `INVALID_COMMENT_CONTENT`, exit code 2, for empty or whitespace-only content.
- `TASK_NOT_FOUND`, exit code 3, when the target task does not exist.
- `AGENT_NOT_FOUND`, exit code 3, when a supplied valid UUID is not registered.
- Malformed UUID and CLI syntax errors use exit code 2.
- Repository, database, permission, and unexpected errors retain shared codes and exit behavior.

Every failure leaves comments and task state unchanged.

## Functional acceptance criteria

1. A logical user or registered agent can add a non-empty Markdown comment to an active task and receive its stable ID, task ID, content, author, and creation time.
2. The same add behavior is supported for archived tasks without changing the archive reason or any other task field.
3. User attribution is stored and returned as `"user"`; agent attribution requires and returns the exact registered UUID, while human output also identifies the agent by display name.
4. Empty or whitespace-only content, malformed or unknown agents, and unknown tasks return the documented error and exit behavior without writing a comment.
5. Adding a comment preserves the task's `updatedAt`, `updatedBy`, status, archive state, claim, content, hierarchy, and dependencies.
6. Listing returns all comments for an active or archived task ordered by `createdAt` and then `id`, ascending.
7. Listing a task with no comments is a successful empty result; listing an unknown task returns `TASK_NOT_FOUND`.
8. Add and list use the documented human output and stable camelCase JSON comment shape without adding comments to the shared full-task aggregate.
9. Concurrent additions create distinct durable comments without partial rows, lost successful writes, or database corruption.

## Non-functional acceptance criteria

1. Comment insertion is atomic, UUID uniqueness and referential integrity are enforced in SQLite, and failures roll back completely.
2. Comment listing is read-only, observes one consistent snapshot, and never applies pending migrations.
3. Operations are safe across concurrent local processes and linked worktrees sharing the canonical database.
4. Add and list are local, network-free, and feel immediate for a personal repository.
5. Timestamps use RFC 3339 UTC; JSON fields remain stable camelCase; chronological ordering is deterministic.
6. Core owns validation, actor resolution, persistence, ordering, transactions, and typed errors; CLI owns argument parsing, rendering, and exit mapping.
7. Tests cover practical Linux, macOS, and Windows behavior through the workspace's existing portability strategy.

## Verification

- Apply the comment migration to fresh and existing compatible schema-version-1 repositories and verify earlier task and agent data is preserved.
- Add comments as `user` and as a registered agent to active and archived tasks; verify exact persistence, author attribution, and unchanged task metadata.
- Exercise empty content, malformed/unknown agent, unknown task, empty list, and archived list paths with exact outputs, exits, and no unintended writes.
- Force tied timestamps and verify ordering by timestamp then UUID; run concurrent additions and controlled UUID collisions to verify uniqueness and durability.
- Inject lookup/insert failures and verify transaction rollback leaves no partial comment.
- Confirm list uses a read-only snapshot, does not apply pending migrations, and has no persistent database side effects.
- Snapshot human and JSON add/list output and run format, lint, and workspace tests through `mise`.

## Out of scope

- Editing comment content or metadata.
- Deleting comments or enforcing deletion ownership; E6-S2.
- Correction workflows beyond later delete-and-recreate behavior; E6-S3.
- Adding comments to the E2-S2 full-task projection.
- Pagination, filtering, reverse ordering, reactions, attachments, mentions, or remote synchronization.
- Editor, stdin, or file-based comment input.

## Implementation task

See [E6-S1-T1: Implement task comments](../tasks/E6-S1-T1-implement-task-comments.md).
