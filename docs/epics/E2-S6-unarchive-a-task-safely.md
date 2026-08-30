---
id: E2-S6
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E2-S5
  - E4-S4
---

# E2-S6: Unarchive a task safely

## Outcome

The logical user can return archived work to active planning with a clear warning when restoring incomplete work adds dependency blockers to direct downstream tasks, without changing any claim.

## User story

As a user, I want to restore an archived task so that work can return to active planning with clear downstream impact.

## Command

```text
tbtm task unarchive <id> [--yes] [--json]
```

## Product decisions

### Transition and authorization

- Only the logical `user` may unarchive. The command has no `--agent` option and successful mutation records `updatedBy: user`.
- Unarchive changes `archived` to `false`, clears `archiveReason` to `null`, and records one UTC `updatedAt`. It preserves status, content, identity, creation metadata, hierarchy, dependencies, and all downstream claims.
- An archived-to-active target remains unclaimed because archive already released its claim. Unarchive never creates, releases, transfers, or otherwise changes a claim; an already-active no-op therefore preserves any existing claim.
- Effective completion after unarchive comes only from current status. A completed status produces no downstream impact; an incomplete status can restore an unresolved dependency.
- Availability and blocking are derived from committed state; unarchive writes no availability cache.

### Downstream impact

- Impact includes only active, incomplete tasks that depend directly on the target. Recursive descendants do not automatically gain a blocker because a direct dependent's effective-completion state does not change.
- Impacted tasks are partitioned, with claim taking precedence, into:
  - `claimed`: the downstream task has an active claim, whether or not another dependency already blocks it.
  - `otherwiseAvailable`: it has no claim and satisfies the availability predicate before restoration.
  - `alreadyBlockedElsewhere`: it has no claim and already has another unresolved upstream dependency.
- Completed and archived downstream tasks are excluded. All three arrays are ordered by task ID.
- Each item contains `taskId`, `title`, a nullable shared claim summary, and `otherUnresolvedUpstreamTaskIds`. Other blocker IDs exclude the target and are ordered by task ID.
- Impact is empty when the target status is completed or no qualifying direct downstream task exists.

### Warning and confirmation

- Empty preflight impact requires no prompt. Non-empty impact must be confirmed.
- Interactive human execution shows all three impact groups and prompts unless `--yes` is supplied. Declining prints `Unarchive cancelled.`, exits `0`, and performs no mutation.
- `--yes` is explicit advance confirmation and is valid with empty or non-empty impact. JSON and non-interactive execution never prompt; when impact exists they require `--yes`.
- The write transaction recalculates impact. If latest impact is non-empty and the invocation has neither an accepted prompt nor `--yes`, it returns `CONFIRMATION_REQUIRED` with latest impact and makes no change. This covers an empty-to-non-empty race without a snapshot token.
- Once any non-empty impact has been confirmed, the MVP accepts that its details may change before the transaction. It does not require repeated confirmation. Success reports actual transactional impact.
- Downstream claims are warnings only: confirmation never authorizes deleting or modifying them.

### Transaction, idempotency, and output

- Rechecking archive state and status, calculating impact, enforcing confirmation, clearing archive state/reason, updating metadata, and loading the result occur in one SQLite write transaction.
- Unarchiving an already-active task is a successful no-op. It does not prompt, returns empty impact, writes no rows, and preserves update metadata plus the complete task aggregate, including any existing claim.
- An archived-to-active transition returns `{task, impact}`. `task` is E2-S2 full detail with `archived: false`, `archiveReason: null`, and `claim: null`; `impact` is the latest transactional three-group payload. An already-active no-op uses the same wrapper but returns unchanged full detail and empty impact.
- Human success renders the task and concise impact summary. JSON stdout contains only the shared envelope; prompts and warning prose never contaminate it.

### Errors and exits

- Latest non-empty impact without confirmation returns `CONFIRMATION_REQUIRED`, exit `2`, with complete deterministic impact details.
- Missing task returns `TASK_NOT_FOUND`, exit `3`.
- No claim-conflict or agent override exists because unarchive accepts no agent actor and mutates no claims.
- Shared repository, configuration, database, permission, and unexpected categories remain unchanged.

## Functional acceptance criteria

1. Only the logical user can unarchive an archived task.
2. Successful unarchive atomically clears archive flag and reason, records user metadata, and preserves content, relationships, status, identity, creation metadata, and claims.
3. An archived-to-active target remains unclaimed and derives effective completion from current status; an already-active no-op preserves any existing claim.
4. Active, incomplete direct downstream tasks are deterministically classified into the three groups; completed, archived, and recursive-only tasks are excluded.
5. Non-empty impact is shown in human mode or returned in machine-readable confirmation details before an unconfirmed mutation can proceed.
6. TTY confirmation, `--yes`, JSON, non-interactive, cancellation, and empty-to-non-empty race behavior cause no partial mutation.
7. Once impact is confirmed, changed details may be accepted and actual transactional impact is returned without changing downstream claims.
8. An already-active target is a metadata-preserving no-op with empty impact.
9. After commit, availability immediately reflects current status and the dependency graph.
10. Human and JSON results, errors, ordering, codes, and exits are deterministic.

## Non-functional acceptance criteria

1. Validation, impact, confirmation enforcement, mutation, metadata, and returned data are transactionally consistent under concurrent local processes and linked worktrees.
2. The operation performs no network access and feels immediate for a local repository.
3. Typed core operations reuse authoritative dependency, blocking, availability, claim, and full-task models rather than duplicating them in CLI code.
4. SQLite access is parameterized, timestamps use RFC 3339 UTC, and JSON fields use camelCase.
5. Confirmation is safe in TTY, non-interactive, and JSON modes while intentionally avoiding an MVP snapshot-token protocol.

## Verification

- Unarchive completed and incomplete tasks with no, available, claimed, and already-blocked direct dependents.
- Include direct and recursive-only graphs and prove only qualifying direct dependents appear.
- Exercise TTY accept/decline, `--yes`, JSON, non-interactive input, and an empty-to-non-empty race.
- Change impact after a confirmed warning and verify success reports actual impact while preserving every downstream claim.
- Verify reason clearing, user metadata, unchanged fields/relationships, target claim absence, availability, stable ordering, codes, exits, and active-task no-op behavior.
- Run formatting, lint, and workspace tests through `mise` after implementation.

## Out of scope

- Agent-authored unarchive, automatic re-claim, downstream claim mutation, interruption, or notification.
- Impact snapshot tokens, `IMPACT_CHANGED`, repeated confirmation for changed details, or lifecycle-event history.
- Editing status, content, hierarchy, dependencies, archive reason, or comments during unarchive.
- Recursive relationship-map presentation; E4-S5 owns maps.
- Visual Studio Code integration; E8 consumes this core contract.

## Implementation task

See [E2-S6-T1: Implement safe task unarchive](../tasks/E2-S6-T1-implement-safe-task-unarchive.md).
