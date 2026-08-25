---
id: E3-S1-T1
kind: implementation_task
planning_status: done
implementation_status: done
implementation_commit: 4d8ebb2
depends_on:
  - E1-S1-T1
---

# E3-S1-T1: Implement default status codes

## Parent story

[E3-S1: Use default statuses](../epics/E3-S1-use-default-statuses.md)

## Objective

Add stable machine codes to the baseline status schema, seed the canonical defaults during fresh initialization, and provide a transaction-friendly core lookup for downstream task operations.

## Readiness

Planning is complete and E1-S1-T1 is implemented. This task is ready for implementation and unblocks E2-S1-T1.

## Deliverables

- Updated baseline `statuses` schema with a required, unique, validated code.
- Updated fresh-init inserts for the three canonical defaults.
- Minimal core status model and exact lookup by code.
- Unit and initialization integration tests for schema, seed values, lookup, atomicity, and unchanged CLI output.
- Updated E1 initialization tests and fixtures that describe the fresh schema.

## Technical choices

- Modify `crates/tbtm-core/migrations/0001_repository_foundation.sql`; do not add a status-code migration.
- Keep `LATEST_MIGRATION = 2` and config `schemaVersion: 1`.
- Add a `code TEXT NOT NULL UNIQUE` column with a SQLite `CHECK` equivalent to `[a-z][a-z0-9_]*`.
- Keep UUID `id` as the relationship key and retain the existing constraints on name, completion, display order, and default flag.
- Seed statuses only in the existing initialization transaction, after the baseline schema exists.
- Model code as readable but not mutable. No core API in this story updates a status or its code, and E3-S2 must keep code out of rename/update inputs.
- Allow lookup to borrow a caller-provided SQLite connection/transaction and perform no commit or nested transaction.
- Add no CLI subcommand, output field, or exit-code mapping.

## Proposed structure

Follow the existing core layout; a small module is preferred if it keeps initialization focused:

```text
crates/tbtm-core/
├── migrations/
│   └── 0001_repository_foundation.sql
└── src/
    ├── lib.rs
    └── status.rs
```

The exact module boundary may follow the implemented crate style. CLI code stays out of core, and status lookup must remain independently testable without launching a process.

## Implementation flow

### 1. Extend the baseline schema

Add `code` to `statuses` with database constraints for:

- Non-null content.
- First character in lowercase ASCII `a` through `z`.
- Remaining characters limited to lowercase ASCII letters, digits, or underscore.
- Repository-wide uniqueness.

Use a SQLite expression that enforces the whole grammar, including rejecting empty strings and invalid characters anywhere in the value. Preserve the existing UUID primary key, unique name, unique display order, boolean checks, and default flag.

This is intentionally a fresh-schema change. Do not implement detection, repair, backfill, or upgrade of a database created by the previous development-only baseline.

### 2. Seed canonical defaults

Update initialization to insert:

```text
to_do       | Todo        | false | 0 | true
in_progress | In progress | false | 1 | true
done        | Done        | true  | 2 | true
```

Generate one UUID per row using the existing UUID representation. Keep migration ledger entries, repository metadata, and all default inserts inside the existing database transaction. Any insert or constraint failure aborts the full transaction.

### 3. Add the core status model and lookup

Represent at least:

- `id`
- `code`
- `name`
- `completed`
- `display_order`
- `is_default`

Provide exact, case-sensitive lookup by code. The lookup accepts a database handle borrowed from its caller, performs only the query, and distinguishes a found status from no match without owning transaction boundaries. This lets E2-S1 map no match to `STATUS_NOT_FOUND` while keeping its status lookup and task insertion in one transaction.

Do not add list, create, rename, reorder, completion-change, or delete APIs in this task.

### 4. Preserve external behavior

`tbtm init` continues to return its existing human and JSON output. E3-S1 is observable through the initialized database and downstream status consumers, not through a new command or response field.

## Test plan

### Schema and unit tests

- Accept canonical and representative valid custom-style codes.
- Reject null, empty, uppercase-leading, digit-leading, whitespace, hyphenated, or otherwise invalid codes.
- Reject duplicate codes even when other status fields differ.
- Map every persisted field into the core status model.
- Lookup all three exact codes and return no match for unknown or differently cased values.
- Run lookup within a caller-owned transaction and prove the caller retains commit/rollback control.

### Initialization integration tests

- Fresh init creates exactly three statuses in display order with the canonical field values.
- Every status ID parses as UUID and remains the row identity returned by lookup.
- Initialization from root, nested directories, and linked worktrees writes the same canonical store behavior established by E1-S5.
- A forced seed failure rolls back schema transaction contents and does not produce a valid partial initialization.
- Existing init human/JSON snapshots and exit codes do not change.
- Force init creates a new fresh store with the canonical statuses after preserving the old workspace under the existing E1 rules.

Legacy-schema repositories, status-code backfill, and migration-version `3` tests are explicitly excluded.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also initialize a temporary repository and query `statuses` directly to compare its rows with the canonical table in the parent story.

## Definition of done

- The parent story's functional and non-functional acceptance criteria pass.
- Baseline schema and fresh-init seeds include stable status codes without adding a migration version.
- Database constraints reject every invalid code class in the test plan.
- Core lookup is reusable inside E2-S1's transaction and does not own commit behavior.
- Initialization remains atomic and its CLI contract remains unchanged.
- Documentation no longer promises legacy backfill or preservation of the previous development-only `0001` schema.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E3-S1 epic](../epics/E3-S1-use-default-statuses.md)
- [PRD status model](../PRD.md#75-status)
- [PRD status management requirement](../PRD.md#fr-6-status-management)
- [E1-S1 repository initialization](../epics/E1-S1-initialize-repository.md)
- [E1-S1 implementation task](E1-S1-T1-implement-repository-initialization.md)
- [E2-S1 task creation](../epics/E2-S1-create-a-task.md)
