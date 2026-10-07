---
id: E9-S2
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E3-S3
  - E4-S1
  - E4-S2
  - E4-S3
  - E4-S4
  - E4-S5
  - E2-S5
  - E2-S6
---

# E9-S2: Verify graph and availability invariants

Approved on 2026-10-07 after user confirmation of the shared understanding.

## Outcome

Automated verification proves that existing graph and availability contracts remain correct across supported lifecycle changes.

## User story

As a user, I want graph and availability rules tested so that agents do not receive blocked work.

## Scope and decisions

- Reuse existing cycle, availability, blocker, relationship-map, and linked-worktree proofs. Add only the agreed cross-operation and ordering gaps.
- With C depending on B and B depending on A, initially incomplete and unclaimed, verify claim eligibility progresses from A to B to C as each direct upstream becomes effectively completed.
- Preserve current contracts: status updates do not require satisfied dependencies or an existing claim. A completed or archived B satisfies C's direct dependency even when A remains unresolved.
- Blocker explanations stop at effectively completed upstream tasks. Relationship maps continue through those tasks. Hierarchy alone cannot block availability.
- Verify exact ordered available-task IDs immediately after archive, unarchive, and both directions of repository-wide status completion changes, including multiple tasks sharing a status.
- Existing downstream claims retain the exact owner and original timestamp when unarchive or status completion changes introduce blockers. Archive releases its target's claim; unarchive does not restore it.
- Verify ordering by priority descending, creation time ascending, and task ID ascending, including a deliberate priority/time tie.
- One real CLI journey owns cross-operation verification; a focused ordering test owns the final tie-break. No new concurrency or linked-worktree matrix is required.
- Discovered production defects require retained reproducible evidence and a separately approved remediation task. E9-S2 completion waits for the fix and passing verification.

## Acceptance criteria

1. Existing hierarchy and dependency cycle proofs pass, including their no-partial-write assertions.
2. The multi-level journey proves normal claim eligibility and current direct-dependency satisfaction exceptions.
3. Availability, blocker explanations, and map state agree with their distinct contracts, including containment independence and completed-upstream traversal boundaries.
4. Every lifecycle transition returns the exact expected ordered available-task IDs without a refresh or persisted derived-state update.
5. Both repository status-completion directions affect all tested tasks sharing the status; archive continues to supply effective completion independently of status.
6. Downstream claims remain exactly equal across blocking changes, while archive/unarchive follow target-claim release and non-restoration rules.
7. All three ordering keys are proved, including equal priority and creation time.
8. Tests are deterministic, bounded, network-free, and isolated in temporary repositories. Fixture ownership and command diagnostics survive for the full test.
9. The implementation records an acceptance-to-test matrix, focused results, and passing format, lint, and workspace tests.

## Out of scope

New completion preconditions, product semantics, commands, migrations, production hooks, concurrency matrices, performance targets, recovery/backup work, and acceptance-scenario catalog changes.

The [completion-precondition decision](../../.scratch/e9-reliability/issues/08-decide-completion-dependency-precondition.md) remains open and does not block this story.

## Implementation task

[E9-S2-T1: Verify graph and availability transitions](../tasks/E9-S2-T1-verify-graph-and-availability-transitions.md).

## Planning review

- Contract dependencies match the PRD and dashboard; all prerequisite contracts are approved and planned.
- Existing E2-S5/S6, E3-S3, E4-S1 through S5, and E5-S5 semantics remain authoritative.
- User confirmed the shared understanding on 2026-10-07. E9-S2-T1 completed on 2026-10-07 with passing focused and workspace verification.
