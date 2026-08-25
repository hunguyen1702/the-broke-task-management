---
id: E3-S1
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E1-S1
---

# E3-S1: Use default statuses

## Outcome

Every freshly initialized repository has three UUID-backed default statuses with stable machine codes, canonical display values, and explicit completion semantics that downstream task operations can reference safely.

## User story

As a user, I want todo, in-progress, and done statuses so that a new repository is immediately usable.

## Product decisions

### Canonical defaults

Fresh initialization creates exactly these default rows in board order:

| Code | Display name | Completed | Display order | Default |
|---|---|---:|---:|---:|
| `to_do` | Todo | false | 0 | true |
| `in_progress` | In progress | false | 1 | true |
| `done` | Done | true | 2 | true |

Each row receives its own UUID. The UUID is the database relationship identity; code is the stable machine selector; name is the user-facing label.

### Status-code contract

- Codes are repository-unique, non-null, and match `[a-z][a-z0-9_]*`.
- A code is immutable after creation. This story exposes no operation that changes it, and later rename/update paths must not accept code changes.
- SQLite enforces presence, format, and uniqueness. Core code owns the immutable update contract; direct unsupported database edits are outside the product interface.
- Core exposes a minimal status model and lookup by code that can run on a connection or transaction supplied by its caller. It must not open a nested transaction, so E2-S1 can resolve a status and insert a task within one snapshot.

### Fresh-schema scope

- No released or user repository requires schema compatibility yet. E3-S1 therefore updates baseline migration `0001` and the fresh-initialization seed path directly.
- No follow-up status-code migration, legacy backfill, existing-row repair, or old-schema upgrade test is required.
- The internal latest migration remains version `2`; repository config remains `schemaVersion: 1`.
- Schema creation, migration ledger entries, repository metadata, and default-status inserts remain one atomic initialization transaction.

### User interface

E3-S1 adds no CLI command and does not change init human or JSON output. Status listing and management belong to E3-S2 and E7. The persisted result is verified directly through SQLite and consumed by later core operations.

## Functional acceptance criteria

1. A fresh `tbtm init` creates exactly the three canonical default statuses with the documented codes, names, completion values, board order, and default flags.
2. Every default status has a valid UUID, and code is distinct from UUID and display name.
3. SQLite rejects null, malformed, or duplicate status codes.
4. Core can retrieve each default status by exact machine code and returns no match for an unknown code.
5. Status lookup can participate in a transaction owned by a caller without creating a separate transaction or database snapshot.
6. Initialization commits schema, repository metadata, and all three statuses together or leaves no valid initialized database.
7. E3-S1 introduces no new CLI command or output shape.

## Non-functional acceptance criteria

1. Initialization and lookup are synchronous, local, network-free, and feel immediate for a personal repository.
2. Schema constraints protect stable code identity and deterministic lookup under concurrent local readers.
3. Status UUIDs use the repository's existing UUID representation and SQLite booleans remain constrained to `0` or `1`.
4. Tests provide practical Linux, macOS, and Windows coverage through the existing workspace strategy.

## Verification

- Initialize repositories from canonical root, nested directory, and linked worktree, then query the canonical SQLite store.
- Assert the exact row count, code/name/completion/order/default mapping, UUID validity, and deterministic ordering.
- Exercise status lookup inside a caller-owned transaction for all defaults and an unknown code.
- Attempt direct invalid inserts to prove null, format, and uniqueness constraints.
- Inject a seed failure and prove the initialization transaction does not leave partial metadata or statuses.
- Confirm init human/JSON output contracts are unchanged.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Upgrading or backfilling repositories created with an earlier development schema.
- Listing statuses through the CLI.
- Creating, renaming, or reordering custom statuses; E3-S2.
- Changing completion semantics; E3-S3.
- Deleting statuses; E3-S4.
- Creating tasks or selecting their initial status; E2-S1 consumes this story's contract.

## Implementation task

See [E3-S1-T1: Implement default status codes](../tasks/E3-S1-T1-implement-default-status-codes.md).
