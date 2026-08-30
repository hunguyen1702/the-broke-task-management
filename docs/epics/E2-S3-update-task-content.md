---
id: E2-S3
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E2-S1
  - E4-S1
---

# E2-S3: Update task content

## Outcome

Users and registered agents can partially update the mutable scalar content of an active task while preserving its identity, hierarchy validity, and creation metadata.

## User story

As a user or agent, I want to update task context so that it remains accurate during implementation.

## Command

```text
tbtm task update <id> [fields] [--agent <uuid>] [--json]
```

Fields covered here are `--title`, `--description`, `--goal`, `--acceptance-criteria`, `--type`, `--status`, and `--priority`.

## Product decisions

### Mutable-field boundary

- Update is a partial patch: only explicitly supplied fields are candidates for change.
- Title is trimmed and must remain non-empty. Description, goal, and acceptance criteria are Markdown strings and may be cleared with an explicitly supplied empty string.
- Type, status, and priority reuse E2-S1 validation. Status is selected by immutable machine code and priority remains within `0..=1_000_000`.
- Tags, URLs, estimate, and code references are managed by E2-S4. Parent mutation is managed only by E4-S1. Relationships, claims, comments, and archive state are not changed here.
- Supplying no update field is invalid. CLI parser conventions may expose field names in kebab case while JSON continues to use camelCase.

### Identity and hierarchy

- Task ID, its type-at-creation segment, short suffix, `createdAt`, and `createdBy` never change.
- A type change is validated against both the current parent and every direct child using the hierarchy type matrix.
- Parent assignment, removal, and replacement remain E4-S1 behavior. E2-S3 only ensures a type update cannot make existing direct hierarchy edges invalid.
- The task, current parent, and direct children are read and validated in the same SQLite transaction that performs the update, preventing a competing local mutation from invalidating the check before commit.

### Actor metadata and no-op behavior

- Without `--agent`, the responsible actor is logical `user`. With `--agent <uuid>`, the UUID must resolve to a registered repository agent.
- An effective update writes one RFC 3339 UTC timestamp to `updatedAt` and the resolved actor to `updatedBy`.
- Supplied actor, status, type, and field values are fully resolved and validated before no-op detection.
- A fully valid patch whose persisted values are all unchanged succeeds and returns the task without changing `updatedAt`, `updatedBy`, or any database row.

### Active-only mutation and output

- Archived task content is immutable. Updating an archived task fails without mutation; the task must first be unarchived through E2-S6.
- Success returns the complete persisted task using the same normalized detail contract as `task view`, in the shared JSON envelope or human detail form.
- The mutation and returned aggregate are transactionally consistent. No partial scalar or metadata update is visible.

### Errors and exits

- Missing task, unknown status code, or unknown agent UUID returns `TASK_NOT_FOUND`, `STATUS_NOT_FOUND`, or `AGENT_NOT_FOUND` with exit code `3`.
- Invalid title, type, priority, hierarchy result, or absence of update fields returns a stable validation error with exit code `2`.
- Updating an archived task returns `TASK_ARCHIVED` with exit code `2`.
- Exit code `4` remains reserved for E5 claim conflicts. Repository and unexpected failures preserve the shared E1/E2 categories.

## Functional acceptance criteria

1. A user or registered agent can patch any supported mutable field of an active task without resupplying unchanged fields.
2. Title, type, status, and priority use the established validation rules; Markdown fields may be explicitly cleared.
3. A type change is rejected atomically if the resulting type is incompatible with the current parent or any direct child.
4. Task ID and creation metadata remain unchanged for every successful update.
5. An effective update records exactly one responsible actor and UTC update timestamp; an unknown agent mutates nothing.
6. A fully validated identical patch succeeds without changing persisted data or update metadata; a command with no update field is rejected.
7. Archived tasks and every invalid or missing input fail with the documented stable code and no mutation.
8. Human and JSON success output matches the E2-S2 full-task detail contract.

## Non-functional acceptance criteria

1. Hierarchy validation, scalar mutation, actor metadata, and aggregate reading are atomic and consistent under concurrent local processes and linked worktrees.
2. The command performs no network access and feels immediate for a local personal repository.
3. Parameterized SQLite operations, typed core inputs, and deterministic serialization preserve the CLI/core boundary.
4. Timestamps remain RFC 3339 UTC and JSON fields remain camelCase.
5. Tests cover practical Linux, macOS, and Windows CLI behavior.

## Verification

- Patch each field independently and in combinations as `user` and a registered agent.
- Verify title, type, status, priority, missing task, missing agent, archived task, and no-field failures plus exact exits.
- Exercise every relevant parent/child type-matrix boundary and confirm rejected changes leave task and metadata untouched.
- Compare effective-update and valid-no-op timestamps, actors, and persisted rows.
- Snapshot human and JSON output against the E2-S2 detail shape.
- Exercise concurrent hierarchy/update transactions from repositories sharing one canonical store.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Tags, URLs, estimates, or code references; E2-S4.
- Parent assignment, removal, or replacement; E4-S1.
- Archive and unarchive; E2-S5 and E2-S6.
- Dependency and claim mutations; E4 and E5.
- Comments, permanent deletion, interactive editing, or Markdown input from files/stdin.

## Implementation task

See [E2-S3-T1: Implement task-content updates](../tasks/E2-S3-T1-implement-task-content-updates.md).
