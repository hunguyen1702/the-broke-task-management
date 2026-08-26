---
id: E2-S2-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E2-S1-T1
---

# E2-S2-T1: Implement task detail and listing

## Parent story

[E2-S2: View and list tasks](../epics/E2-S2-view-and-list-tasks.md)

## Objective

Implement read-only task detail and deterministic filtered listing, normalize the shared full-task output contract, and provide stable human, JSON, empty-result, and error behavior.

## Deliverables

- Core read models and read-only queries for one task and task lists.
- Archive-scope, status, type, and tag filter input models.
- Deterministic SQLite ordering and bounded aggregate loading.
- `tbtm task view` and `tbtm task list` CLI commands.
- Shared full-task and compact list serializers plus human renderers.
- Retrofit of `task create` output to the normalized hierarchy/dependency shape.
- Unit and process-level integration tests for query, output, and error contracts.

## Proposed structure

Extend the implemented task modules while preserving the CLI/core boundary:

```text
crates/tbtm-core/src/task/
├── query.rs
└── model.rs

crates/tbtm-cli/src/commands/
├── task_view.rs
└── task_list.rs
```

Exact files may follow the current repository layout. Core owns filters, read models, SQLite queries, and typed errors. CLI owns Clap parsing, rendering, envelopes, and exit mapping.

## Technical choices

- Resolve the canonical E1-S5 repository in read-only, no-create mode and do not run pending migrations from either query command.
- Read each command from one SQLite transaction/snapshot.
- Express archive scope, filters, and ordering in parameterized SQL. Load child collections in bounded bulk queries or equivalent aggregate queries; avoid per-task query growth.
- Reuse E2-S1 status, type, task-context, actor, and timestamp models instead of defining competing identities.
- Keep related-task and claim output types explicit so E4/E5 can populate them without changing the JSON shape.
- Apply no schema migration for placeholder relationship fields.

## Query models

Represent list input with:

- archive scope: active-only, archived-only, or all;
- zero or more status codes;
- zero or more fixed type tokens;
- zero or more exact case-sensitive tags.

Within each non-empty filter group, match any value. Across non-empty groups, require all groups. Validate type tokens before database access. Resolve every distinct status code; if any requested code is unknown, return `STATUS_NOT_FOUND` rather than partial results.

Order final rows by `priority DESC, created_at ASC, id ASC`. Preserve ordinal order for tags, URLs, and code references in detail output. Relationship collections use deterministic task-ID ordering until their owning stories define a stronger domain order.

## Implementation flow

### 1. Parse commands

Provide:

```text
tbtm task view <id> [--json]
tbtm task list \
  [--archived | --all] \
  [--status <code>]... \
  [--type <type>]... \
  [--tag <tag>]... \
  [--json]
```

Reject `--archived` with `--all` as `CONFLICTING_ARGUMENTS`. Keep both commands non-interactive.

### 2. Resolve repository and validate selectors

1. Resolve the shared repository and open the existing SQLite database read-only.
2. Parse fixed type tokens and reject invalid values before querying.
3. Resolve supplied status codes within the command snapshot. Reject the command if any code is unknown.
4. Do not mutate schema, migration state, timestamps, or task records.

### 3. Read task detail

Select the exact task ID without filtering archive state. Load its scalar fields, status, actors, tags, URLs, and code references within one snapshot. Return `TASK_NOT_FOUND` if no task exists.

Build the normalized full-task result:

```json
{
  "hierarchy": {"parent": null, "children": []},
  "dependencies": {"upstream": [], "downstream": []},
  "claim": null
}
```

The omitted keys are the complete E2-S1 task fields. A future related-task summary is `{id, title, type, status: {code, name, completed}}`; a future claim is `{agent: {id, displayName}, claimedAt}`. Only direct relationships populate this contract; E4-S5 owns recursive maps.

### 4. Read task lists

