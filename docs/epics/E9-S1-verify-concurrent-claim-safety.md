---
id: E9-S1
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E1-S5
  - E5-S1
  - E5-S2
---

# E9-S1: Verify concurrent claim safety

## Outcome

Automated reliability tests repeatedly prove that concurrent claim commands preserve one-owner task claims and the documented outcomes across processes and Git worktrees.

## User story

As a user, I want claim concurrency verified so that duplicate agent work is prevented.

## Scope

- Harden automated verification for concurrent `task claim` and `task claim-next` operations.
- Exercise real CLI child processes against one canonical SQLite database rather than only concurrent calls within one process.
- Cover contenders launched from the main worktree and at least two linked worktrees.
- Use a deterministic readiness/start gate so contenders are released together without sleep-based coordination or timing assertions.
- Run a fixed, bounded repetition matrix that is strong enough to catch regressions while remaining suitable for the normal workspace test suite.
- Verify command outcomes, persisted claims, winner identity and timestamp, canonical-store use, and SQLite integrity.

## Product and solution decisions

### Verification-only contract

- E9-S1 adds reliability coverage; it does not add a command, change public output, alter claim semantics, introduce a migration, or replace SQLite coordination.
- E1-S5 remains authoritative for one canonical database across Git worktrees.
- E5-S1 remains authoritative for specified-task claim conflicts and winner preservation.
- E5-S2 remains authoritative for atomic next-task selection, successful empty results, and deterministic candidate ordering.
- If the new suite exposes a production defect, preserve the failing evidence and treat the defect as a blocker requiring a separate remediation task. Do not silently broaden E9-S1 into a behavior or production-code change.

### Specified-task claim race

- Every round starts with one available task and at least two registered contenders.
- Exactly one process exits `0` with the claimed task.
- Every loser exits `4` with `CLAIM_CONFLICT`.
- Each conflict reports the exact persisted winner agent identity and original `claimedAt`; no loser overwrites or refreshes the winner.
- The canonical database contains exactly one active claim for the task.

### Claim-next races

- With one candidate, exactly one contender returns a non-null claimed task and every other contender returns the documented successful empty result. All exit `0`, and none returns `CLAIM_CONFLICT`.
- With at least as many ordered candidates as contenders, every contender exits `0`, claimed task IDs are distinct, and the final claimed set is the first current candidates under E5-S2 ordering.
- These outcomes intentionally narrow the PRD shorthand that losing claim operations return conflicts: conflict is the losing outcome only for specified-task claim, while claim-next retains its E5-S2 empty-success contract.

### Process and worktree matrix

- Cover main-worktree versus linked-worktree contention and linked-worktree versus linked-worktree contention using at least two linked worktrees.
- Each process resolves the shared canonical store through normal CLI repository discovery.
- No linked worktree receives a `.tbtm` store or SQLite side database.
- The implementation task chooses and documents one fixed repetition count. The count must not depend on machine timing, environment variables, randomness, or an unbounded stress loop.

### Synchronization and assertions

- A test-owned readiness/start gate holds child processes before command execution and releases them only after all contenders report ready.
- Sleeps, elapsed-time thresholds, scheduler assumptions, and probabilistic overlap are not correctness evidence.
- After each round, or after a clearly isolated batch whose individual rows remain attributable, inspect the canonical database for expected claim rows and `PRAGMA integrity_check = 'ok'`.
- Test diagnostics identify the race kind, repetition, worktree pairing, command status, stdout, and stderr so a CI failure is actionable.

## Functional acceptance criteria

1. Repeated specified-task races produce exactly one exit-`0` winner and only exit-`4` `CLAIM_CONFLICT` losers.
2. Every specified-task conflict reports the persisted winner's exact agent identity and unchanged claim timestamp.
3. Repeated one-candidate claim-next races produce exactly one non-null success and successful empty results for all other contenders, never a conflict.
4. Repeated multi-candidate claim-next races produce distinct claims for the first ordered candidates without duplicate ownership.
5. The full race matrix covers main-to-linked and linked-to-linked contenders against one canonical database.
6. Every verified round or isolated batch contains the expected active-claim rows, no task has more than one active claim, and SQLite integrity is `ok`.
7. Linked worktree races create no linked-worktree-local TBTM store or database.
8. A failed assertion reports enough process and round context to diagnose the observed outcome.

## Non-functional acceptance criteria

1. Concurrency is coordinated with an explicit readiness/start gate and does not rely on sleeps or timing thresholds.
2. Repetitions are fixed and bounded so the suite is deterministic and practical in routine local and CI verification.
3. Tests invoke the real CLI process boundary and remain isolated in temporary Git repositories.
4. The suite performs no network access and leaves no durable test repositories.
5. Production source, schema, journal mode, busy timeout, and public command contracts remain unchanged when the existing implementation satisfies the verification.

## Test and verification approach

- Add a focused process-level reliability test module or extend the existing claim integration suite with a reusable gated-child harness.
- Build isolated main and linked worktrees, initialize once, register distinct agents, and prepare fresh candidates for each round.
- Run the specified-claim, one-candidate claim-next, and multi-candidate claim-next matrices with exact JSON and exit assertions.
- Query only the main worktree's canonical database for row-level ownership, timestamp preservation, uniqueness, and `PRAGMA integrity_check`.
- Assert linked worktrees have no `.tbtm` artifacts.
- Run repository formatting, lint, and workspace tests.

## Out of scope

- New claim, retry, reservation, transfer, expiry, heartbeat, or automatic status behavior.
- Changing `CLAIM_CONFLICT`, claim-next empty success, candidate ordering, busy-timeout behavior, or output envelopes.
- WAL, external lock files, daemon coordination, network or cross-machine concurrency.
- Performance benchmarking, elapsed-time service levels, randomized or unbounded soak testing.
- Acceptance-scenario catalog changes or acceptance execution.
- Fixing a production defect discovered by this verification work without a separately approved remediation task.

## Planning review and dependency cross-check

- Decision review: `READY`.
- Final direct-document review: `READY`.
- Re-read E1-S5/E1-S5-T1, E5-S1/E5-S1-T1, and E5-S2/E5-S2-T1 after creating this plan.
- Cross-checked canonical storage, claim identity, specified-claim conflict output, claim-next empty and ordered outcomes, persistence, transaction ownership, busy-timeout behavior, and linked-worktree concurrency.
- `contract_depends_on` matches the PRD and status dashboard; E9-S1 consumes the existing claim model and does not define a competing source of truth.
- Result: `NO CONFLICT`.

## Implementation task

See [E9-S1-T1: Harden concurrent claim verification](../tasks/E9-S1-T1-harden-concurrent-claim-verification.md).
