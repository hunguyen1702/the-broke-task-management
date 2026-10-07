---
id: E5-S4
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E5-S1
---

# E5-S4: Force-unclaim stale work

## Outcome

The logical user can explicitly release a crashed or abandoned agent's active claim, with confirmation bound to the observed claim and an immediate, authoritative availability result, without changing task workflow or mutation metadata.

## User story

As a user, I want to force-release a claim so that a crashed or abandoned agent cannot hold work forever.

## Command

E5-S4 extends the existing E5-S3 command rather than introducing a second claim-release verb:

```text
tbtm task unclaim <task-id> --force [--yes] [--json]
```

The existing owner path remains:

```text
tbtm task unclaim <task-id> --agent <uuid> [--json]
```

`--agent` and `--force` are mutually exclusive. The force path represents the logical `user`; it accepts no agent identity.

## Product decisions

### User intent and confirmation

- Force-unclaim is explicit and available only through `--force`; normal `--agent` unclaim retains E5-S3 ownership validation.
- In an interactive human terminal, `--force` displays the task ID, current claimant UUID/display name, and original `claimedAt`, then requests confirmation unless `--yes` is present.
- `--yes` skips the prompt. JSON and non-interactive force execution require `--force --yes`; otherwise the command returns `CONFIRMATION_REQUIRED`.
- `--yes` without `--force` is invalid. Declining interactive confirmation prints `Unclaim cancelled.`, exits `0`, and performs no mutation. JSON mode never prompts and has no cancellation result.
- The logical-user convention and confirmation are intent guardrails, not authentication. Agent-runtime approval for user-only commands is a future improvement and does not block this story.

### Claim and task lifecycle

- A task without an active claim returns `CLAIM_NOT_FOUND`; force-unclaim is not an idempotent no-op.
- Successful force-unclaim deletes only the active claim row. It does not change task status, archive state, content, relationships, `updatedAt`, or `updatedBy`.
- An archived, completed, or dependency-blocked task may still have a claim released. These states affect the returned availability result, not authorization to release.
- Force-unclaim does not transfer a claim, expire claims automatically, or create a claim-lifecycle history record.

### Confirmation and concurrency

- Interactive approval is bound to the observed `{agentId, claimedAt}` pair, not merely the task ID.
- The write transaction re-reads the current claim. If the observed claim remains, that exact claim may be deleted.
- If the observed claim disappeared before the transaction, the command returns `CLAIM_NOT_FOUND`. There is no other mutation to continue.
- If a different agent or timestamp replaced the observed claim, the command returns `CLAIM_CHANGED` with the current claimant and timestamp, makes no change, and requires a new explicit confirmation.
- For `--yes`, which skips pre-mutation prompting, the claim loaded in the write transaction is the claim authorized and reported by that invocation.
- Claim lookup, comparison, exact deletion, full-task hydration, released-claim capture, availability calculation, and commit use one immediate SQLite transaction. Busy-timeout exhaustion remains an operational database failure.

### Updated availability

- Availability remains derived from authoritative archive, status, claim, and dependency state; no availability flag or blocker cache is persisted.
- After claim deletion, the same transaction evaluates availability using E5-S1 precedence: `archived`, `completed`, then `dependencies_blocked`.
- An otherwise claimable task returns `{available: true, reason: null}`.
- An unavailable task returns `available: false` and one stable reason: `archived`, `completed`, or `dependencies_blocked`.
- `dependencies_blocked` additionally returns every unresolved direct upstream task ID sorted ascending as `unresolvedUpstreamIds`. The other outcomes omit that field.

### Success output

- JSON success wraps three transactionally consistent projections under the shared success envelope:
  - `task`: the E2-S2 normalized full-task aggregate after deletion, with `claim: null` and every other task field unchanged;
  - `releasedClaim`: `{agent: {id, displayName}, claimedAt}` for the claim that was deleted;
  - `availability`: the updated projection described above.
- Interactive confirmation renders `Task: <task-id>`, then `Current claim: <display-name> (<agent-id>) at <claimed-at>`, then prompts with `Force-unclaim this claim? [y/N]`.
- Human success renders `Force-unclaimed task: <task-id>`, then `Released claim: <display-name> (<agent-id>) at <claimed-at>`, then exactly one availability line: `Available: yes`, `Available: no (archived)`, `Available: no (completed)`, or `Available: no (dependencies blocked: <id-1>, <id-2>)`.
- Dependency-blocked IDs are sorted by task ID ascending and joined with comma plus one space.
- Both interactive confirmation and `--yes` success identify the exact released owner and original claim time.

