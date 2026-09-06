---
id: E7-S4
kind: epic
planning_status: done
implementation_status: in_progress
contract_depends_on:
  - E4-S3
  - E4-S4
  - E5-S1
  - E5-S2
  - E5-S3
  - E5-S4
  - E5-S5
  - E7-S1
---

# E7-S4: Expose availability and claim operations

## Outcome

Coding agents and users can discover and invoke the complete work-acquisition lifecycle from one coherent CLI surface, using informative availability reads and authoritative atomic claim mutations without confusing the two.

## User story

As a coding agent, I want dedicated availability and atomic claim commands so that work acquisition is safe.

## Product decisions

### Complete command surface

E7-S4 consolidates and protects the command surface already introduced by E4 and E5:

```text
tbtm task available [--status <code>]... [--type <type>]... [--tag <tag>]...
tbtm task blockers <task-id>
tbtm task claim <task-id> --agent <uuid>
tbtm task claim-next --agent <uuid> [--status <code>]... [--type <type>]... [--tag <tag>]...
tbtm task unclaim <task-id> --agent <uuid>
tbtm task unclaim <task-id> --force [--yes]
```

- Existing names, nesting, required arguments, repeatable filters, and confirmation options remain authoritative. E7-S4 adds no top-level claim group, aliases, aggregate acquisition workflow, or alternative syntax.
- If implementation audit finds a listed operation missing or disconnected, it is wired to its existing authoritative core operation rather than reimplementing availability or claim logic in the CLI.
- `task available` is a stale-able informative query. Only `task claim` and `task claim-next` create ownership, and both retain their transaction semantics from E5.
- Status, hierarchy, dependency, and map configuration completeness belongs to E7-S3; comment completeness belongs to E7-S5.

### Actor and authority placement

- `task available` and `task blockers` are read-only and accept no actor selector.
- `task claim`, `task claim-next`, and normal `task unclaim` require an explicit registered-agent UUID through `--agent`; they never infer current-agent state or accept the logical `user`.
- Force-unclaim remains a distinct logical-user path on the same `task unclaim` verb. `--force` and `--agent` are mutually exclusive, and `--yes` is meaningful only with `--force`.
- E7-S4 does not add authentication, claim transfer, implicit identity, expiry, heartbeat, or automatic status transitions.

### Availability, conflicts, and confirmation

- `task available`, `task blockers`, specified claim, and claim-next reuse the exact E4-S3 availability predicate and established filters/order where applicable; no derived availability state is persisted.
- `task blockers` reports all current reasons through E4-S4 but does not reserve work or guarantee subsequent claim success.
- Specified claim retains stable current-state rejection and conflict results from E5-S1. Claim-next retains successful `data: null`/human empty output when no candidate exists and never fabricates a claim conflict.
- Normal unclaim preserves owner checks from E5-S3. Force-unclaim preserves E5-S4 observed-claim confirmation binding and stable `CLAIM_CHANGED` behavior under concurrent replacement.
- JSON and non-terminal force-unclaim require `--force --yes`; interactive human cancellation remains a successful no-write result. Other owned commands never prompt.
- Claim creation and removal never imply a status transition, and status mutation never implicitly creates or removes a claim, preserving E5-S5 independence.

### Help and transport preservation

- `task --help` exposes the five owned leaves under the established task hierarchy. Owned leaf help, especially `task unclaim --help`, states enough about required agent identity, filters, read-only versus atomic behavior, and force confirmation to form all six invocation forms.
- Help remains concise and does not duplicate the domain contracts or promise freshness for informative queries.
- Every operation consumes E7-S1's inherited, idempotent global `--json`, single response envelope, stream discipline, parse-error behavior, exit taxonomy, and non-interactive rules.
- Successful payloads, human projections, empty results, error codes/details, validation precedence, ordering, transactions, and concurrency outcomes remain owned by E4/E5.
- This story is an integration and completeness boundary. It adds no migration, persistent state, domain model, payload, error code, or concurrency protocol.

## Functional acceptance criteria

1. Task-group help exposes the five owned leaves, and their leaf help exposes the available-query, blocking-explanation, specified-claim, claim-next, owner-unclaim, and force-unclaim invocation forms with their established names and nesting.
2. Every listed command reaches its authoritative E4/E5 core behavior and preserves arguments, filters, payloads, human output, empty results, errors, exits, validation precedence, confirmations, and transaction contracts.
3. Read-only availability and blocker commands expose no actor option; claim and claim-next require a registered-agent UUID; unclaim preserves mutually exclusive owner and force paths.
4. Informative reads make no ownership guarantee, specified claim remains conflict-capable, and claim-next returns successful empty output when no current candidate exists.
5. Owner unclaim and force-unclaim preserve their distinct authorization, confirmation, observed-claim, and concurrent-current-state behavior while changing only the claim relation.
6. Claim and status remain independent across every owned command; no command adds an implicit workflow transition.
7. Every operation supports E7-S1 JSON placement and one-envelope behavior; only force-unclaim can prompt, and never in JSON or non-terminal execution.
8. Main and linked worktrees expose the same commands against the canonical shared repository state.

## Non-functional acceptance criteria

1. CLI wiring and help remain in `tbtm-cli`; availability rules, ownership, validation, persistence, transactions, and typed errors remain in `tbtm-core`.
2. A compact command-surface regression matrix detects missing commands, accidental name/nesting changes, filter or actor drift, force-path ambiguity, and shared-transport bypasses.
3. Coverage reuses owning E4/E5/E7-S1 tests for detailed predicates, error payloads, rollback, and concurrency rather than duplicating their full matrices.
4. Audit-driven production changes are minimal and correct only demonstrated exposure, help, or integration gaps.
5. The story adds no database migration, persistent field, external service, authentication mechanism, or concurrency protocol.

## Verification

- Inspect `task` help and every owned leaf help for discoverability, required identity, filters, mutually exclusive unclaim paths, and confirmation flags.
- Exercise representative human and JSON invocations for non-empty and empty availability, available and blocked explanations, specified-claim success/conflict, claim-next success/empty, owner release, and confirmed force release.
- Verify unsupported actor/force combinations fail before domain mutation and JSON/non-terminal force execution never prompts.
- Run a representative acquire/release chain from main and linked worktrees and confirm one canonical claim state.
- Preserve focused E4/E5/E7-S1 tests and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Redesigning command names, nesting, filters, payloads, errors, exit categories, confirmation semantics, availability rules, or transaction behavior.
- Adding aliases, a top-level claim group, batch claims, claim transfer, implicit current-agent state, expiry, heartbeat, or automatic status changes.
- Status, hierarchy, dependency, relationship-map, repository, task-content, agent, or comment command completeness owned by the neighboring E7 stories.
- New persistence, migrations, authentication, networking, daemon coordination, or Visual Studio Code integration.
- Creating, editing, reviewing, approving, or executing acceptance scenarios during planning or implementation.

## Implementation task

See [E7-S4-T1: Harden the availability and claim CLI surface](../tasks/E7-S4-T1-harden-availability-and-claim-cli-surface.md).
