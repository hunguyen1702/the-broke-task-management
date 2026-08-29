---
id: E2-S6-T1
kind: implementation_task
planning_status: done
implementation_status: blocked
depends_on:
  - E2-S5-T1
  - E4-S4-T1
---

# E2-S6-T1: Implement safe task unarchive

## Parent story

[E2-S6: Unarchive a task safely](../epics/E2-S6-unarchive-a-task-safely.md)

## Objective

Implement user-only task unarchive with deterministic direct-downstream impact reporting, explicit confirmation for impact, atomic archive-reason clearing, preserved claims, and stable human/JSON behavior.

## Deliverables

- A typed direct-impact model backed by authoritative E4 blocking and availability rules.
- A transactional core operation covering lifecycle state, latest impact, confirmation enforcement, mutation, metadata, and aggregate reload.
- `tbtm task unarchive` parsing, TTY confirmation, human rendering, JSON envelopes, stable errors, and exit mapping.
- Core, CLI, graph, transaction, concurrency, idempotency, and regression tests.

## Proposed structure

Extend the authoritative lifecycle, dependency, availability, claim, and task-detail modules present after dependencies are implemented. A likely shape is:

```text
crates/tbtm-core/src/task/unarchive.rs
crates/tbtm-core/src/task/impact.rs
crates/tbtm-cli/src/commands/task_unarchive.rs
```

Exact files follow repository layout at implementation time. Core owns impact classification, confirmation enforcement, transaction boundaries, lifecycle mutation, and typed results. CLI owns preflight presentation, TTY detection, prompting, envelopes, rendering, and exits.

## Technical choices

- Reuse E2-S5 storage and invariants: active tasks have `archiveReason = null`, archived tasks have a non-empty reason, and an archived target has no active claim. An already-active target may legitimately have a claim, which a no-op must preserve.
- Reuse E2-S2 full-task aggregate, E4-S4 unresolved-dependency explanations, E4-S3 availability predicate, and E5 claim summary. Do not create parallel graph, blocker, availability, or claim representations.
- Represent impact as ordered `claimed`, `otherwiseAvailable`, and `alreadyBlockedElsewhere` arrays. Each item carries `taskId`, `title`, nullable claim summary, and ordered `otherUnresolvedUpstreamTaskIds` excluding the target.
- Query only active, incomplete direct dependents. Classify an active claim first; otherwise use pre-unarchive availability to distinguish otherwise-available from already-blocked-elsewhere.
- Treat a completed target status as empty impact because removing archive completion does not make the target incomplete.
- Pass boolean confirmation authorization into core. It is true for `--yes` or an accepted prompt and does not encode a graph snapshot.
- Use one SQLite write transaction to re-read target state/status, calculate latest impact, enforce confirmation, mutate lifecycle fields and metadata, and reload task plus impact.
- Preserve target/downstream claims and all relationships. Availability remains derived and uncached.

## CLI and confirmation contract

Expose:

```text
tbtm task unarchive <id> [--yes] [--json]
```

- Actor is always logical `user`; no `--agent` or force option exists.
- Perform a read-only preflight sufficient to render a warning. Empty impact needs no prompt. Non-empty interactive human impact prompts unless `--yes` is present.
- Declining prints `Unarchive cancelled.`, exits `0`, and does not enter the mutation path.
- JSON and non-TTY execution never prompt. `--yes` supplies confirmation; otherwise non-empty impact returns `CONFIRMATION_REQUIRED` with impact details.
- Core recalculates impact in the write transaction. If it is non-empty and confirmation authorization is false, roll back and return `CONFIRMATION_REQUIRED` with latest impact. This covers a preflight empty-to-non-empty race.
- If authorization is true, proceed even if latest task IDs, categories, blockers, or claims differ from the preflight warning. This bounded stale-warning trade-off avoids snapshot tokens for the MVP.

## Implementation flow

1. Resolve the canonical writable repository and apply compatible pending migrations.
2. Parse mode and `--yes`, then load target state and preflight direct impact through shared read models.
3. Return the active-task no-op without prompting, preserving its full aggregate including any claim. For archived non-empty impact, render the human warning and prompt when required; return cancellation if declined.
4. Begin a SQLite write transaction and load the exact target. Return `TASK_NOT_FOUND` if absent; if now active, return the write-free no-op with empty impact.
5. Load current status completion and calculate current direct impact from the transaction snapshot.
6. If current impact is non-empty and confirmation authorization is false, return `CONFIRMATION_REQUIRED` with current impact and make no change.
7. Otherwise set `archived = false`, clear `archive_reason`, and write one UTC `updated_at` plus logical-user `updated_by`. Do not mutate claims or relationships.
8. Reload the E2-S2 full task and retain current impact in the same transaction, then commit and render `{task, impact}`.

