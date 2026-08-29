---
id: E5-S3-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E5-S1-T1
---

# E5-S3-T1: Implement owner-controlled task unclaim

## Parent story

[E5-S3: Unclaim owned work](../epics/E5-S3-unclaim-owned-work.md)

## Objective

Implement an agent-only command that atomically removes the invoking registered agent's own active claim, preserves all task state and mutation metadata, and reuses the existing full-task output contract.

## Readiness

Planning is complete. E5-S1-T1 has established the authoritative active-claim schema, claim hydration, and specified-task claim behavior, so this task is ready.

## Deliverables

- Core unclaim input/result behavior, ownership validation, exact deletion, transaction handling, and typed errors.
- `tbtm task unclaim` CLI parsing, concise human rendering, JSON envelope, and exit mapping.
- Core, CLI, concurrency, linked-worktree, output, rollback, and post-release availability regression tests.
- A reusable owner-release primitive suitable for E2-S5 archive integration without weakening E5-S3 ownership rules.

## Proposed structure

Extend the authoritative claim implementation introduced by E5-S1 while preserving the core/CLI boundary:

```text
crates/tbtm-core/src/task.rs          # or task/claim.rs
crates/tbtm-cli/src/main.rs           # or existing task command module
```

Exact file boundaries should follow the E5-S1 implementation present when this task becomes ready. No migration is expected: E5-S3 deletes from the active-claim relation established by E5-S1.

## Technical choices

- Reuse E5-S1's `task_claims` table, canonical shared database resolver, busy timeout, claim models, full-task loader, and `TransactionBehavior::Immediate`.
- Represent the request with specified task ID and required agent UUID. Reuse registered-agent and task lookup errors.
- Add typed `ClaimNotFound` carrying task ID and `ClaimNotOwned` carrying task ID plus E5-S1's claimant identity/display name and original timestamp.
- Delete by both `task_id` and the validated owner `agent_id`, with affected-row checking as a defensive integrity guard. Never issue an unconditional delete after ownership was observed outside the transaction.
- Do not update the task row. Status, archive state, content, relationships, `updated_at`, and actor columns remain unchanged.
- Keep availability derived. E5-S3 writes no availability cache and adds no availability field to the full-task result.
- Expose an internal transaction-scoped owner-release operation that archive can compose inside its broader transaction; do not make archive perform a separate committed unclaim command.

## Implementation flow

### 1. Define core and error contracts

Add a core request for task ID plus required agent UUID. Successful core execution returns the existing hydrated `FullTask` after deletion.

Define stable errors:

- `ClaimNotFound { task_id }` → `CLAIM_NOT_FOUND`, details `{taskId}`;
- `ClaimNotOwned { task_id, agent, claimed_at }` → `CLAIM_NOT_OWNED`, details `{taskId, agent: {id, displayName}, claimedAt}`.

Preserve E1-S3/E5-S1 behavior for malformed UUID, `AGENT_NOT_FOUND`, `TASK_NOT_FOUND`, repository errors, and operational database failures.

### 2. Release the claim under one immediate transaction

1. Parse task ID and required agent UUID at the CLI boundary.
2. Resolve the canonical repository read-write and apply compatible pending migrations through the established write path.
3. Start an immediate write transaction.
4. Resolve the registered agent or return `AGENT_NOT_FOUND`.
5. Load the task or return `TASK_NOT_FOUND`.
6. Load its active claim. Return `CLAIM_NOT_FOUND` if absent.
7. Compare the claim owner with the invoking UUID. Return `CLAIM_NOT_OWNED` with the current claimant if different.
8. Delete the exact task/owner claim and require exactly one affected row.
9. Hydrate the normalized full-task aggregate in the same transaction and assert its claim is absent.
10. Commit, then return the result.

This order makes no-claim and foreign-owner behavior deterministic after agent and task resolution. A task's archived, completed, or dependency-blocked state is irrelevant to owner release and must not be validated as an unclaim precondition.

If immediate lock acquisition times out, return the inherited operational database error. After a waiter acquires the lock, it reads committed current state: a prior removal yields `CLAIM_NOT_FOUND`, while a current foreign owner yields `CLAIM_NOT_OWNED`. Any affected-row mismatch or hydration failure rolls back.

### 3. Preserve derived availability and task metadata

