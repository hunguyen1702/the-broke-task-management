---
id: E9-S1-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E1-S5-T1
  - E5-S1-T1
  - E5-S2-T1
---

# E9-S1-T1: Harden concurrent claim verification

## Parent story

[E9-S1: Verify concurrent claim safety](../epics/E9-S1-verify-concurrent-claim-safety.md)

## Objective

Add a deterministic, bounded process-level reliability suite that repeatedly proves specified-task and claim-next concurrency semantics across the main Git worktree and multiple linked worktrees.

## Deliverables

- A reusable test-only child-process readiness/start gate.
- Fixed-count race coverage for specified-task claim, one-candidate claim-next, and multi-candidate claim-next.
- Main-to-linked and linked-to-linked contention using at least two linked worktrees and one canonical database.
- Exact exit, JSON outcome, winner identity/timestamp, claim-row, canonical-store, and SQLite-integrity assertions.
- Actionable per-round failure diagnostics.
- No production code, migration, public output, or acceptance-scenario changes unless a separately approved remediation task is created.

## Proposed structure

Prefer a focused integration test file so reliability repetitions and process orchestration remain distinct from single-behavior cases:

```text
crates/tbtm-cli/tests/
├── task_claim.rs
└── claim_concurrency.rs
```

Small shared test helpers may be extracted only when doing so reduces duplicated Git-worktree or command setup without creating a production API. Core claim behavior stays in `tbtm-core`; the new work verifies it through the compiled CLI boundary.

## Technical choices

- Spawn `CARGO_BIN_EXE_tbtm` as independent OS processes, preserving stdout, stderr, and exit status for each contender.
- Introduce a test-only gate, such as inherited pipe/file-descriptor signaling or an equivalent cross-platform ready/release protocol, that blocks each child immediately before invoking the claim command.
- Require every child to signal readiness before the parent releases the round. Do not use `sleep`, elapsed-time assertions, or hoped-for scheduler overlap.
- Use fresh tasks and claims per round so outcomes are attributable and cleanup does not participate in the race.
- Use a fixed repetition count documented as a test constant. Start with 20 repetitions per race/worktree pairing; reduce it only with measured suite-cost evidence while retaining repeated coverage.
- Keep contender count small and fixed for routine verification. At minimum use two contenders; multi-candidate claim-next provides at least one candidate per contender.
- Inspect the canonical database directly only after child processes complete. Assert row invariants and `PRAGMA integrity_check = 'ok'`; do not mutate state through the assertion connection.
- Keep the existing SQLite journal mode, busy timeout, immediate transactions, and command contracts unchanged.

## Implementation flow

### 1. Build isolated shared-worktree fixtures

1. Create a temporary Git repository with an initial commit.
2. Add at least two linked worktrees using normal Git commands.
3. Initialize TBTM once at the canonical store and register one agent per contender.
4. Verify all invocation roots resolve the same database and that linked worktrees contain no `.tbtm` directory.

The fixture must retain the temporary-directory owner for the entire test and provide explicit main, linked-A, linked-B, and canonical database paths.

### 2. Implement deterministic gated processes

Create a test-only launcher that captures:

- race kind and repetition number;
- invocation worktree and agent;
- full argument vector;
- exit status, stdout, and stderr.

Each contender reports ready at the last test-controlled point before execution. The parent waits for all readiness signals, releases all contenders, waits for completion, and returns labeled results. A readiness or child timeout may prevent a hung test, but elapsed time must not determine correctness.

If a wrapper/helper process is required to create the gate, it remains test-only and ultimately executes the real `tbtm` binary with unchanged user-facing arguments.

### 3. Verify specified-task races

For every fixed repetition and required worktree pairing:

1. Create one fresh available task.
2. Launch at least two agents with `task claim <id> --agent <uuid> --json`.
3. Assert exactly one exit `0` response with a populated claim.
4. Assert every loser exits `4` with `CLAIM_CONFLICT`.
5. Load the one canonical claim row and assert the winner agent and `claimedAt` match both the success and every conflict detail exactly.
6. Assert the task has exactly one claim and the winner timestamp was not refreshed.

### 4. Verify claim-next races

One-candidate matrix:

