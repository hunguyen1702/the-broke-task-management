---
id: E5-S5-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1-T1
  - E2-S3-T1
  - E5-S1-T1
  - E5-S3-T1
  - E2-S5-T1
  - E2-S6-T1
implements:
  - E5-S5
---

# E5-S5-T1: Harden claim/status independence

## Epic

[E5-S5: Preserve claim/status independence](../epics/E5-S5-preserve-claim-status-independence.md)

## Objective

Consolidate and regression-test the invariant that claim and workflow status change only through their explicit lifecycle operations, with archive as the sole automatic claim-release exception in scope.

## Readiness

Planning is complete. E3-S1-T1 provides stable status completion semantics; E2-S3-T1 implements status mutation; E5-S1-T1 and E5-S3-T1 implement authoritative claim and owner release; E2-S5-T1 and E2-S6-T1 implement archive and unarchive. All implementation dependencies are done, so this task is ready.

## Deliverables

- A cross-command core regression matrix for claim, status update, owner unclaim, archive, and unarchive invariants.
- CLI integration coverage proving unchanged human/JSON contracts and actor behavior on claimed tasks.
- Transaction, concurrency, and linked-worktree coverage for independent state transitions.
- Minimal production fixes or shared-helper refactors only if the new tests expose an invariant violation.
- Updated E5-S5 planning and implementation status documentation after verification.

No migration, new CLI surface, output projection, error code, or cached lifecycle state is expected.

## Proposed structure

Prefer extending the tests beside the authoritative operations and process coverage already established by their owning tasks:

```text
crates/tbtm-core/src/task.rs
crates/tbtm-cli/tests/
```

Focused test modules or files may be introduced if that keeps the transition matrix readable. Follow the implementation-time repository layout; do not move CLI behavior into core or duplicate existing task/claim loaders.

## Technical choices

- Treat the existing `tasks.status_id` relation and `task_claims` row as independent authoritative persistence. Add no synchronization trigger or coupled state field.
- Reuse E2-S3's atomic update and no-op behavior, E5-S1/E5-S3 claim primitives, E2-S5 archive transaction, E2-S6 unarchive behavior, and the shared full-task loader.
- Compare exact retained claims by task ID, agent UUID, and `claimed_at`; verify both direct database state and serialized full-task `claim` output.
- For effective status updates, assert E2-S3 updates status, `updated_at`, and `updated_by` while leaving the claim row byte-for-byte/logically unchanged. For identical-status no-ops, assert neither task nor claim rows change.
- Do not add a claim-owner check to status update. User, owner agent, and foreign registered agent remain valid actors for active-task updates.
- Keep archive's permitted claim deletion inside its existing transaction. Do not generalize the exception into a status-completion hook.
- Production refactoring is justified only where it makes the invariant explicit or fixes demonstrated coupling; preserve public inputs, outputs, errors, and exits.

## Implementation flow

### 1. Establish reusable invariant assertions

Add focused test helpers or fixtures that capture:

- task status ID/code, archive state, `updated_at`, and `updated_by`;
- optional claim task ID, claimant UUID, and `claimed_at`;
- hydrated full-task status and claim projection.

Helpers should compare only the state relevant to an operation while still detecting unintended task-row or claim-row writes.

### 2. Cover claim and status transitions

For claim creation, start with available tasks in `to_do` and `in_progress`, claim them, and assert status plus task mutation metadata are unchanged.

For already-claimed active tasks, exercise:

1. `to_do` or `in_progress` to `done`;
2. `done` back to each incomplete default status;
3. an identical-status valid no-op.

Run effective updates through the logical user, current claim owner, and a different registered agent. Every path retains exact claimant UUID and `claimedAt`; effective updates retain E2-S3 attribution semantics, while the no-op changes no task or claim state. A completed claimed task remains unavailable because completion and claim are independent simultaneous facts.

### 3. Cover explicit release and archive exception

- Owner-unclaim claimed tasks in incomplete and completed statuses. Assert only the claim disappears and status plus task metadata remain unchanged.
- Archive own-claimed incomplete and completed tasks. Assert status remains unchanged while archive state, reason, actor metadata, and claim release commit atomically.
- Force-archive an approved foreign claim as the logical user and assert the same invariant.
- Exercise foreign-claim rejection, interactive cancellation, and observed-claim removal/replacement handling. Every non-success outcome retains the exact claim and status.

E5-S4 force-unclaim has its own implementation task. Do not block this task on E5-S4 or duplicate its result wrapper; its tests must continue to enforce claim-only mutation when it is implemented.

