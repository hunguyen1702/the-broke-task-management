---
id: E3-S2
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1
---

# E3-S2: Create and organize custom statuses

## Outcome

Users can extend and arrange the repository workflow while every status retains a stable machine identity and an unambiguous board position.

## User story

As a user, I want custom ordered statuses so that the board matches my workflow.

## Commands

```text
tbtm status list [--json]
tbtm status create --code <code> --name <name> [--before <code> | --after <code>] [--json]
tbtm status rename <code> --name <name> [--json]
tbtm status move <code> (--before <code> | --after <code>) [--json]
```

Status configuration is a logical-user operation. These commands do not accept `--agent`.

## Product decisions

### Stable identity and creation

- Every status keeps its UUID row identity and immutable unique machine code. Custom codes use the exact E3-S1 grammar: a lowercase ASCII letter followed only by lowercase ASCII letters, digits, or underscores.
- Creation requires an explicit code and display name. A custom status is incomplete initially; E3-S3 owns later completion-semantics changes.
- `--before` and `--after` are mutually exclusive. When neither is present, the new status is appended to the end of the board.
- A placement target is resolved by exact immutable code. An unknown target fails without inserting or reordering anything.
- Created statuses have `isDefault: false`; the three E3-S1 statuses retain `isDefault: true`.

### Names and rename

- Leading and trailing whitespace is removed before a name is validated and stored. A name must remain non-empty.
- Names are unique across the repository using ASCII case-insensitive comparison. Non-ASCII characters are allowed and otherwise compared exactly; the accepted spelling is preserved for display.
- Migration `0009` adds a unique `name COLLATE NOCASE` index. Preexisting manually-created names that violate the new invariant make the migration fail atomically through the shared migration error path.
- Only custom statuses can be renamed. A default status is immutable by this command, including a request that repeats its current name.
- Renaming a custom status to its current normalized name is a successful no-op and does not write state.
- Rename never changes UUID, code, completion semantics, default/custom identity, or board position.

### Board ordering

- `displayOrder` is zero-based and contiguous across all statuses.
- Both default and custom statuses may be moved. Moving one status shifts the affected range while preserving every other relative order.
- A move that already describes the current position is a successful no-op without a write.
- Using the source status itself as the `--before` or `--after` target is invalid input rather than a no-op.
- Create placement and move update the complete affected order atomically; failures preserve the prior board.

### Reads, writes, and concurrency

- `status list` is read-only, does not apply pending migrations, and reads one consistent snapshot under the existing repository compatibility rules.
- Create, rename, and move use the canonical shared repository, apply compatible pending migrations, and perform lookup, validation, uniqueness checks, order changes, and result hydration in one immediate SQLite transaction.
- The database rejects exact code duplicates and ASCII case-insensitive name duplicates. Immediate transactions prevent partial or non-contiguous order changes under concurrent supported writers.
- Busy-timeout exhaustion remains a shared operational database error. E3-S2 adds no actor or audit metadata to statuses.

### Output

The shared status object remains:

```json
{
  "id": "UUID",
  "code": "review",
  "name": "Review",
  "completed": false,
  "displayOrder": 2,
  "isDefault": false
}
```

- `status list` returns all statuses ordered by `displayOrder`, then code as a deterministic corruption-safe tie-breaker.
- Create and rename return the resulting status object directly as shared-envelope `data`.
- Move returns `{status, statuses}` as `data`, where `status` is the moved status and `statuses` is the complete resulting ordered collection.
- Human output includes code, display name, completed/incomplete state, default/custom identity, and zero-based position. Mutation output concisely identifies the completed operation.

### Errors and exits

| Code | Exit | Stable details and condition |
|---|---:|---|
| `INVALID_STATUS_CODE` | 2 | `{code}`; a create code violates the E3-S1 grammar. |
| `INVALID_STATUS_NAME` | 2 | `{name}`; a normalized create or rename name is empty. |
| `STATUS_NOT_FOUND` | 3 | `{code, role}`; a source or placement target does not exist. |
| `STATUS_CODE_CONFLICT` | 4 | `{code}`; a status already uses the requested code. |
| `STATUS_NAME_CONFLICT` | 4 | `{name}`; a status already uses the requested name under ASCII case-insensitive comparison. |
| `STATUS_DEFAULT_IMMUTABLE` | 5 | `{code}`; rename targets a default status. |
| `INVALID_STATUS_POSITION` | 2 | `{code, targetCode}`; a move targets its source. |

Clap handles missing required arguments and conflicting `--before`/`--after` syntax as exit `2`. Repository, database, migration, permission, contention, and unexpected failures retain shared envelopes and exit behavior. Every error leaves status content and ordering unchanged.

## Functional acceptance criteria

1. A user can list all default and custom statuses in deterministic board order in human or JSON mode.
2. A user can create an explicitly coded, named, initially incomplete custom status at the end, before a status, or after a status.
3. Creation rejects invalid or duplicate codes, empty normalized names, ASCII case-insensitive duplicate names, and unknown placement targets without mutation.
4. A user can rename a custom status while preserving its UUID, code, completion value, custom identity, and order; repeating its current normalized name is a no-write success.
5. Default-status rename is rejected, and all failed renames preserve the existing status.
6. A user can move a default or custom status before or after another status; the result has contiguous zero-based order and preserves unaffected relative order.
7. Current-position moves succeed without writing, while self-targeted or unknown-source/target moves fail without changing order.
8. Outputs, errors, details, and exits match the documented human and JSON contracts.

## Non-functional acceptance criteria

1. Core owns validation, uniqueness, ordering, transactions, and typed errors; CLI owns parsing, rendering, envelopes, and exit mapping.
2. Mutations and migration `0009` are atomic and safe across concurrent processes and linked worktrees sharing the canonical database.
3. Reads use one snapshot and never migrate; writes follow the existing compatible-migration convention.
4. Status codes remain immutable, JSON remains camelCase, and all operations are local and network-free.
5. Ordinary listing and mutation remain immediate for a personal repository with a small workflow.

## Verification

- Exercise list, append creation, before/after insertion, custom rename, default/custom moves, and valid no-ops in human and JSON modes.
- Reject each documented validation, conflict, missing-target, and default-immutability path while proving no data changed.
- Verify migration success and atomic failure on preexisting ASCII-case-conflicting names.
- Verify contiguous ordering and stable UUID/code/completion/default fields after every mutation.
- Race conflicting names, codes, and moves through the main and a linked worktree; verify one coherent committed order and no partial writes.
- Verify migration policy, exact envelopes, public help, and exit codes.

## Out of scope

- Changing `completed`; E3-S3.
- Deleting any status; E3-S4.
- Renaming default statuses or changing any status code.
- Unicode normalization or Unicode case folding of names.
- Authentication, agent authority, colors, icons, WIP limits, transition rules, bulk mutations, and interactive board editing.
- Acceptance-scenario creation, review, approval, or execution in this workflow.

## Implementation task

See [E3-S2-T1: Implement custom status creation and ordering](../tasks/E3-S2-T1-implement-custom-status-creation-and-ordering.md).