1. Create one fresh available candidate and isolate it from earlier rounds.
2. Launch at least two agents with equivalent `task claim-next ... --json` filters selecting only that candidate set.
3. Assert all processes exit `0`, exactly one response has the task, all others have `data: null`, and no response contains `CLAIM_CONFLICT`.
4. Assert the canonical database contains exactly one matching claim.

Multi-candidate matrix:

1. Create at least one fresh candidate per contender with deterministic priorities/order.
2. Use filters that isolate the round's candidates.
3. Launch contenders concurrently and assert all exit `0` with distinct task IDs.
4. Assert the claimed set equals the first current candidates under `priority DESC, created_at ASC, id ASC` and every selected task has one claim.

Use a unique exact tag or equivalent supported filter per round to prevent unclaimed fixtures from another round entering claim-next selection.

### 5. Verify shared storage and integrity

After each round, or after a documented isolated batch if per-round integrity checks make the suite impractical:

- query the canonical main-worktree database for expected claim rows;
- assert no task has more than one active claim;
- run `PRAGMA integrity_check` and require exactly `ok`;
- assert neither linked worktree contains `.tbtm` or a database side store.

Failures must print the labeled child outcomes and relevant persisted rows without discarding stdout or stderr.

### 6. Preserve scope on discovered failures

If the reliability suite fails against current production behavior:

1. Keep a minimal reproducible failing test or captured evidence.
2. Report E9-S1-T1 blocked by the discovered defect.
3. Propose a separate remediation task tied to the authoritative E1-S5, E5-S1, or E5-S2 contract.
4. Do not change production code or observable behavior under this implementation task.

## Test plan

### Harness tests

- Prove the gate waits for every contender and releases all children without sleeps.
- Prove child failures retain status, stdout, stderr, race label, worktree pairing, and repetition in diagnostics.
- Prove fixture setup yields one canonical database and no linked-worktree store.

### Reliability matrix

- Specified claim: main versus linked-A and linked-A versus linked-B.
- Claim-next with one candidate: main versus linked-A and linked-A versus linked-B.
- Claim-next with multiple ordered candidates: main versus linked-A and linked-A versus linked-B.
- Repeat each matrix entry using the fixed test constant.
- For specified claim, validate one winner plus exact persisted-winner conflicts.
- For claim-next, validate success-plus-empty or distinct ordered successes and the absence of conflicts.
- Validate claim uniqueness, winner timestamp preservation, canonical-store targeting, and SQLite integrity throughout.

### Regression boundary

- Retain existing single-command, validation, busy-timeout, rollback, filter, and output tests in `task_claim.rs`.
- Do not duplicate unrelated availability branches already covered by E5-S1/E5-S2 tests.
- Run the full workspace suite to catch interference or excessive runtime.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

During implementation, run the focused integration target repeatedly before the full suite, using Cargo's exact test-target name established by the new file.

## Definition of done

- Every E9-S1 functional and non-functional acceptance criterion passes.
- The deterministic process gate and fixed repetition matrix cover both required worktree pairings and all three race shapes.
- Specified claim proves one winner, exact persisted-winner conflicts, and no overwrite or timestamp refresh.
- Claim-next proves one-candidate success-plus-empty and multi-candidate distinct ordered claims without conflicts.
- Canonical row checks and `PRAGMA integrity_check` prove uniqueness and database integrity; linked worktrees remain free of local TBTM stores.
- Existing public behavior and production code remain unchanged, or a discovered production defect is reported as a blocker for separate remediation.
- Acceptance impact is classified as `none` because the change is internal automated verification only.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [Product requirements: E9-S1](../PRD.md#story-e9-s1-verify-concurrent-claim-safety)
- [E9-S1 epic](../epics/E9-S1-verify-concurrent-claim-safety.md)
- [E1-S5 shared Git-worktree state](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [E1-S5 implementation task](E1-S5-T1-implement-shared-git-worktree-repository-resolution.md)
- [E5-S1 specified-task claim contract](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [E5-S1 implementation task](E5-S1-T1-implement-atomic-specified-task-claiming.md)
- [E5-S2 claim-next contract](../epics/E5-S2-claim-the-next-available-task-atomically.md)
- [E5-S2 implementation task](E5-S2-T1-implement-atomic-next-available-task-claiming.md)
- [Existing claim integration tests](../../crates/tbtm-cli/tests/task_claim.rs)
