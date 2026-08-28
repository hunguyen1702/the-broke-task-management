---
id: E2-S4-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E2-S1-T1
  - E2-S2-T1
---

# E2-S4-T1: Implement structured task-context updates

## Parent story

[E2-S4: Manage tags, URLs, estimates, and code references](../epics/E2-S4-manage-structured-task-context.md)

## Objective

Implement atomic partial replacement and clearing of task estimates, tags, external URLs, and code references through the shared `task update` command.

## Deliverables

- Typed tri-state patch inputs that distinguish omission, replacement, and clearing.
- Transactional core operation for lookup, validation, no-op detection, collection replacement, actor metadata, and aggregate reload.
- Structured-field options on `tbtm task update`, shared full-detail rendering, stable errors, and exit mapping.
- Focused validation, persistence, transaction, no-op, concurrency, and process-level CLI tests.

## Proposed structure

Extend the task modules that own the existing E2-S1 validators/storage and E2-S2 detail loader:

```text
crates/tbtm-core/src/task/
├── update.rs
└── model.rs

crates/tbtm-cli/src/commands/
└── task_update.rs
```

Exact files should follow the repository layout at implementation time. Core owns normalization, validation, replacement semantics, transaction boundaries, typed errors, and mutation results. CLI owns option parsing, actor arguments, envelopes, rendering, and exit codes.

E2-S4 may create the shared update modules and command before E2-S3. Structure them so E2-S3 can add scalar patch members and hierarchy validation without duplicating the command or transaction orchestration.

## Technical choices

- Reuse E2-S1 estimate, tag, URL, and code-reference parsers and domain validators; do not create update-only interpretations.
- Reuse E2-S1 child tables and ordinal columns. No schema change is expected.
- Reuse the E2-S2 full-task aggregate loader and serializers for the transactionally consistent success result.
- Represent estimate as omitted, set to a finite non-negative value, or clear to `null`.
- Represent each collection as omitted, replace with a non-empty validated ordered list, or clear to an empty list.
- Normalize the complete supplied patch before effective-change detection. Compare ordered normalized collections, not unordered membership.
- Use one SQLite write transaction for task/actor lookup, archived-state check, validation, effective-change detection, replacements, metadata, and final aggregate read.
- Replace a changed collection by deleting its current child rows and inserting the new list with contiguous ordinals inside that transaction. Do not touch an omitted or identical collection.
- Reserve one update orchestration path for combining future E2-S3 scalar members with these structured members atomically.

## CLI and patch contract

Expose:

```text
tbtm task update <id> \
  [--estimate <decimal-hours> | --clear-estimate] \
  [--tag <tag>]... [--clear-tags] \
  [--url <absolute-http-url>]... [--clear-urls] \
  [--code-ref <reference>]... [--clear-code-refs] \
  [--agent <uuid>] [--json]
```

For each collection, one or more value options replace the complete persisted list in supplied order. An omitted field is unchanged. Its clear flag produces an empty list and conflicts with its value option. Estimate follows the same omitted/set/clear distinction, with zero treated as a set value rather than clear.

Parse clear flags and value options without declaring them as native Clap conflicts, then reject a same-field combination in application validation as `CONFLICTING_ARGUMENTS`, exit `2`. This keeps human and JSON failures in the shared CLI error envelope. Core must still receive an unambiguous typed patch and enforce that at least one mutable field is present for non-CLI callers.

## Validation contract

- Estimate must be finite and at least zero, including when Rust floating-point parsing accepts textual non-finite values.
- Trim tags, reject normalized empties, and reject exact case-sensitive duplicates after trimming.
- Require absolute `http`/`https` URLs, preserve accepted strings, and reject exact duplicates.
- Parse code references exactly as E2-S1: split optional `::description`, recognize only a final numeric `:start` or `:start-end`, validate lexical repository-relative paths, and do not check filesystem existence.
- Require positive one-based lines and `end >= start`; reject exact duplicate normalized code-reference structures.
- Validate all supplied collections before deleting or inserting any child row.

Use the existing E2-S1 stable codes `INVALID_ESTIMATE`, `INVALID_EXTERNAL_URL`, and `INVALID_CODE_REFERENCE` for malformed scalar/structured values. Empty or duplicate tags, duplicate URLs, and duplicate normalized code references use the existing `DUPLICATE_TASK_CONTEXT`. Use `CONFLICTING_ARGUMENTS` for a clear flag combined with values for the same field. Use `NO_UPDATE_FIELDS`, `TASK_ARCHIVED`, `TASK_NOT_FOUND`, and `AGENT_NOT_FOUND` consistently with E2-S3. Every validation error exits `2` through the shared envelope and must not mutate data.

## Implementation flow

