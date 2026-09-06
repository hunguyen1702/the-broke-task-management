---
id: E3-S4-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E3-S2-T1
  - E2-S1-T1
---

# E3-S4-T1: Implement safe custom status deletion

## Parent story

[E3-S4: Delete an unused custom status](../epics/E3-S4-delete-an-unused-custom-status.md)

## Objective

Implement an atomic logical-user command that deletes only an unused custom status, compacts board ordering, and cannot orphan active or archived task data.

## Readiness

Planning is complete. E3-S2-T1 supplies authoritative status management, immutable codes, default/custom identity, ordering invariants, the status CLI group, and typed error patterns. E2-S1-T1 supplies task persistence and the `tasks.status_id` foreign-key relationship required for usage protection. Both implementations are done, so this task is ready. Existing repository operations already use canonical shared-worktree resolution. E3-S3 completion changes are independent of deletion eligibility and are not an implementation dependency.

## Deliverables

- Core custom-status deletion operation with task-usage protection, typed errors, immediate transaction, and atomic order compaction.
- `tbtm status delete <code> [--json]` with stable human/JSON output and exit mapping.
- Focused core and CLI coverage for success, rejection, rollback, concurrency, linked-worktree, migration, and regression behavior.

## Proposed structure

```text
crates/tbtm-core/src/status.rs
crates/tbtm-core/src/lib.rs
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/status.rs
```

Keep core behavior testable without invoking a process. Exact internal factoring may follow the repository structure present during implementation. No schema migration is expected because existing task foreign keys and status fields are sufficient.

## Technical choices

- Reuse E3-S2 `Status`, exact-code lookup, canonical repository resolution, compatible write-path migrations, busy timeout, status renderer conventions, and shared envelopes.
- Add a core delete operation that returns the full `Status` snapshot loaded before deletion.
- Begin an immediate transaction before source lookup. Reject a default status before counting usage, then count every row in `tasks` with the source UUID without filtering `archived`.
- Represent `STATUS_IN_USE` as a typed conflict error with exit `4` and camelCase details `{code, taskCount}`. Reuse `STATUS_NOT_FOUND` with role `source`. Preserve the `STATUS_DEFAULT_IMMUTABLE` code, `{code}` details, and exit `5`, but make its human message command-neutral (`default status is immutable: <code>`) so it is truthful for both E3-S2 rename and E3-S4 delete.
- Delete by stable UUID, compact only subsequent `display_order` values, and preserve the relative order of every remaining status. Use the existing transaction-safe ordering technique or an equivalent update that cannot leave transient uniqueness conflicts.
- Rely on the task-to-status foreign key as defense in depth while retaining the explicit usage count for a stable product error.
- Do not add confirmation, actor fields, history, force behavior, replacement assignment, or completion-specific logic.

## Implementation flow

### 1. Extend core errors and deletion behavior

Add the typed in-use error and its stable code, exit category, message, and details. Add a public deletion entry point that resolves the canonical repository read-write and applies compatible pending migrations.

Update the existing default-immutable display text and regression expectations to the command-neutral wording without changing its machine-readable contract.

Inside one immediate transaction:

1. Find the source by exact code or return `STATUS_NOT_FOUND` with role `source`.
2. Reject `is_default = true` with `STATUS_DEFAULT_IMMUTABLE`.
3. Count all active and archived task rows referencing the status UUID; reject a positive count with `STATUS_IN_USE`.
4. Capture the hydrated source snapshot, delete exactly its row, and compact later positions.
5. Commit and return the captured snapshot.

Any lookup, validation, count, delete, reorder, hydration, or commit failure rolls back the complete mutation.

### 2. Add the CLI command

Add `Delete` to the existing top-level `status` subcommands with positional `code` and optional `--json`. Do not accept `--agent`, `--yes`, or interactive input.

Render the returned `Status` directly as shared-envelope JSON data. Human output identifies the deleted code and name without implying that its old position remains occupied.

### 3. Preserve concurrency and data integrity

Use the established immediate-transaction mutation helper so supported writers serialize the usage check with task creation and task status changes. Confirm the database foreign key prevents a dangling task reference even if an unexpected path bypasses product validation.

The expected race outcomes are:

- A task assignment committed first is included in `taskCount` and deletion fails.
- Deletion committed first removes the status, so a later assignment by code cannot resolve it and fails without creating or changing a task.
- Lock contention beyond the busy timeout retains the shared operational error and changes nothing.

## Output and error contract

Success JSON `data` is the deleted pre-delete `Status` object, including its former zero-based `displayOrder`. Human success identifies its code and name. The resulting ordered list is not part of the response.

| Code | Exit | Details |
|---|---:|---|
| `STATUS_NOT_FOUND` | 3 | `{code, role: "source"}` |
| `STATUS_DEFAULT_IMMUTABLE` | 5 | `{code}` |
| `STATUS_IN_USE` | 4 | `{code, taskCount}` |

Clap missing-argument behavior remains exit `2`. Product validation precedence is missing source, immutable default, then in-use conflict. Shared repository and operational failures retain existing envelopes and exits.

## Test plan

### Core and persistence

- Delete unused custom statuses at the first, middle, and last custom/default-relative positions; assert returned snapshots and contiguous remaining order.
- Prove deletion changes no task, claim, hierarchy, dependency, structured-context, archive, or comment row.
- Reject each default status before usage handling and preserve all data.
- Reject status use by one active task, one archived task, and a mixture; verify exact total `taskCount` and no mutation.
- Reject an unknown exact code with source-role details.
- Inject delete and compaction failures and verify transaction rollback leaves the source and every position unchanged.
- Verify no migration is introduced and the mutation applies any existing compatible pending migration before deletion.

### CLI, concurrency, and regression

- Snapshot help plus exact human and JSON success/error shapes and exit codes; verify no actor or confirmation options exist.
- Race delete against task creation and task status updates from the main and a linked worktree; assert one coherent serialized outcome and no dangling reference.
- Exercise busy-timeout exhaustion and shared operational error mapping without partial mutation.
- Regression-test status list/create/rename/move, default/custom identity, task create/update/view/list, and E3-S3 completion behavior when present.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test deletion in an isolated repository from its main and linked worktrees, including an archived-task usage rejection and one concurrent assignment race.

## Acceptance scenario impact

`add`: E3-S4 introduces a public status-deletion workflow with likely user-facing default-status and in-use rejection paths. A separate acceptance-scenario workflow should cover those common cases after implementation.

## Definition of done

- Every functional and non-functional E3-S4 criterion passes.
- Only unused custom statuses can be deleted; active and archived usage is counted and reported exactly.
- Successful deletion returns the pre-delete snapshot and atomically restores contiguous ordering without changing other repository data.
- Concurrency cannot create a dangling task reference or bypass the documented in-use result.
- Core/CLI separation, migration policy, help, output, typed details, and exits match the contract.
- Acceptance impact is recorded only; no scenario is created, edited, reviewed, approved, or executed.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E3-S4 epic](../epics/E3-S4-delete-an-unused-custom-status.md)
- [PRD status model](../PRD.md#75-status)
- [PRD E3-S4 story](../PRD.md#story-e3-s4-delete-an-unused-custom-status)
- [E3-S2 status-management contract](../epics/E3-S2-create-and-organize-custom-statuses.md)
- [E3-S2 implementation task](E3-S2-T1-implement-custom-status-creation-and-ordering.md)
- [E2-S1 task persistence contract](../epics/E2-S1-create-a-task.md)
- [E2-S1 implementation task](E2-S1-T1-implement-task-creation.md)
- [SQLite foreign keys](https://www.sqlite.org/foreignkeys.html)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
