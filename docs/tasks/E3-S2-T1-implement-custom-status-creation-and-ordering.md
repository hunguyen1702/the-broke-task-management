---
id: E3-S2-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E3-S1-T1
---

# E3-S2-T1: Implement custom status creation and ordering

## Parent story

[E3-S2: Create and organize custom statuses](../epics/E3-S2-create-and-organize-custom-statuses.md)

## Objective

Implement logical-user commands and reusable core operations to list, create, rename, and reorder repository statuses while preserving immutable codes, unique names, contiguous order, and atomic shared-worktree behavior.

## Readiness

Planning is complete. E3-S1-T1 supplies the authoritative status schema, model, code grammar, default rows, and transaction-friendly lookup. Its implementation is done, so this task is ready.

## Deliverables

- Migration `0009` enforcing ASCII case-insensitive status-name uniqueness.
- Core list/create/rename/move operations, validation, typed errors, transactional ordering, and no-op handling.
- Top-level `tbtm status` command group with stable human/JSON output, help, and exit mapping.
- Focused core, CLI, migration, rollback, concurrency, linked-worktree, and regression tests.

## Proposed structure

```text
crates/tbtm-core/migrations/0009_status_name_nocase.sql
crates/tbtm-core/src/status.rs
crates/tbtm-core/src/lib.rs
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/status.rs
```

Keep core behavior testable without invoking a process. Exact internal factoring may follow the repository structure present during implementation.

## Technical choices

- Reuse `statuses`, `Status`, canonical repository resolution, E3-S1 exact-code lookup, compatible write-path migrations, read-only no-migration behavior, busy timeout, and shared envelopes.
- Migration `0009` creates a unique index over `name COLLATE NOCASE`. SQLite `NOCASE` defines the chosen ASCII case-insensitive behavior; non-ASCII characters otherwise remain exact.
- Normalize names by trimming before persistence, require a non-empty result, and preserve the accepted spelling.
- Keep codes exact and immutable. Reuse one E3-S1 grammar validator so invalid input becomes a typed error rather than a generic constraint failure.
- Use immediate transactions for every mutation. Maintain zero-based contiguous `display_order` values and avoid transient unique collisions with a transaction-safe two-phase or equivalent update.
- Detect semantic no-ops before issuing writes. Custom same-name rename and already-positioned move return hydrated current state without mutation.
- Do not add actor fields, history, completion mutation, or deletion.

## Implementation flow

### 1. Add the compatible uniqueness migration

Create migration `0009` with a unique `NOCASE` index on status names and register it in the migration sequence. Fresh and upgraded repositories receive the invariant. If preexisting manually-created rows collide, index creation fails and the shared migration transaction rolls back without advancing the migration version.

### 2. Extend core status contracts

Add reusable operations for deterministic listing, initially-incomplete custom creation with optional placement, custom rename by immutable code, and moving any status before or after another.

Define typed errors with the exact codes, exits, and camelCase details from the parent story. Keep transaction ownership in mutation entry points and reuse hydration inside their transactions.

### 3. Add the top-level CLI group

Expose the four documented `tbtm status` commands. Use Clap conflicts and required groups for placement syntax. None accepts `--agent`; status configuration is logical-user authority.

### 4. Implement consistent reads and atomic writes

- List via one read-only snapshot ordered by `display_order, code`, without applying pending migrations.
- Mutations resolve the canonical store read-write, apply compatible migrations, and begin an immediate transaction.
- Validate and normalize inputs, resolve source/target rows, enforce precedence, calculate the final ordered UUID sequence, safely update changed rows, hydrate results, and commit.
- Create uses a new UUID, `completed = false`, and `is_default = false`; omitted placement appends it.
- Rename rejects a default before same-name no-op handling and changes only `name`.
- Move accepts default/custom sources, rejects self-targeting, and does not write when the requested relation already holds.
- Every failed validation, constraint, write, hydration, or commit rolls back.

### 5. Render output and errors

- List `data` is the ordered status array.
- Create and rename `data` is the status object directly.
- Move `data` is exactly `{status, statuses}`, with the full resulting collection in `statuses`.
- Human output exposes code, name, completion state, default/custom identity, and zero-based position.
- Stable error details are: invalid/conflicting code `{code}`, invalid/conflicting name `{name}`, missing status `{code, role}`, immutable default `{code}`, and self-position `{code, targetCode}`.

Validation precedence is CLI syntax, scalar code/name validation, source lookup where applicable, target lookup, default rename protection, conflicts, then mutation. Create checks code conflict before name conflict. Move rejects syntactic self-targeting before lookup ambiguity.

## Test plan

### Migration, core, and persistence

- Upgrade migration 8 to 9 successfully and verify fresh repositories apply the index.
- Seed ASCII-case-conflicting names before migration; prove migration failure is atomic and version 9 is absent.
- List complete default/custom models deterministically.
- Create at end/before/after across edge and middle positions; verify UUID, flags, and contiguous order.
- Reject every invalid-code class, duplicate code, empty normalized name, ASCII case variant, and unknown target without writes.
- Rename custom statuses and verify field preservation plus same-name no-write behavior; reject every default and conflicting name.
- Move default/custom statuses across all positions; verify relative order, contiguous positions, no-op, self-target, and missing source/target behavior.
- Inject multi-row reorder failure and prove rollback.

### CLI, concurrency, and regression

- Snapshot help and exact human/JSON shapes for every command and error.
- Race duplicate-code creates, ASCII-case-conflicting names, and overlapping moves from main/linked worktrees; verify coherent winners and order.
- Verify list does not migrate while each mutation applies compatible pending migrations.
- Regression-test E3-S1 seeds and downstream task create/update/list/available/claim/blocker behavior with custom incomplete statuses.
- Verify no status command accepts `--agent` and no mutation changes task or claim rows.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test all four status commands in an isolated repository from its main and linked worktrees, including one migration or mutation rollback path.

## Acceptance scenario impact

`add`: E3-S2 introduces public status listing, creation, rename, and board-order workflows. A separate acceptance-scenario workflow should cover common successes and likely validation failures after implementation.

## Definition of done

- Every functional and non-functional E3-S2 criterion passes.
- Migration `0009` enforces the documented name uniqueness and rolls back cleanly on incompatible existing data.
- Codes remain immutable; default rename is rejected; create/move leave contiguous deterministic order under failure and concurrency.
- Valid no-ops do not write, and every documented failure preserves the prior collection.
- Core/CLI separation, migration policy, help, output, typed details, and exits match the contract.
- Acceptance impact is recorded only; no scenario is created, edited, reviewed, approved, or executed.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E3-S2 epic](../epics/E3-S2-create-and-organize-custom-statuses.md)
- [PRD status model](../PRD.md#75-status)
- [PRD status management requirement](../PRD.md#fr-6-status-management)
- [PRD E3-S2 story](../PRD.md#story-e3-s2-create-and-organize-custom-statuses)
- [E3-S1 default-status contract](../epics/E3-S1-use-default-statuses.md)
- [E3-S1 implementation task](E3-S1-T1-implement-default-status-codes.md)
- [E4-S3 availability contract](../epics/E4-S3-query-available-tasks.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [SQLite NOCASE collation](https://www.sqlite.org/datatype3.html#collating_sequences)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
