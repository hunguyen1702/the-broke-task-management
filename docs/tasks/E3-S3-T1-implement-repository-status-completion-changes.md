---
id: E3-S3-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E3-S2-T1
  - E4-S3-T1
---

# E3-S3-T1: Implement repository status completion changes

## Parent story

[E3-S3: Change status completion semantics](../epics/E3-S3-change-status-completion-semantics.md)

## Objective

Implement a logical-user-only, repository-scoped status completion mutation with deterministic before/after task impact, explicit confirmation, preserved claims, and immediate reuse by authoritative completion and availability predicates.

## Readiness

Planning is complete. E3-S2-T1 now provides the public status commands, shared status object, mutable custom/default ordering model, and migration `0009`. E4-S3-T1 provides the authoritative availability selector, so this task is ready for implementation.

## Deliverables

- Typed core input, result, and impact models for changing one repository status `completed` value.
- Bounded before/after impact selection reusing authoritative task, status, dependency, claim, blocking, and availability rules.
- One immediate SQLite transaction covering validation, latest impact, confirmation enforcement, mutation, and result hydration.
- `tbtm status set-completed` parsing, TTY preflight/prompting, human rendering, JSON envelopes, typed errors, and exits.
- Core and CLI coverage for both directions, no-ops, impact categories, concurrency, rollback, repository isolation, linked worktrees, and exact public behavior.

## Proposed structure

Extend the authoritative status module introduced by E3-S2 and reuse the implemented task/dependency/availability modules. A likely organization is:

```text
crates/tbtm-core/src/status.rs
crates/tbtm-core/src/task.rs
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/status_set_completed.rs
```

Exact files follow repository layout at implementation time. Core owns repository semantics, impact queries, transaction boundaries, and typed errors. CLI owns Clap syntax, TTY detection, prompting, rendering, shared envelopes, and exit mapping.

## Technical choices

- Mutate the one `statuses.completed` field identified by exact immutable code. Do not update task rows: tasks resolve their status by UUID and inherit the repository definition.
- Permit both default and custom statuses. Preserve UUID, code, name, display order, and `is_default` exactly.
- Reuse E3-S2 `Status` output and lookup behavior. Return `{status, impact}` rather than inventing a task-scoped status representation.
- Model impact with ordered `status_tasks` and `downstream_tasks` serialized as `statusTasks` and `downstreamTasks`.
- A status item contains task ID, title, nullable shared claim summary, and authoritative `available_before`/`available_after` booleans.
- A downstream item adds ordered unresolved direct-upstream IDs before and after. Include each active direct dependent of an affected status task when it is incomplete on either side; exclude it only when archived or completed on both sides.
- Deduplicate downstream tasks and allow the same task in both top-level arrays. Order all task collections and unresolved-upstream collections by task ID.
- Compute both worlds from the same database snapshot by evaluating the current status value and the proposed value logically. Do not temporarily mutate and roll back merely to calculate preflight, and do not persist derived state.
- Use the exact E4-S3 availability conjunction and the PRD section 7.6/7.8 unresolved direct-upstream rule for both worlds. Claims are read for projection and availability only and are never mutated.
- A boolean confirmation authorization reaches core from `--yes` or an accepted TTY prompt; it does not identify an impact snapshot.

## Implementation flow

1. Add `SetCompleted` under the E3-S2 `status` CLI namespace with required status code and `--completed <bool>`, plus shared `--yes` and `--json` conventions.
2. Resolve the canonical writable repository and apply compatible pending migrations, including E3-S2 migration `0009` when necessary.
3. Load a read-only preflight from one snapshot: resolve the source status, detect a same-value no-op, and otherwise calculate before/after impact without writing.
4. For non-empty human TTY impact, render the transition and both groups and prompt unless `--yes` is present. Decline with exact cancellation output and no mutation call. JSON and non-TTY paths never prompt.
5. Begin an immediate transaction, resolve the exact status again, and return `STATUS_NOT_FOUND` for a missing source. If its latest value already equals the requested value, return a no-write result with empty impact.
6. Calculate latest impact inside the transaction. If it is non-empty and confirmation authorization is false, return `CONFIRMATION_REQUIRED` carrying that exact impact and roll back.
7. Update only `statuses.completed`, then hydrate the shared status object and retain actual impact from the same transaction before commit.
8. Render `{status, impact}`. Ensure every failed, declined, or same-value path preserves all status/task/claim/relationship rows and task metadata.

Preflight is advisory. Transactional recalculation is authoritative. After any non-empty impact has been confirmed, changed task IDs, claims, blockers, categories, or availability details do not demand a second confirmation; the result reports the actual transactional impact.

## Impact query details

### Status tasks

- Select every non-archived task whose `status_id` matches the source status UUID.
- For each task, project ID/title/claim and evaluate exact availability with the current and proposed status completion values.
- Archived tasks remain effectively completed in both worlds and are excluded.

### Direct downstream tasks

