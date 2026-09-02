---
id: E5-S5
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1
  - E5-S1
  - E5-S3
  - E2-S5
---

# E5-S5: Preserve claim/status independence

## Outcome

Agents and users can change workflow status and claim ownership through their explicit commands without either lifecycle silently driving the other, while archive remains the documented claim-release exception.

## User story

As an agent, I want claim ownership and workflow status to remain independent so that I control each transition explicitly.

## Existing commands

E5-S5 adds no command. It consolidates and regression-tests behavior exposed by:

```text
tbtm task claim <task-id> --agent <uuid> [--json]
tbtm task update <task-id> --status <code> [--agent <uuid>] [--json]
tbtm task unclaim <task-id> --agent <uuid> [--json]
tbtm task archive <task-id> --reason <markdown> [--agent <uuid>] [--force] [--yes] [--json]
tbtm task unarchive <task-id> [--yes] [--json]
```

No schema, migration, output projection, error code, or persisted lifecycle history is introduced.

## Product decisions

### Independent transitions

- Claiming an available task creates only the active claim. It does not change status, `updatedAt`, or `updatedBy`.
- Updating an active task's status changes status and normal E2-S3 mutation metadata only. It does not create, release, transfer, refresh, or otherwise rewrite a claim.
- A status transition from incomplete to completed may retain an active claim. A transition from completed to incomplete also retains the same claim even though the claim makes the task unavailable.
- A valid identical-status update is an E2-S3 no-op and preserves both task mutation metadata and the exact claim.
- Exact claim retention means the claimant UUID and original `claimedAt` remain unchanged in persistence and hydrated full-task output.
- Owner unclaim removes only the claim regardless of whether the current status is incomplete or completed. It does not change status or task mutation metadata.

### Actor and ownership boundary

- The logical user, the current claim owner, and a different registered agent may each update the status of an active claimed task under E2-S3's actor rules.
- Claim ownership is a coordination signal, not an authorization lock for status updates. A foreign claim does not produce a claim conflict for `task update --status`.
- Actor resolution and update attribution remain independent of claim ownership: an effective status update records the invoking actor while preserving the claim owner and time.

### Archive exception and unarchive behavior

- Archive is the intentional exception: every successful archive atomically releases any permitted active claim while leaving the stored status unchanged.
- An own-claimed archive and a logical-user force archive of an approved foreign claim follow E2-S5. A rejected, raced, or cancelled archive preserves the exact claim and status.
- Unarchive never recreates the claim released by archive. An archived-to-active transition is therefore unclaimed and derives completion from its preserved status.
- Calling unarchive on an already-active task remains E2-S6's metadata-preserving no-op and must preserve any active claim that already exists.
- Force-unclaim remains E5-S4 scope. Its contract must continue to remove only the claim, but E5-S5 does not depend on or duplicate that command's implementation.

### Implementation boundary

- This story hardens the existing claim, status-update, archive, and unarchive paths with one explicit cross-command invariant matrix.
- Reuse the authoritative task, status, claim, availability, and full-task models. Do not add a second lifecycle abstraction or cached derived state.
- Production changes are limited to fixes or small shared-helper refactors required when the regression matrix exposes a violation.
- Core continues to own lifecycle mutations and transaction boundaries; CLI continues to own parsing and rendering.

## Functional acceptance criteria

1. Claiming tasks in each incomplete default status preserves that status and all task mutation metadata.
2. Effective status changes from incomplete to completed and completed to incomplete preserve the exact active claim while applying E2-S3 status and actor metadata behavior.
3. A valid identical-status no-op on a claimed task preserves task metadata, claimant UUID, and `claimedAt` without writing the claim row.
4. The logical user, claim owner, and a foreign registered agent can each update an active claimed task's status without a claim authorization error or claim mutation.
5. Owner unclaim succeeds for incomplete and completed tasks, removes only the claim, and leaves status and task mutation metadata unchanged.
6. Successful archives of own-claimed tasks and approved foreign-claimed tasks atomically release the claim while preserving status; every rejected or cancelled archive preserves the exact claim and status.
7. An archived-to-active unarchive remains unclaimed, while an already-active unarchive no-op preserves any existing claim.
8. Existing human and JSON success/error contracts remain unchanged and hydrated task results consistently show the independent committed status and claim state.

## Non-functional acceptance criteria

1. Status update, claim mutation, archive, and unarchive each expose one transactionally consistent committed state under concurrent local processes and linked worktrees.
2. No operation adds cached availability, coupled status/claim columns, or lifecycle side effects outside its documented transaction.
3. Regression coverage verifies persisted SQLite state and public core/CLI results without weakening the core/CLI boundary.
4. Existing camelCase JSON, RFC 3339 UTC timestamps, deterministic output, error codes, and exit behavior remain stable.
5. All behavior remains local and network-free and feels immediate for an ordinary personal repository.

## Verification

- Run the claim/status transition matrix for `to_do`, `in_progress`, and `done`, inspecting task rows, claim rows, actor metadata, and full-task hydration before and after each operation.
- Exercise status updates by the logical user, claim owner, and a foreign registered agent, including effective transitions and identical-status no-ops.
- Unclaim incomplete and completed tasks and verify only the active claim row changes.
- Archive incomplete and completed own-claimed tasks plus approved foreign-claimed tasks; cover rejected, cancelled, removed, and replacement-claim confirmation outcomes.
- Unarchive previously claimed archived tasks and exercise the already-active no-op with a current claim.
- Coordinate status updates with supported claim lifecycle operations across separate processes and linked worktrees; assert serializable current-state outcomes and no partial coupling.
- Snapshot representative human and JSON output and run formatting, lint, and all workspace tests through `mise`.

## Out of scope

- New claim, status, archive, or unarchive commands and output shapes.
- Status-driven automatic claim, unclaim, transfer, refresh, or expiry.
- Claim-based authorization for general task updates.
- Claim lifecycle history, notifications, heartbeats, or background recovery.
- Custom-status creation or completion-semantics editing; E3-S2 and E3-S3.
- Implementing logical-user force-unclaim; E5-S4.

## Implementation task

See [E5-S5-T1: Harden claim/status independence](../tasks/E5-S5-T1-harden-claim-status-independence.md).
