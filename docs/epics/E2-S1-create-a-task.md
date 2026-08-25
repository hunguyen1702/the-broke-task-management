---
id: E2-S1
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E1-S1
  - E1-S3
  - E3-S1
---

# E2-S1: Create a task

## Outcome

A user or registered agent can create one validated, typed task with actionable coding context, stable identity, default workflow state, and auditable actor metadata.

## User story

As a user or agent, I want to create a typed task with coding context so that the work is actionable.

## Command

```text
tbtm task create --title <title> --type <type> [options] [--agent <uuid>] [--json]
```

Options covered by this story are `--description`, `--goal`, `--acceptance-criteria`, `--status`, `--priority`, `--estimate`, repeatable `--tag`, repeatable `--url`, and repeatable `--code-ref`.

## Product decisions

### Creation scope and defaults

- `title` and `type` are required. Title is trimmed and must remain non-empty.
- Type is one of the fixed machine values `epic`, `story`, `task`, `improvement`, `refactor`, `bug`, `spike`, `testing`, or `poc`.
- Description, goal, and acceptance criteria are Markdown strings and default to empty strings.
- Status defaults to the stable code `to_do`; callers may select another configured status with `--status <code>`.
- Priority defaults to `50` and must remain within `0..=1_000_000`.
- Estimate is optional, expressed only in hours, and accepts a finite non-negative decimal including zero.
- Tags, external URLs, and code references default to empty collections.
- Newly created tasks are active and unclaimed. Parent and dependency creation are deferred to E4.

### Status identity and display

- A status keeps its UUID as the database foreign-key identity and also has an immutable, repository-unique machine code separate from its display name.
- Status codes match `[a-z][a-z0-9_]*` and are selected by code, never by display name or UUID, at the CLI boundary.
- The canonical defaults are exactly `to_do` → `Todo`, `in_progress` → `In progress`, and `done` → `Done`.
- JSON returns status `{id, code, name, completed}`; human output uses the display name.
- E3-S1 owns status codes in the fresh baseline schema and provides lookup by code for task creation. Future custom-status creation requires an explicit immutable code; renaming changes only the display name.

### Structured coding context

- Repeat `--tag`, `--url`, and `--code-ref` to provide multiple values.
- Tags are trimmed, must be non-empty, and are compared exactly for duplicates.
- External URLs represent task-related issue, pull-request, documentation, CI, or reference links. They must be absolute `http` or `https` URLs, are preserved after validation, and reject exact duplicates.
- Repository files use code references rather than external URLs.
- A code reference uses `path[:start[-end]][::description]`. The path is a non-empty lexical repository-relative path; it must not be absolute or escape the repository. Lines are one-based positive integers and `end` cannot precede `start`.
- A referenced file need not exist when the task is created. This permits planning references to files that implementation will add later.
- Empty or duplicate structured values are rejected rather than silently dropped.

### Stable task identity

- Generate IDs as `<stored-repository-prefix>-<type-at-creation>-<short-uuid>`.
- Type segments use the fixed machine values above; `poc` is the proof-of-concept segment.
- The short UUID is the first eight lowercase hexadecimal characters of a UUID v4 without hyphens.
- Persist the short suffix separately and enforce repository-wide uniqueness on it in SQLite, in addition to complete-ID uniqueness. A suffix collision across any task types rolls back that attempt, generates a new UUID, and retries within a bounded operation.
- The ID never changes, including when later stories allow mutable task fields or type changes.

### Actor metadata

- Without `--agent`, the logical actor is `user`.
- With `--agent <uuid>`, the UUID must identify a registered repository agent; display name and base name are not accepted as actor identity.
- Creation sets `createdBy` and `updatedBy` to the same actor and uses one UTC timestamp for `createdAt` and `updatedAt`.
- `createdBy` remains immutable. E2-S3 may set `updatedBy` for later mutations, while E5 claim operations represent work assignment.
- The product does not add a separate task-to-agent assignment relationship.

### Persistence and output

- The task row and all tag, URL, and code-reference rows commit in one SQLite transaction or leave no task artifacts.
- JSON returns the entire persisted task in the shared envelope. Optional scalar fields use `null`; collections use `[]`.
- Human success output shows at least ID, title, type, status display name, priority, and actor.
- The command is non-interactive when required values and options are supplied.

### Errors and exit behavior

- Input validation failures return stable validation codes with exit code `2` and no mutation.
- An unknown agent UUID or status code returns a stable not-found error with exit code `3` and no mutation.
- Exit code `4` remains reserved for E5 claim conflicts.
- Repository, configuration, database, permission, and unexpected errors preserve the shared contracts from E1.

## Functional acceptance criteria

1. A user or registered agent can create one task with every field in this story, and omitted optional fields receive the documented defaults.
2. Required title/type, fixed type values, status code, priority, estimate, URLs, and code references are validated before commit.
3. Omitted status resolves to `to_do`; explicit status selection uses an existing immutable machine code and output distinguishes code from display name.
4. Tags, URLs, and code references accept multiple ordered inputs and reject empty or duplicate values.
5. Every task receives a collision-safe ID containing the stored repository prefix, type-at-creation token, and eight-character UUID-derived suffix; the suffix itself is unique across all repository tasks.
6. Creation records one actor and timestamp consistently in both created and updated metadata; unknown agents create nothing.
7. The task and every associated context row are atomic.
8. Human and JSON success output follow the documented shapes; validation and not-found failures preserve exit codes `2` and `3` respectively.
9. Creation does not add parents, dependencies, claims, comments, archive state changes, or a separate agent assignment.

## Non-functional acceptance criteria

1. Creation performs no network access and feels immediate for a local personal repository.
2. SQLite constraints and transaction boundaries prevent partial tasks and duplicate IDs under concurrent local processes and linked worktrees.
3. Timestamps use RFC 3339 UTC, JSON fields use camelCase, and collection ordering is deterministic.
4. Code-reference paths remain portable repository-relative values and do not require filesystem existence.
5. Tests cover Linux, macOS, and Windows behavior at a practical level.

## Verification

- Create minimum and fully populated tasks as both `user` and a registered agent.
- Query SQLite to verify defaults, status UUID linkage, actor columns, structured context, and one-transaction behavior.
- Force an ID collision and concurrent creates to verify retry and uniqueness behavior.
- Exercise every validation boundary, missing agent/status, duplicate collection value, and malformed code reference.
- Snapshot human and JSON output and assert the shared exit-code categories.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Viewing and listing existing tasks; E2-S2.
- Updating task content or actor history; E2-S3.
- Managing structured context after creation; E2-S4.
- Archive and unarchive behavior; E2-S5 and E2-S6.
- Parent and dependency relationships; E4.
- Claiming or assigning work; E5.
- Comments; E6.
- Interactive prompting, editor launch, stdin/file-based Markdown ingestion, or permanent task deletion.

## Implementation task

See [E2-S1-T1: Implement task creation](../tasks/E2-S1-T1-implement-task-creation.md).