- Starting from affected active status-task IDs, select distinct active direct dependents.
- Evaluate the dependent's own completed state in both worlds. Retain it when incomplete before or incomplete after; exclude only when completed on both sides.
- This rule intentionally retains a dependent that uses the target status while it crosses completion state, and permits that task to appear in both arrays.
- For every retained dependent, calculate all unresolved direct upstream IDs before and after using `upstream.archived = false AND upstream.status.completed = false`, substituting the proposed value for upstreams using the changed status.
- Calculate availability before and after with the full authoritative predicate, including the dependent's own completion, archive state, claim, and all upstreams.
- Keep claimed and otherwise-blocked dependents even when both availability booleans are false, because their dependency-resolution details change.

Use set-oriented SQL, temporary in-memory representations, or equivalent bounded-query techniques. Avoid per-task query growth and preserve one-snapshot consistency.

## Output and error contract

Success data is:

```json
{
  "status": {
    "id": "UUID",
    "code": "review",
    "name": "Review",
    "completed": true,
    "displayOrder": 2,
    "isDefault": false
  },
  "impact": {
    "statusTasks": [],
    "downstreamTasks": []
  }
}
```

Human output reports repository status code/name, old/new completed state, and both impact groups. Items show claim identity when present, availability before/after, and downstream unresolved upstreams before/after. Exact wording should remain concise and receive process-output coverage. Cancellation is exactly `Status completion change cancelled.` on stdout with exit `0`.

| Condition | Code/category | Details | Exit |
|---|---|---|---:|
| Latest non-empty impact is unconfirmed | `CONFIRMATION_REQUIRED` | Exact impact payload | 2 |
| Source status is absent | `STATUS_NOT_FOUND` | `{code, role: "source"}` | 3 |
| Missing or malformed boolean/arguments | Clap invalid input | Clap diagnostic | 2 |

Lookup precedes no-op handling. Same-value success returns the unchanged shared status plus empty impact without a write or prompt. Shared repository, migration, database, permission, contention, and unexpected errors retain existing envelopes and exits.

## Test plan

### Core behavior

- Change incomplete to completed and completed to incomplete for both default and custom statuses; assert only `completed` changes.
- Verify separate canonical repositories with the same code remain isolated, while a main and linked worktree share the mutation.
- Cover zero, one, and many active tasks using the status; include available, claimed, dependency-blocked, and archived tasks.
- Assert effective completion, unresolved dependencies, `task available`, specified claim, and next claim immediately consume the changed definition without cached state.
- Prove claims and every task field/metadata row remain unchanged.

### Impact

- Verify exact status-task and downstream-task fields and ordering in both directions.
- Include downstream tasks that become available/unavailable, remain blocked elsewhere, retain claims, depend on multiple affected sources, and overlap the status-task set.
- Include downstream tasks completed before only, after only, or both; include archived and recursive-only tasks and verify the documented inclusion rule.
- Exercise duplicate paths to one downstream and assert one ordered item with complete ordered before/after blocker IDs.
- Compare impact booleans with the authoritative available selector and blocker IDs with authoritative unresolved-dependency behavior.

### Confirmation, transactions, and CLI

- Exercise TTY accept/decline, `--yes`, JSON and non-TTY with and without confirmation, and empty impact without a prompt.
- Cover a same-value no-write request, unknown source, malformed boolean, stable details, shared envelopes, exact cancellation, public help, and exits.
- Race preflight impact empty to non-empty and non-empty to empty or changed details. Verify unauthorized latest non-empty impact rolls back, confirmed impact proceeds, and success reports the actual transaction view.
- Force failures before update, after impact selection, and during result hydration to prove atomic rollback and no claim/task mutation.
- Run command/output coverage from both the main and linked worktree and verify one coherent canonical result under concurrent writers.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test two temporary repositories plus a linked worktree. Build affected source tasks, overlapping direct dependents, claims, and alternate blockers; exercise both completion directions in human and JSON modes and compare task availability/blocking before and after.

## Acceptance scenario impact

`add`: E3-S3 introduces the public `status set-completed` workflow, confirmation behavior, and user-visible task/dependency impact. A separate acceptance-scenario workflow should cover its common success and likely user-error paths.

## Definition of done

- Every E3-S3 functional and non-functional criterion passes and E3-S2 behavior remains intact.
- Completion is demonstrably repository-scoped: all referencing tasks in one canonical repository inherit the change, while another repository does not.
- Impact, ordering, confirmation details, success output, errors, cancellation, and exits match the contract.
- Both transition directions, overlap semantics, claims, other blockers, no-op behavior, concurrency, and rollback are covered.
- The mutation updates only the status definition, preserves claims and task metadata, and is transactionally safe across linked worktrees.
- Core/CLI boundaries and authoritative completion, dependency, blocking, and availability reuse are maintained with bounded queries.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [Product requirements](../PRD.md)
- [E3-S3 epic](../epics/E3-S3-change-status-completion-semantics.md)
- [E3-S2 status configuration contract](../epics/E3-S2-create-and-organize-custom-statuses.md)
- [E3-S2 implementation task](E3-S2-T1-implement-custom-status-creation-and-ordering.md)
- [E4-S3 available-task contract](../epics/E4-S3-query-available-tasks.md)
- [E4-S3 implementation task](E4-S3-T1-implement-available-task-querying.md)
- [E2-S6 confirmation precedent](../epics/E2-S6-unarchive-a-task-safely.md)
- [Effective completion](../PRD.md#76-effective-completion)
- [Availability definition](../PRD.md#81-available-task-definition)