1. Resolve the canonical writable repository and apply compatible pending migrations through the shared mutation-command path.
2. Parse clear/value states into the typed patch and reject a patch with no E2-S3 or E2-S4 mutable member.
3. Begin a SQLite write transaction and load the exact task; return `TASK_NOT_FOUND` when absent.
4. Resolve the actor, normalize and validate every supplied structured value, then reject archived state with `TASK_ARCHIVED`.
5. Compare the normalized estimate and ordered collections with the persisted aggregate.
6. If every supplied value is identical, return full detail without issuing task or child-row writes and preserve `updatedAt`/`updatedBy`.
7. Otherwise update the supplied changed estimate, replace only supplied changed collections, and write exactly one `updated_at`/`updated_by` pair.
8. Reload the E2-S2 full-task aggregate in the same transaction, commit, and render it.

Any parse, validation, lookup, constraint, or child insertion failure leaves the task row, every child collection, and update metadata unchanged.

## Output and exit contract

Success uses the E2-S2 normalized full-task data and shared envelope; human output uses the shared detail renderer where practical. The result includes the persisted estimate, tags, `externalUrls`, and `codeReferences` in stored order plus all unchanged task fields.

| Condition | Code/category | Exit |
|---|---|---:|
| No mutable field supplied | `NO_UPDATE_FIELDS` | 2 |
| Invalid estimate | `INVALID_ESTIMATE` | 2 |
| Empty or duplicate tag; duplicate URL or normalized code reference | `DUPLICATE_TASK_CONTEXT` | 2 |
| Malformed or empty external URL | `INVALID_EXTERNAL_URL` | 2 |
| Malformed or empty code reference | `INVALID_CODE_REFERENCE` | 2 |
| Conflicting clear/value options | `CONFLICTING_ARGUMENTS` | 2 |
| Task is archived | `TASK_ARCHIVED` | 2 |
| Task missing | `TASK_NOT_FOUND` | 3 |
| Agent missing | `AGENT_NOT_FOUND` | 3 |

Exit `4` remains reserved for claim conflicts. Shared repository/configuration/database/permission/unexpected errors retain their existing codes and exits.

## Test plan

### Core tests

- Cover omitted, replacement, and clear states for estimate and each collection, including estimate zero.
- Cover finite/negative/NaN/infinite estimate boundaries and every inherited structured validator boundary.
- Verify tag trimming, exact case-sensitive duplicate behavior, URL preservation, normalized code-reference duplicates, nonexistent paths, and line-range validation.
- Prove supplied ordering and contiguous ordinals survive replacement and detail reload.
- Prove omitted and identical fields issue no writes, preserve metadata, and do not reorder collections.
- Combine fields in effective and no-op patches; force validation and insertion failures to verify whole-aggregate rollback.
- Verify user and registered-agent attribution, unknown actors, missing and archived tasks, immutable identity/creation metadata, and unchanged unrelated fields.
- Exercise competing updates through repositories sharing one canonical store and verify each returned aggregate matches its committed transaction.

### CLI integration tests

- Assert parsing for every value/clear option, repeated values, and every same-field conflict.
- Exercise single and combined replacements, clears, omission, effective changes, and valid no-ops as both actor types.
- Snapshot full-detail human and JSON success output and compare with `task view`.
- Assert the documented stable error categories, exact exits, and no mutation for every failure path.
- Verify that the E2-S4 command layout can accept E2-S3 scalar options later without changing structured semantics.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test a temporary initialized repository with fully populated tasks. Replace and clear each structured field from the main and a linked worktree, inspect SQLite values/ordinals and metadata, exercise invalid and no-op patches, and compare returned detail with a subsequent `task view`.

## Definition of done

- Every E2-S4 functional and non-functional acceptance criterion passes.
- Omitted, replacement, and clear states are unambiguous at CLI and core boundaries.
- Validation and collection replacement are atomic and preserve deterministic order.
- Effective changes update actor metadata once; valid identical patches write nothing.
- Archived, missing, unknown-actor, conflicting, duplicate, and malformed inputs leave the complete aggregate unchanged with documented exits.
- Success output matches E2-S2 full detail and the implementation leaves one extensible update path for E2-S3 scalar fields.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [PRD task model, FR-3, validation rules, and E2-S4](../PRD.md)
- [E2-S1 create-task contract](../epics/E2-S1-create-a-task.md)
- [E2-S1 implementation task](E2-S1-T1-implement-task-creation.md)
- [E2-S2 task-detail contract](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S2 implementation task](E2-S2-T1-implement-task-detail-and-listing.md)
- [E2-S3 scalar-update contract](../epics/E2-S3-update-task-content.md)
- [E2-S3 implementation task](E2-S3-T1-implement-task-content-updates.md)