### Errors and exits

Validation precedence for the force path is command syntax and flag conflicts, execution-mode confirmation requirements, task existence, active-claim existence, interactive observation/confirmation when required, then transactional claim recheck and mutation.

| Code | Exit | Condition and stable details |
|---|---:|---|
| `CONFLICTING_ARGUMENTS` | 2 | `--agent` with `--force`, or `--yes` without `--force` |
| `CONFIRMATION_REQUIRED` | 2 | JSON or non-interactive force execution omits `--yes` |
| `TASK_NOT_FOUND` | 3 | The supplied task ID does not exist |
| `CLAIM_NOT_FOUND` | 3 | No active claim exists at observation or the observed claim disappeared before deletion; details are `{taskId}` |
| `CLAIM_CHANGED` | 4 | The observed claim was replaced; details are `{taskId, agent: {id, displayName}, claimedAt}` for the current claim |

Syntax failures exit `2`. Repository, database, permission, busy-timeout, and unexpected failures retain shared behavior. Declined confirmation is a successful cancellation with no JSON branch. Every error or cancellation leaves claim and task state unchanged.

## Functional acceptance criteria

1. The logical user can explicitly force-release a specified active claim through the existing `task unclaim` command without supplying an agent identity.
2. Owner unclaim remains available through E5-S3, and mutually exclusive flags prevent an invocation from mixing owner and force semantics.
3. Interactive execution identifies the observed owner and claim time and requires confirmation; `--yes`, JSON, non-interactive, and cancellation follow the documented behavior.
4. Confirmation can delete only the observed `{agentId, claimedAt}` claim. Removal yields `CLAIM_NOT_FOUND`; replacement yields `CLAIM_CHANGED` with current owner details and no deletion.
5. Successful force-unclaim removes only the claim and preserves status, archive state, content, relationships, timestamps, and actor metadata.
6. JSON success returns transactionally consistent `task`, `releasedClaim`, and updated `availability` projections; human success reports the same essential result.
7. Updated availability uses the authoritative predicate, stable reason precedence, and sorted unresolved upstream IDs without persisting derived state.
8. Human and JSON failures use the documented codes, details, exits, and no-write guarantees.

## Non-functional acceptance criteria

1. Claim recheck, exact deletion, result hydration, availability calculation, and commit are one SQLite transaction and cannot expose a partial or mixed-state result.
2. The operation is safe across concurrent local processes and linked worktrees sharing the canonical database.
3. Force-unclaim is local and network-free and feels immediate for a personal repository under ordinary contention.
4. JSON fields use stable camelCase, timestamps use RFC 3339 UTC, and unresolved dependency IDs are deterministic.
5. Core owns claim comparison, transactions, typed errors, and availability derivation; CLI owns parsing, prompting, and rendering.
6. The UI contract makes destructive intent explicit without representing logical actors as a security boundary.

## Verification

- Force-unclaim through interactive accept/decline, `--yes`, JSON, and non-interactive modes; verify exact output, exits, and mutation behavior.
- Exercise missing task, missing claim, conflicting flags, and missing confirmation with exact validation precedence and no writes.
- Remove or replace the observed claim between prompt and transaction and verify `CLAIM_NOT_FOUND` or `CLAIM_CHANGED` without deleting a replacement claim.
- Release claims from otherwise-available, archived, completed, and dependency-blocked tasks; verify exact availability shape, reason precedence, and sorted unresolved upstream IDs.
- Compare task state before and after success to prove only the claim changed, including `updatedAt`, `updatedBy`, content, hierarchy, and dependencies.
- Inject deletion, hydration, and availability-query failures and verify rollback preserves the original claim.
- Coordinate force-unclaim with claim lifecycle operations in separate processes and linked worktrees; verify one committed current-state outcome and no database corruption.
- Run formatting, lint, and workspace tests through `mise` after implementation.

## Out of scope

- Authentication or proof that a human rather than an agent process invoked a logical-user command.
- Agent-runtime approval rules for user-only commands.
- Automatic expiry, heartbeats, background stale detection, claim transfer, notifications, or daemons.
- Changing task status or mutation metadata as a consequence of claim release.
- Claim/status lifecycle regression consolidation; E5-S5.
- Force-archive, which remains governed by E2-S5's separate archive confirmation contract.

## Implementation task

See [E5-S4-T1: Implement user force-unclaim](../tasks/E5-S4-T1-implement-user-force-unclaim.md).