Apply archive scope and all filters in SQL before deterministic ordering. Fetch the compact projection for all matching rows and load tags in a bounded query. Return an empty collection normally.

Each JSON list item is equivalent to:

```json
{
  "id": "project-task-a1b2c3d4",
  "title": "Implement parser",
  "type": "task",
  "status": {"id": "UUID", "code": "to_do", "name": "Todo", "completed": false},
  "priority": 50,
  "estimate": null,
  "tags": [],
  "archived": false,
  "claim": null,
  "createdAt": "RFC 3339 UTC timestamp",
  "updatedAt": "RFC 3339 UTC timestamp"
}
```

Do not load or serialize Markdown fields, URLs, code references, actors, or relationship summaries for list output.

### 5. Normalize shared output and render

Replace the implemented create serializer's `parentId` and dependency-array placeholders with the normalized hierarchy and dependency objects used by detail. Keep all other E2-S1 fields stable.

Human detail prints every task field and separate hierarchy, dependency, and claim sections. Human list uses a compact table for the list projection; exact column widths may adapt to the terminal without changing JSON. Print exactly `No tasks found.` for an empty human list.

JSON continues to use the shared envelope, with the detail object or list array under `data`.

### 6. Map errors and exits

| Exit | Conditions |
|---:|---|
| 0 | Detail found or list completed, including empty list |
| 1 | Database or other operational failure |
| 2 | `INVALID_TASK_TYPE` or `CONFLICTING_ARGUMENTS` |
| 3 | `TASK_NOT_FOUND`, `STATUS_NOT_FOUND`, or repository not found |
| 4 | Reserved for claim conflict; never returned here |
| 5 | Permission denied |

Use the shared JSON error envelope and do not expose SQL or parser internals.

## Test plan

### Core tests

- Fetch minimum and fully populated active and archived task aggregates.
- Preserve structured-context ordinals and one-snapshot consistency.
- Cover active-only, archived-only, and all scopes.
- Cover each filter alone, repeated OR values, cross-group AND behavior, tag case sensitivity, and no matches.
- Verify priority, creation-time, and ID ordering tie-breakers after filtering in every scope.
- Verify unknown status and missing task typed errors.
- Assert a bounded query count for lists with multiple tasks and tags.

### CLI integration tests

- Snapshot human and JSON detail output for minimum and fully populated tasks.
- Snapshot the exact list projection and verify excluded long/detail fields.
- Verify empty human text, JSON `data: []`, and exit `0`.
- Verify invalid type, unknown status, missing task, conflicting flags, and exact exit codes.
- Verify view includes archived tasks while default list excludes them.
- Verify create output now shares the normalized relationship shape with view.
- Run queries from main and linked worktrees against the same canonical database.

### Future integration obligations

- E4 hierarchy and dependency tasks must populate direct summaries and regression-test create/view/list compatibility without changing this shape.
- E5 claim tasks must populate claim summaries in detail and list output and regression-test unclaimed null behavior.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test a temporary repository by creating tasks with varied priorities, statuses, types, tags, timestamps, and archive flags; then run every detail, scope, filter, empty, and invalid query in both output modes.

## Definition of done

- Every functional and non-functional criterion in E2-S2 passes against the post-E2-S1 repository model.
- Detail and create share the normalized full-task relationship contract.
- List scope, filtering, projection, and ordering are stable and tested.
- Empty results and all documented errors use exact output and exits.
- Commands are read-only, snapshot-consistent, network-free, and avoid per-task query growth.
- No hierarchy, dependency, claim, update, archive mutation, or custom-sort behavior is introduced.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E2-S2 epic](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S1 create-task epic](../epics/E2-S1-create-a-task.md)
- [E2-S1 task creation](E2-S1-T1-implement-task-creation.md)
- [E1-S2 repository resolution](../epics/E1-S2-resolve-repository-configuration.md)
- [E1-S5 shared worktree resolution](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [E3-S1 default statuses](../epics/E3-S1-use-default-statuses.md)