Do not run a separate availability mutation or add output fields. Claim deletion changes the authoritative predicate naturally. Regression tests should prove:

- an active incomplete task with resolved dependencies becomes available after release;
- archived, completed, or dependency-blocked tasks remain unavailable after release for their existing reason;
- a subsequent specified-task claim succeeds only when the E5-S1 predicate is now true;
- task `updated_at`, `updated_by`, status, archive fields, content, hierarchy, and dependencies remain byte-for-byte/logically unchanged.

Until E4-S3 is implemented, verify the predicate through E5-S1 claim attempts or existing core helpers; add direct availability-query regression coverage when E4-S3 becomes available rather than creating a competing query here.

### 4. Add CLI parsing and rendering

Expose:

```text
tbtm task unclaim <task-id> --agent <uuid> [--json]
```

The command is non-interactive and requires `--agent`. Human success confirms only the task ID and released agent display name. JSON returns the shared full-task aggregate directly under the existing success envelope with `claim: null`.

Map outcomes:

| Exit | Outcome |
|---:|---|
| 0 | Successful owner unclaim |
| 1 | Database, busy-timeout, or unexpected operational failure |
| 2 | Malformed or missing arguments |
| 3 | `AGENT_NOT_FOUND`, `TASK_NOT_FOUND`, or `CLAIM_NOT_FOUND` |
| 5 | `CLAIM_NOT_OWNED` |

No result adds an availability projection. Every error uses the shared envelope and leaves state unchanged.

## Test plan

### Core tests

- Unclaim an owned claim and verify exactly one claim deletion plus a returned full task with `claim: null`.
- Verify completed, dependency-blocked, and otherwise-available owned tasks can all be unclaimed.
- Assert status, archive fields, content, structured context, hierarchy, dependencies, timestamps, and actor metadata do not change.
- Exercise unknown agent, missing task, missing claim, and foreign owner with exact validation precedence, typed details, and no writes.
- Inject delete and post-delete hydration failures and prove rollback restores/preserves the original claim.
- Force an affected-row mismatch and verify an operational integrity failure rather than false success.
- Exercise busy-timeout exhaustion separately from current-state ownership outcomes.

### Concurrency and integration tests

- Coordinate two owner unclaim attempts. Assert one success, one `CLAIM_NOT_FOUND`, and no task mutation.
- Coordinate owner unclaim with a competing supported claim lifecycle operation; verify the transaction observes one committed state and never deletes a replacement or foreign claim.
- Repeat relevant operations from linked worktrees and verify they share the canonical claim state.
- After release, claim an otherwise-available task from another registered agent. Also verify completed and dependency-blocked released tasks remain rejected by E5-S1 for their existing reason.
- Snapshot exact human success, JSON full-task success, malformed UUID, missing agent/task/claim, foreign-owner, and operational output and exits.
- Verify task view/list immediately show `claim: null` after commit without changing their other projections or ordering.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test owner release and subsequent acquisition through two registered agents in separate linked worktrees.

## Definition of done

- Every functional and non-functional criterion in E5-S3 passes.
- Only the current registered owner can remove a normal active claim; missing and foreign claims return exact stable errors.
- Claim lookup, owner validation, exact deletion, aggregate hydration, and commit are one immediate transaction.
- Successful release changes no task status or mutation metadata and returns the established full-task shape with `claim: null`.
- Subsequent availability and claim behavior immediately reflects committed claim absence without an availability cache or new output projection.
- Concurrency, rollback, linked-worktree, human/JSON output, and exit-code tests pass.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E5-S3 epic](../epics/E5-S3-unclaim-owned-work.md)
- [PRD claim model](../PRD.md#79-claim)
- [PRD availability definition](../PRD.md#81-available-task-definition)
- [PRD claim management](../PRD.md#fr-8-claim-management)
- [PRD agent work lifecycle](../PRD.md#102-agent-work-lifecycle)
- [PRD E5-S3 story](../PRD.md#story-e5-s3-unclaim-owned-work)
- [E5-S1 authoritative claim contract](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [E5-S1 implementation task](E5-S1-T1-implement-atomic-specified-task-claiming.md)
- [E1-S3 agent identity](../epics/E1-S3-register-an-agent.md)
- [E2-S2 full-task and claim shape](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S5 archive claim release consumer](../epics/E2-S5-archive-a-task.md)
