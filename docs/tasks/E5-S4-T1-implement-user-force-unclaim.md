---
id: E5-S4-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E5-S1-T1
  - E5-S3-T1
  - E4-S3-T1
  - E2-S2-T1
  - E2-S5-T1
implements:
  - E5-S4
---

# E5-S4-T1: Implement user force-unclaim

## Epic

[E5-S4: Force-unclaim stale work](../epics/E5-S4-force-unclaim-stale-work.md)

## Objective

Extend `tbtm task unclaim` with an explicit logical-user force path that safely confirms and removes the exact observed active claim, preserves all task state, and returns the released claim plus authoritative post-release availability.

## Readiness

Planning is complete. E5-S1-T1 and E5-S3-T1 have implemented the authoritative active-claim relation, claim hydration, immediate-transaction conventions, typed claim errors, and the existing owner-unclaim command. E4-S3-T1 provides the authoritative availability predicate and caller-owned selector behavior. E2-S5-T1 provides the established force-confirmation pattern. The task is ready.

## Deliverables

- Core force-unclaim input/result models, observed-claim comparison, exact deletion, post-release availability projection, transaction handling, and typed errors.
- An extended `tbtm task unclaim` CLI supporting mutually exclusive owner and force paths, interactive confirmation, `--yes`, human rendering, JSON output, and stable exit mapping.
- Core, CLI, confirmation, output, rollback, concurrency, linked-worktree, metadata-preservation, and availability regression tests.
- Documentation-compatible behavior that preserves E5-S3 owner-unclaim unchanged.

## Proposed structure

Extend the established claim and task implementations without moving CLI policy into core:

```text
crates/tbtm-core/src/task.rs          # or established task/claim modules
crates/tbtm-cli/src/main.rs           # or established task command modules
crates/tbtm-cli/tests/task_unclaim.rs
```

Exact file boundaries should follow the code present at implementation time. No migration is expected: E5-S4 deletes from the E5-S1 active-claim relation and persists no availability or approval state.

## Technical choices

- Reuse E5-S1/E5-S3 claim models, canonical shared database resolver, busy timeout, full-task loader, and `TransactionBehavior::Immediate`.
- Keep the existing owner-unclaim core path and output contract stable. Add a distinct force-unclaim input/result path rather than weakening owner validation with a generic bypass flag.
- Represent an interactively observed claim token as exact agent UUID plus `claimedAt`. Pass it to core only after approval; do not authorize deletion by task ID alone.
- Define `ClaimChanged` carrying task ID plus the current claim's agent UUID/display name and original timestamp.
- Reuse the authoritative E5-S1 availability reason precedence and E4-S3 transaction-scoped dependency evaluation after deletion. Persist no derived availability state.
- Capture `releasedClaim`, hydrate the full task with `claim: null`, and calculate availability before commit so the response describes one database state.
- Delete by exact task ID, agent ID, and claim timestamp where supported, and check affected rows defensively.
- Do not update the task row or mutation attribution.

## Implementation flow

### 1. Extend CLI parsing without changing owner unclaim

Support:

```text
tbtm task unclaim <task-id> --agent <uuid> [--json]
tbtm task unclaim <task-id> --force [--yes] [--json]
```

- Require exactly one execution path: `--agent` for E5-S3 owner release or `--force` for E5-S4 logical-user release.
- Reject `--agent --force` and `--yes` without `--force` as `CONFLICTING_ARGUMENTS`, exit `2`.
- Preserve the existing missing-`--agent` owner-path syntax behavior unless `--force` selects the user path.
- In JSON or non-TTY force mode, require `--yes`; otherwise return `CONFIRMATION_REQUIRED`, exit `2`, before mutation.

### 2. Observe and confirm an interactive force release

For interactive `--force` without `--yes`:

1. Resolve the canonical repository and read the specified task plus current claim for warning output.
2. Return `TASK_NOT_FOUND` or `CLAIM_NOT_FOUND` if current state cannot produce a force-unclaim prompt.
3. Display task ID, claimant UUID/display name, and `claimedAt`.
4. If the user declines, print `Unclaim cancelled.`, exit `0`, and return before opening the mutation transaction.
5. If approved, pass the observed `{agentId, claimedAt}` token to the core mutation.

For `--yes`, skip the preliminary prompt read. The claim read in the mutation transaction is the authorized and reported claim for that explicit invocation.

### 3. Force-release under one immediate transaction

1. Resolve the writable canonical repository and apply compatible pending migrations.
2. Start an immediate SQLite transaction.
3. Load the task or return `TASK_NOT_FOUND`.
4. Load the active claim or return `CLAIM_NOT_FOUND`.
5. When an observed token is supplied, compare both agent ID and `claimedAt`. If the claim differs, return `CLAIM_CHANGED` with the current claim details without deleting it.
6. Copy the exact current claim into the result's `releasedClaim` value.
7. Delete only that exact claim and require one affected row.
8. Hydrate the E2-S2 full-task aggregate and assert `claim: null`.
9. Evaluate authoritative availability after deletion: archive, completed status, then unresolved dependencies. Sort unresolved direct upstream IDs ascending.
10. Commit before rendering and return the transactionally consistent wrapper.

If an observed claim disappears, step 4 returns `CLAIM_NOT_FOUND`. A replacement claim returns `CLAIM_CHANGED`; it is never deleted. Busy-timeout exhaustion remains an operational database error. Deletion, hydration, or availability-evaluation failure rolls back and preserves the original claim.

### 4. Define result and availability contracts

