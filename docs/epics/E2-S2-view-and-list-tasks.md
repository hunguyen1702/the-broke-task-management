---
id: E2-S2
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E2-S1
---

# E2-S2: View and list tasks

## Outcome

Users and agents can inspect one complete task or deterministically browse filtered repository work in stable human and JSON forms.

## User story

As a user or agent, I want to view task details and task lists so that I can inspect repository work.

## Commands

```text
tbtm task view <id> [--json]
tbtm task list [--archived | --all] [--status <code>]... [--type <type>]... [--tag <tag>]... [--json]
```

## Product decisions

### Scope and filters

- `task view` resolves an exact stable task ID regardless of archive state.
- `task list` returns active tasks by default. `--archived` returns only archived tasks; `--all` returns both. Supplying both flags is invalid.
- Status, type, and tag filters are repeatable. Values within one filter group use OR; different groups use AND.
- Status uses immutable machine codes. Type uses the fixed task-type tokens. Tag matching is exact and case-sensitive.
- Filters are applied within the selected archive scope. Full-text search, priority ranges, actor filters, claim filters, and custom sorting are outside this story.

### Deterministic ordering

- After scope and filters, lists order by priority descending, creation time ascending, then stable task ID ascending.
- The same ordering applies to active-only, archived-only, and combined scopes.
- This story intentionally exposes no custom sort option.

### Detail contract

- Detail output contains every scalar and structured field established by E2-S1, including Markdown context, status, tags, URLs, code references, archive state, timestamps, and actors.
- The normalized full-task shape uses `hierarchy: {parent, children}`, `dependencies: {upstream, downstream}`, and `claim` rather than the original E2-S1 `parentId` and dependency-array placeholders.
- Relationship collections contain only direct relationships. Recursive maps remain E4-S5 scope.
- A related-task summary contains `id`, `title`, `type`, and status `{code, name, completed}`.
- A claim contains agent `{id, displayName}` and `claimedAt`.
- Until E4 and E5 introduce authoritative relationship and claim data, detail and create output return `parent: null`, empty relationship collections, and `claim: null`. E4/E5 must retrofit this query and add regression tests when they introduce those records.
- E2-S2 is complete against the repository model available after E2-S1; it is not held open as a partial story merely because later stories add new relationship data.

### List contract

- Each list item contains `id`, `title`, `type`, `status`, `priority`, `estimate`, `tags`, `archived`, `claim`, `createdAt`, and `updatedAt`.
- List results omit Markdown bodies, external URLs, code references, actors, and hierarchy/dependency summaries.
- Human list output is a compact table. Human detail output shows all task fields and dedicated hierarchy, dependency, and claim sections.
- JSON uses the shared success envelope. An empty list succeeds with exit code `0` and `data: []`; human output is `No tasks found.`

### Errors and exit behavior

- A missing task returns `TASK_NOT_FOUND` with exit code `3`.
- An unknown status filter returns `STATUS_NOT_FOUND` with exit code `3` rather than silently producing an empty list.
- An invalid type returns `INVALID_TASK_TYPE` with exit code `2`.
- Conflicting archive flags return `CONFLICTING_ARGUMENTS` with exit code `2`.
- Repository, database, permission, and unexpected failures preserve the shared E1/E2 contracts.

## Functional acceptance criteria

1. `task view` returns the complete active or archived task selected by exact stable ID.
2. `task list` defaults to active tasks and supports mutually exclusive archived-only and combined scopes.
3. Repeatable status, type, and tag filters use OR within a group and AND across groups, with exact case-sensitive tag matching.
4. Every list is ordered by priority descending, creation time ascending, then task ID ascending after all scope and filters are applied.
5. Detail JSON uses the normalized hierarchy, dependency, and claim shapes; direct related-task and claim summaries use the documented fields.
6. List JSON uses only the documented projection, while human list and detail output remain concise and readable.
7. Empty lists succeed; missing tasks, unknown statuses, invalid types, and conflicting flags return the documented stable codes and exits.
8. Create and view share the normalized full-task relationship shape, with null/empty values until later authoritative models exist.

## Non-functional acceptance criteria

1. Queries are read-only, perform no network access, and do not apply migrations or mutate repository state.
2. Each command reads from one SQLite snapshot so task fields and child collections are internally consistent.
3. Query count is bounded rather than growing once per returned task; list filtering and ordering are performed by SQLite where practical.
4. JSON field names use camelCase, timestamps remain RFC 3339 UTC, and all collection and row ordering is deterministic.
5. Human output does not expose SQL details and remains usable in a normal terminal for repository-sized local task sets.

## Verification

- View minimum and fully populated tasks, including archived records, in human and JSON modes.
- Exercise each archive scope, filter group, OR/AND combination, tag case behavior, and ordering tie-breaker.
- Verify normalized relationship defaults and the E2-S1 create-output retrofit.
- Verify empty, invalid, unknown, and conflicting-input output plus exact exit codes.
- Inspect query behavior and one-snapshot consistency with multiple structured child rows.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Updating task content or actor metadata; E2-S3.
- Adding or editing structured task context; E2-S4.
- Archive and unarchive mutations; E2-S5 and E2-S6.
- Creating hierarchy or dependency records and recursive maps; E4.
- Creating or changing claims; E5.
- Full-text search, pagination, user-selected sorting, and new filter categories.

## Implementation task

See [E2-S2-T1: Implement task detail and listing](../tasks/E2-S2-T1-implement-task-detail-and-listing.md).
