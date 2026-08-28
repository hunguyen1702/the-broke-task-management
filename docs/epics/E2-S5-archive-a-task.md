---
id: E2-S5
kind: epic
planning_status: done
implementation_status: blocked
depends_on:
  - E2-S1
  - E4-S3
  - E5-S3
---

# E2-S5: Archive a task

## Outcome

Users and registered agents can remove obsolete work from active planning with a durable reason, without silently interrupting another agent or deleting task context.

## User story

As a user or agent, I want to archive abandoned or obsolete work so that it leaves active planning without blocking downstream tasks.

## Command

```text
tbtm task archive <id> --reason <markdown> [--agent <uuid>] [--force] [--yes] [--json]
```

## Product decisions

### Archive transition and reason

- Any active task may be archived regardless of type, status, hierarchy position, or dependency relationships, subject only to the active-claim guard.
- `--reason` is required for an active-to-archived transition. It is outer-trimmed, must remain non-empty, and otherwise preserves its Markdown content.
- Archive atomically sets `archived = true`, stores `archiveReason`, records one UTC `updatedAt` and responsible `updatedBy`, and releases a permitted active claim.
- Status, task content, stable identity, creation metadata, hierarchy, and dependencies remain unchanged. Effective completion and downstream availability are derived from the committed archive state rather than maintained as a separate cache.
- `archiveReason` is `null` for active tasks and non-empty for archived tasks. It appears in the shared full-task aggregate returned by create, view, and archive, and is omitted from the compact list projection. E2-S6 clears it on successful unarchive; a later archive requires a new reason.
- Comments remain independent supplemental context and do not replace the canonical archive reason.

### Active-claim guard

- An unclaimed task may be archived by the logical `user` or a registered agent.
- An agent may archive a task it currently claims; the same transaction releases its claim.
- A task claimed by another agent rejects normal archive as `TASK_CLAIMED`, including claimant ID, display name, and `claimedAt` in human and JSON errors.
- Only the logical `user` may override a foreign claim with `--force`. An agent using `--force` receives `PERMISSION_DENIED`.
- For a foreign claim, interactive human execution with `--force` shows the observed claimant and reason and asks for confirmation. `--yes` skips the prompt; non-interactive and JSON execution require `--force --yes`.
- `--yes` without `--force` is invalid. Declining an interactive confirmation prints `Archive cancelled.` in human mode, exits `0`, and performs no mutation; it is a cancellation outcome, not archive success. JSON mode never prompts and therefore has no declined-confirmation result.

### Confirmation and concurrency

- A force prompt binds approval to the observed claimant ID and `claimedAt`.
- The write transaction re-reads the claim. If the same claim remains, it may be released; if it disappeared, archive proceeds normally.
- If another foreign claim replaced the observed claim, the operation returns `TASK_CLAIMED` with the current claimant and makes no change, requiring a new explicit confirmation.
- Task lookup, actor resolution, archive-state handling, claim comparison/release, archive mutation, metadata, and returned aggregate use one SQLite write transaction.

### Idempotency and output

- Archiving an already archived task with the same normalized reason is a valid no-op. It returns full detail without changing `updatedAt`, `updatedBy`, the reason, or any row.
- Archiving an already archived task with a different normalized reason returns `TASK_ARCHIVED`; archive is not an implicit reason-editing command.
- Archive success returns the E2-S2 normalized full-task detail after mutation, with `archived: true`, the stored `archiveReason`, `claim: null`, and unchanged relationship summaries. E2-S1 create is retrofitted to return `archiveReason: null`, and E2-S2 view returns `null` or the stored reason; compact list output remains unchanged.

### Errors and exits

- Missing or parser-invalid required reason is a validation failure; a blank-after-trim reason returns `INVALID_ARCHIVE_REASON`, exit `2`.
- `--yes` without `--force`, or missing `--yes` for non-interactive/JSON force confirmation, returns `CONFLICTING_ARGUMENTS` or `CONFIRMATION_REQUIRED` respectively, exit `2`.
- A different reason for an already archived task returns `TASK_ARCHIVED`, exit `2`.
- A missing task or unknown agent UUID returns `TASK_NOT_FOUND` or `AGENT_NOT_FOUND`, exit `3`.
- A foreign active claim returns `TASK_CLAIMED`, exit `4`. An agent attempting force returns `PERMISSION_DENIED`, exit `5`.
- Shared repository, configuration, database, permission, and unexpected error categories remain unchanged.

## Functional acceptance criteria

1. A user or registered agent can archive an active unclaimed task with a required non-empty Markdown reason.
2. An owning agent can archive and atomically release its own claim; a foreign claim blocks normal archive with complete claimant details.
3. Only the logical `user` can force archive a foreign-claimed task through the documented confirmation flow.
4. Force confirmation cannot release a replacement claim that was not shown to the user.
5. Successful archive atomically stores archive state, reason, actor metadata, and claim release; every failure leaves all of them unchanged.
6. Archive makes the task effectively completed and immediately changes downstream availability derived from current state.
7. Status, content, identity, creation metadata, hierarchy, and dependencies remain unchanged and archived tasks remain queryable in explicit archive and graph views.
8. Same-reason repeated archive is a metadata-preserving no-op; a different reason is rejected.
9. Human and JSON success and failure output follow the documented detail, claimant, error, and exit contracts.
10. Create, view, and archive share one full-task `archiveReason` field while compact lists remain unchanged; declined confirmation is reported separately from archive success.

## Non-functional acceptance criteria

1. Claim observation, force authorization, archive mutation, and returned aggregate remain transactionally consistent under concurrent local processes and linked worktrees.
2. The operation performs no network access and feels immediate for a local personal repository.
3. Parameterized SQLite operations and typed core inputs keep lifecycle and concurrency rules outside CLI rendering.
4. Timestamps use RFC 3339 UTC, JSON fields use camelCase, and output ordering remains deterministic.
5. Confirmation is safe in TTY, non-interactive, and JSON execution without allowing an implicit foreign-claim override.

## Verification

- Archive unclaimed and own-claimed tasks as user and registered agents, then inspect archive reason, actor metadata, claim removal, unchanged task fields, and downstream availability.
- Exercise foreign claims with no force, agent force, interactive accept/decline, `--yes`, non-interactive execution, and JSON execution.
- Replace or remove the observed claim between prompt and transaction and verify the compare-before-release behavior.
- Verify blank reasons, missing task/agent, already-archived same/different reasons, exact codes, exits, output envelopes, and no-mutation guarantees.
- Query archived list/detail and relationship views to prove the task and its edges remain visible.
- Run formatting, lint, and workspace tests through `mise` after implementation.

## Out of scope

- Unarchive impact analysis, confirmation, and mutation; E2-S6.
- Editing an archive reason while a task remains archived or retaining lifecycle-event history across unarchive.
- Adding, listing, or deleting comments; E6.
- Creating or editing hierarchy/dependency relationships; E4-S1 and E4-S2.
- General claim, unclaim, force-unclaim, and acquisition commands; E5.
- Permanent task deletion, background notifications, inboxes, daemons, or claim expiry.

## Implementation task

See [E2-S5-T1: Implement safe task archival](../tasks/E2-S5-T1-implement-safe-task-archival.md).