### 4. Cover unarchive boundaries

- Begin with a task whose claim was released by archive, then unarchive it and assert no claim is recreated. Its preserved status alone determines status completion after restoration.
- Invoke unarchive on an already-active claimed task and assert the E2-S6 no-op preserves exact claim, status, and task mutation metadata.
- Where unarchive has downstream impact, preserve E2-S6 confirmation and impact behavior without adding claim mutation to the target or downstream tasks.

### 5. Verify concurrency and public behavior

Coordinate status update with supported claim, owner-unclaim, and archive operations through separate connections/processes and linked worktrees. Accept the serial order produced by the existing immediate transactions, but require each committed result to satisfy the invariant: status operations do not mutate claims, claim operations do not mutate status, and successful archive commits its documented combined lifecycle change atomically.

Exercise representative CLI paths in human and JSON modes. Reuse existing full-task/error envelopes and exact exit mappings; E5-S5 adds no renderer or error variant.

## Output and error contract

There is no E5-S5-specific output or error. Each exercised command retains its owning story's contract:

- claim and status-update success return the shared hydrated full task;
- owner-unclaim returns the shared full task with `claim: null`;
- archive and unarchive retain their established results and confirmation behavior;
- validation, claim conflict, permission, cancellation, and operational outcomes retain existing codes, details, and exits.

Tests must assert these outputs describe the same committed status and claim state observed in SQLite. Do not introduce a combined lifecycle response solely for this story.

## Test plan

### Core tests

- Claim available `to_do` and `in_progress` tasks; assert unchanged status and task metadata.
- Change claimed tasks from both incomplete statuses to `done` and from `done` back to both incomplete statuses; assert exact claim retention and correct update attribution.
- Repeat identical status on a claimed task; assert no task or claim write.
- Run status changes as user, owner, and foreign registered agent; assert no claim authorization branch.
- Owner-unclaim incomplete and completed tasks; assert claim-only mutation.
- Archive own and approved foreign claims across incomplete/completed status, including rollback injection after claim deletion.
- Verify rejected, cancelled, disappeared, and replacement-claim archive paths preserve required state.
- Unarchive after archive and no-op unarchive an already-active claimed task.

### CLI, concurrency, and regression tests

- Snapshot representative human and JSON outputs for every owning command without adding fields or messages.
- Inspect SQLite and subsequent `task view`, `task available`, and `task blockers` results where relevant to distinguish simultaneous completed/claimed facts from derived availability.
- Race status update against claim, owner-unclaim, and archive through separate processes and linked worktrees; assert valid serial outcomes and no partial state.
- Retain existing malformed input, missing actor/task/status, archived-update, ownership, confirmation, busy-timeout, and rollback behavior.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test one claimed task through incomplete-to-completed-to-incomplete transitions, owner release, re-claim, archive, and unarchive while inspecting `task view` after each step.

## Definition of done

- Every E5-S5 functional and non-functional acceptance criterion passes.
- Claim creation never changes status or task mutation metadata.
- Status changes and valid status no-ops never create, delete, transfer, or refresh a claim, regardless of invoking valid actor.
- Completed tasks retain exact active claims until an explicit release or successful archive.
- Owner-unclaim changes only claim state; archive remains the tested atomic automatic-release exception; unarchive never restores an archived claim.
- Existing command outputs, errors, exits, transactions, and core/CLI boundaries remain stable.
- Core database, CLI, rollback, concurrency, and linked-worktree regressions pass.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E5-S5 epic](../epics/E5-S5-preserve-claim-status-independence.md)
- [PRD claim model](../PRD.md#79-claim)
- [PRD effective completion](../PRD.md#76-effective-completion)
- [PRD available-task definition](../PRD.md#81-available-task-definition)
- [PRD E5-S5 story](../PRD.md#story-e5-s5-preserve-claimstatus-independence)
- [PRD MVP acceptance criteria](../PRD.md#13-mvp-acceptance-criteria)
- [E3-S1 default status contract](../epics/E3-S1-use-default-statuses.md)
- [E2-S3 status-update contract](../epics/E2-S3-update-task-content.md)
- [E5-S1 claim contract](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [E5-S3 owner-unclaim contract](../epics/E5-S3-unclaim-owned-work.md)
- [E2-S5 archive contract](../epics/E2-S5-archive-a-task.md)
- [E2-S6 unarchive contract](../epics/E2-S6-unarchive-a-task-safely.md)
- [E5-S4 force-unclaim contract](../epics/E5-S4-force-unclaim-stale-work.md)
