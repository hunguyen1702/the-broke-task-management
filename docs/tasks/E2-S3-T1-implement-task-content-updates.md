---
id: E2-S3-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E2-S1-T1
  - E2-S2-T1
  - E4-S1-T1
---

# E2-S3-T1: Implement task-content updates

## Parent story

[E2-S3: Update task content](../epics/E2-S3-update-task-content.md)

## Objective

Implement atomic partial updates for active-task scalar content, including actor metadata, no-op semantics, hierarchy-safe type changes, and the shared full-task output contract.

## Deliverables

- Typed core patch input and update result models.
- Transactional task lookup, validation, direct-hierarchy checks, mutation, and aggregate reload.
- `tbtm task update` CLI parsing, actor selection, human rendering, JSON envelope, and exit mapping.
- Focused core tests and process-level CLI tests for fields, hierarchy, actors, no-ops, archived tasks, output, and concurrency.

## Proposed structure

Extend the existing task modules and E4-S1 hierarchy query primitives without moving business logic into the CLI:

```text
crates/tbtm-core/src/task/
├── update.rs
└── model.rs

crates/tbtm-cli/src/commands/
└── task_update.rs
```

Exact files should follow the repository layout at implementation time. Core owns patch semantics, validation, transactions, hierarchy invariants, and typed errors. CLI owns Clap parsing, actor arguments, envelopes, rendering, and exit codes.

## Technical choices

- Reuse E2-S1 task types, scalar validators, status lookup, actor representation, and timestamp format.
- Reuse the authoritative E4-S1 parent/direct-child model and validation rules; do not introduce a second hierarchy representation.
- Reuse E2-S2 full-task loading and serialization for success output.
- Model every patch member so omitted and explicitly supplied values are distinguishable. Empty Markdown is a supplied value, not omission.
- Keep tags, URLs, estimate, and code references out of this patch model; E2-S4 owns their mutation semantics.
- Use one write transaction for task lookup, actor/status resolution, hierarchy reads, validation, effective-change detection, update, and final aggregate read.

## Patch and validation contract

The patch supports title, description, goal, acceptance criteria, type, status code, and priority. Reject an input containing none of these fields.

Resolve the exact task including archive state, then validate all supplied inputs before deciding whether the patch is an effective change:

1. Resolve the actor as logical `user` or exact registered agent UUID.
2. Validate title, fixed type token, priority range, and any supplied status code.
3. If type changes, read the current parent and all direct children through E4-S1 and validate every resulting direct edge against the type matrix.
4. Reject archived tasks without modifying task content or metadata.
5. Compare normalized validated values with persisted values.

This ordering means an invalid actor, status, type, or value never becomes hidden by an otherwise identical patch.

## Transaction flow

1. Resolve the canonical writable repository and apply compatible pending migrations through the shared mutation-command path.
2. Begin a SQLite write transaction suitable for serializing hierarchy validation with competing local hierarchy mutations.
3. Load the exact task. Return `TASK_NOT_FOUND` if absent.
4. Resolve actor/status and validate the complete supplied patch.
5. Reject archived state with `TASK_ARCHIVED`.
6. For a supplied changed type, validate the current parent and direct children inside this transaction.
7. If every supplied normalized value equals persisted data, load and return full detail without issuing writes.
8. Otherwise update supplied scalar columns plus one `updated_at`/`updated_by` pair. Never update ID, short suffix, creation type segment, `created_at`, or `created_by`.
9. Reload the complete E2-S2 aggregate in the same transaction, commit, and render it.

Any failure rolls back the transaction and leaves scalar values and actor metadata unchanged.

## CLI and output contract

Expose:

```text
tbtm task update <id> \
  [--title <title>] \
  [--description <markdown>] \
  [--goal <markdown>] \
  [--acceptance-criteria <markdown>] \
  [--type <type>] \
  [--status <code>] \
  [--priority <integer>] \
  [--agent <uuid>] [--json]
```

The command is non-interactive. An explicitly empty Markdown option clears that field; title cannot normalize to empty. Success uses the same normalized full-task data and shared envelope as `task view`; human output uses the shared task-detail renderer where practical.

Stable errors:

| Condition | Code | Exit |
|---|---|---:|
| No mutable field supplied | `NO_UPDATE_FIELDS` | 2 |
| Invalid title | `INVALID_TASK_TITLE` | 2 |
| Invalid type | `INVALID_TASK_TYPE` | 2 |
| Invalid priority | `INVALID_TASK_PRIORITY` | 2 |
| Type invalidates a direct hierarchy edge | `INVALID_TASK_HIERARCHY` | 2 |
| Task is archived | `TASK_ARCHIVED` | 2 |
| Task missing | `TASK_NOT_FOUND` | 3 |
| Status missing | `STATUS_NOT_FOUND` | 3 |
| Agent missing | `AGENT_NOT_FOUND` | 3 |

Exit `4` remains reserved for claim conflicts. Shared repository/configuration/database/permission/unexpected errors retain their existing codes and exits.

## Test plan

### Core tests

- Patch every supported field alone and in combinations; prove omitted fields remain unchanged.
- Clear each Markdown field; reject an empty normalized title and priority boundaries outside `0..=1_000_000`.
- Preserve ID, short suffix, creation metadata, and unrelated structured context.
- Exercise changed types with no relationships, valid/invalid parents, and valid/invalid direct children across the hierarchy matrix.
- Prove hierarchy validation and update roll back together and remain valid under competing local transactions.
- Verify user and registered-agent attribution, unknown agent/status behavior, and one timestamp per effective mutation.
- Verify fully valid identical patches issue no writes and preserve update metadata.
- Reject archived and missing tasks without mutation.

### CLI integration tests

- Assert parsing and output for every option in human and JSON modes.
- Snapshot success against E2-S2 full detail and stable shared envelopes.
- Assert every documented code and exit, including no-field, archived, hierarchy, not-found, and invalid-input cases.
- Run effective and no-op updates from the main and a linked worktree against the canonical shared database.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test a temporary initialized repository with standalone and related active tasks. Update as both actor kinds, inspect SQLite metadata, test valid no-ops and every failure category, and compare output with `task view`.

## Definition of done

- All E2-S3 functional and non-functional criteria pass.
- Partial patches cannot alter omitted fields, stable identity, creation metadata, structured E2-S4 fields, relationships, claims, or archive state.
- Type changes preserve all current direct hierarchy edges under the authoritative E4-S1 model.
- Effective mutations and valid no-ops follow the documented actor/timestamp behavior.
- Archived and invalid updates leave the database unchanged and use exact stable errors/exits.
- Success output matches E2-S2 full detail in both modes.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E2-S3 epic](../epics/E2-S3-update-task-content.md)
- [E2-S1 create-task epic](../epics/E2-S1-create-a-task.md)
- [E2-S2 task-query epic](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S1 task creation](E2-S1-T1-implement-task-creation.md)
- [E2-S2 detail and listing](E2-S2-T1-implement-task-detail-and-listing.md)
- E4-S1 hierarchy epic and task, once planned.