Any lookup, confirmation, graph-query, validation, constraint, or persistence failure leaves lifecycle state, reason, metadata, relationships, and claims unchanged.

## Output and exit contract

Success data is:

```json
{
  "task": {
    "id": "TBTM-task-1234abcd",
    "archived": false,
    "archiveReason": null,
    "claim": null
  },
  "impact": {
    "claimed": [],
    "otherwiseAvailable": [],
    "alreadyBlockedElsewhere": []
  }
}
```

The abbreviated `task` shows the archived-to-active transition, whose target claim is `null`. It stands for complete E2-S2 full detail. An already-active no-op instead returns unchanged full detail, including any existing claim, with empty impact. Each impact item contains `taskId`, `title`, `claim`, and `otherUnresolvedUpstreamTaskIds`; group arrays and blocker IDs use task-ID ordering. Human mode displays equivalent warning and success information. Prompt text never appears in JSON stdout.

| Condition | Code/category | Exit |
|---|---|---:|
| Latest impact requires confirmation | `CONFIRMATION_REQUIRED` | 2 |
| Task missing | `TASK_NOT_FOUND` | 3 |

Accepted unarchive, active-task no-op, and declined interactive confirmation exit `0`. Shared repository/configuration/database/permission/unexpected failures retain established codes and exits. Unarchive introduces no claim-conflict or agent-permission branch.

## Test plan

### Core tests

- Unarchive completed and incomplete archived tasks; verify effective completion, reason, actor metadata, and availability.
- Build direct dependents in every group, including claimed tasks with other blockers, and assert precedence, fields, exclusions, and ordering.
- Add recursive-only downstream tasks and prove exclusion; also exclude completed and archived direct dependents.
- Verify current impact is calculated in the mutation transaction and returned from that snapshot.
- Cover empty preflight becoming non-empty: without authorization return `CONFIRMATION_REQUIRED`; with `--yes` succeed.
- Change IDs, categories, blockers, and claims after confirmed non-empty preflight; proceed with latest impact and change no downstream claim.
- Verify active-task no-op issues no writes, returns empty impact, and preserves metadata plus any existing claim.
- Force late failures and competing lifecycle/dependency/claim changes to prove rollback and consistent results.

### CLI integration tests

- Exercise human TTY with empty impact, confirmation accept/decline, and `--yes`.
- Exercise JSON and non-TTY execution with and without `--yes`, including machine-readable confirmation details and no prompt leakage.
- Snapshot all warning groups, success `{task, impact}`, cancellation text, stable errors, exact exits, and ordering.
- Compare task output with `task view`; inspect SQLite and availability/blocking output after success and rejected paths.
- Verify `--agent` and unsupported force options are parser-invalid and no claim is created, released, or transferred.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test a temporary repository containing an archived incomplete upstream task, direct dependents in all groups, and recursive-only descendants. Exercise TTY and JSON confirmation from main and linked worktrees; inspect lifecycle/metadata/claim state and compare impact with E4 explanations and availability.

## Definition of done

- Every E2-S6 functional and non-functional criterion passes.
- Only logical user can invoke the command; an archived-to-active target remains unclaimed while an active-target no-op preserves any existing claim.
- Impact classification, ordering, confirmation, error details, and success impact are deterministic and share authoritative graph/claim models.
- No unconfirmed non-empty latest impact can commit, including an empty-to-non-empty race.
- A confirmed stale warning may proceed by explicit MVP design, but no downstream claim or relationship mutates.
- Lifecycle change, reason clearing, metadata, impact, and returned task are transactionally consistent; no-op and failure paths write nothing.
- Human/JSON output, cancellation, codes, and exits are stable and tested.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E2-S6 epic](../epics/E2-S6-unarchive-a-task-safely.md)
- [E2-S5 archive contract](../epics/E2-S5-archive-a-task.md)
- [E2-S5 implementation task](E2-S5-T1-implement-safe-task-archival.md)
- [E2-S2 task-detail contract](../epics/E2-S2-view-and-list-tasks.md)
- [E4-S3 available-task story](../PRD.md#story-e4-s3-query-available-tasks)
- [E4-S4 blocking-explanation story](../PRD.md#story-e4-s4-explain-blocking)
- [E5 claim model](../PRD.md#79-claim)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