Return a typed force-unclaim result equivalent to:

```json
{
  "task": {"claim": null},
  "releasedClaim": {
    "agent": {"id": "uuid", "displayName": "agent-name"},
    "claimedAt": "RFC3339 UTC"
  },
  "availability": {
    "available": true,
    "reason": null
  }
}
```

Unavailable projections use exactly `archived`, `completed`, or `dependencies_blocked`, in that precedence. Only `dependencies_blocked` adds sorted `unresolvedUpstreamIds`; do not serialize the field for other outcomes.

Keep this wrapper specific to force-unclaim. E5-S3 owner-unclaim continues returning the bare full-task aggregate and does not gain an availability projection.

### 5. Map errors, exits, and rendering

Map force-path outcomes:

| Exit | Outcome |
|---:|---|
| 0 | Successful force-unclaim or interactive cancellation |
| 1 | Repository, database, busy-timeout, or unexpected operational failure |
| 2 | Syntax, `CONFLICTING_ARGUMENTS`, or `CONFIRMATION_REQUIRED` |
| 3 | `TASK_NOT_FOUND` or `CLAIM_NOT_FOUND` |
| 4 | `CLAIM_CHANGED` |

`CLAIM_CHANGED` details are exactly `{taskId, agent: {id, displayName}, claimedAt}` for the current replacement claim. `CLAIM_NOT_FOUND` details remain `{taskId}`. All errors use the shared envelope and leave state unchanged.

Interactive confirmation uses the exact warning and prompt shape:

```text
Task: <task-id>
Current claim: <display-name> (<agent-id>) at <claimed-at>
Force-unclaim this claim? [y/N]
```

Human success uses these exact lines:

```text
Force-unclaimed task: <task-id>
Released claim: <display-name> (<agent-id>) at <claimed-at>
Available: yes
```

The final line is replaced as follows for unavailable outcomes:

```text
Available: no (archived)
Available: no (completed)
Available: no (dependencies blocked: <upstream-id-1>, <upstream-id-2>)
```

Dependency IDs are sorted by task ID ascending and joined with comma plus one space. JSON returns the result wrapper under the shared success envelope. Cancellation prints only `Unclaim cancelled.` and has no JSON variant.

## Test plan

### Core tests

- Force-release a current claim with no observed token and with a matching observed `{agentId, claimedAt}` token.
- Reject a missing claim as `CLAIM_NOT_FOUND` and a replacement agent or timestamp as `CLAIM_CHANGED` with exact current details.
- Assert deletion affects exactly one claim and returns the deleted owner/time plus a full task with `claim: null`.
- Verify otherwise-available, archived, completed, and dependency-blocked post-release projections, reason precedence, field omission, and sorted unresolved IDs.
- Prove status, archive fields, content, structured context, hierarchy, dependencies, `updatedAt`, and `updatedBy` remain unchanged.
- Inject deletion, post-delete hydration, and availability evaluation failures and prove transaction rollback preserves the claim.
- Exercise affected-row mismatch and busy-timeout exhaustion as operational failures rather than false success or fabricated claim races.

### CLI and integration tests

- Preserve all existing E5-S3 `--agent` parsing, human/JSON output, errors, and ownership tests.
- Exercise `--agent --force`, `--yes` without `--force`, missing force confirmation in JSON/non-TTY mode, and exact exits/envelopes.
- Snapshot interactive warning content, accept, decline, `--yes`, human success, JSON wrapper, and cancellation output.
- Remove and replace the observed claim between prompt and mutation; assert `CLAIM_NOT_FOUND` or `CLAIM_CHANGED` and no deletion of replacement state.
- Coordinate force-unclaim with supported claim/unclaim operations across separate processes and linked worktrees; verify current-state outcomes and canonical shared state.
- Verify task view/list and availability/claim commands immediately reflect claim removal without any other projection changing.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test interactive and `--yes --json` force-unclaim from the main worktree and a linked worktree against the same canonical claim.

## Definition of done

- Every functional and non-functional criterion in E5-S4 passes.
- The existing owner-unclaim path remains backward-compatible while the logical-user force path is explicit and mutually exclusive.
- Confirmation identifies and binds to the observed claim; disappeared and replacement races return stable, no-write outcomes.
- Success deletes only the active claim and returns consistent full-task, released-claim, and post-release availability data.
- Task state and mutation metadata remain unchanged, and no derived availability state is persisted.
- Confirmation, output, error, rollback, process concurrency, and linked-worktree tests pass.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E5-S4 epic](../epics/E5-S4-force-unclaim-stale-work.md)
- [PRD claim model](../PRD.md#79-claim)
- [PRD available-task definition](../PRD.md#81-available-task-definition)
- [PRD claim management](../PRD.md#fr-8-claim-management)
- [PRD CLI experience](../PRD.md#fr-11-cli-experience)
- [PRD validation rules](../PRD.md#11-validation-and-edge-case-rules)
- [PRD E5-S4 story](../PRD.md#story-e5-s4-force-unclaim-stale-work)
- [E5-S1 authoritative claim contract](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [E5-S3 owner-unclaim contract](../epics/E5-S3-unclaim-owned-work.md)
- [E2-S5 force-confirmation contract](../epics/E2-S5-archive-a-task.md)
- [E2-S2 full-task shape](../epics/E2-S2-view-and-list-tasks.md)
- [E4-S3 authoritative availability query](../epics/E4-S3-query-available-tasks.md)
- [Agent approval future improvement](../improvements/agent-approval-for-user-only-commands.md)
