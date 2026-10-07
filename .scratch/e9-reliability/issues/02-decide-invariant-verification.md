# Decide the graph and availability verification boundary

Type: grilling
Status: resolved
Parent: [E9 reliability and guidance implementation route](../map.md)
Blocked by: 01

## Question

Which missing cross-operation checks and existing proofs together satisfy graph/availability verification, and what approved story and implementation-task boundary should capture them?

## Context

- [Graph and availability requirements](../../../docs/PRD.md#story-e9-s2-verify-graph-and-availability-invariants).
- Existing behavior belongs to hierarchy, dependency, availability, blocker, relationship-map, archive/unarchive, and status-completion contracts.
- Availability requires every direct upstream to be effectively completed. An effectively completed direct upstream satisfies its edge even if its own upstream remains unresolved. Recursive blocker context must preserve that distinction.
- Hierarchy and dependency cycles are independently prohibited; containment alone cannot block work.
- Verify archive/unarchive and status-semantic transitions immediately, deterministic priority/time/ID ordering, and preserved claims where required.
- Audit existing core and CLI tests before proposing additions; record reusable proofs and concrete gaps. Decide whether evidence warrants process-boundary and linked-worktree coverage beyond existing suites.

## Resolution criterion

Agree on a narrow acceptance matrix, the test layer owning each proof, and the handling of discovered production defects. Produce an approved story/task plan with exact dependencies and verification commands; update its authoritative frontmatter and dashboard together.

## Comments

- 2026-10-07: user confirmed the final shared understanding. Story/task approval and dashboard metadata were updated together; ticket 02 is resolved. Implementation remains unclaimed.
- 2026-10-07 draft validation: referenced reusable test names were checked against the repository; `rtk mise run format`, `rtk mise run lint`, `rtk mise run test`, and `rtk git diff --check` passed. These establish the current baseline, not acceptance of the planned new E9-S2 proofs. Draft frontmatter and dashboard both record planning `in_progress`, implementation `not_planned` pending final confirmation. No source code changed.
- 2026-10-07: user accepted Q3 and Q4. One real CLI journey in a temporary repository owns cross-operation checks; a focused ordering test owns the final tie-break. Reuse existing cycle, blocker, map, and linked-worktree evidence without a new concurrency/worktree matrix. Package one verification-only implementation task under E9-S2 with an acceptance-to-test matrix and exact commands. [Draft story](../../../docs/epics/E9-S2-verify-graph-and-availability-invariants.md) and [draft task](../../../docs/tasks/E9-S2-T1-verify-graph-and-availability-transitions.md) are prepared; final shared-understanding confirmation remains pending before approval and ticket resolution.
- 2026-10-07: user accepted Q1's proof scope: reuse existing graph/availability proofs; add focused checks for normal A-to-B-to-C claim eligibility and completion exceptions allowed by current contracts, exact available-task results after archive/unarchive and status-completion changes, exact downstream claim owner/timestamp preservation, and ID ordering when priority and creation time tie. Include both status-completion transition directions and multiple tasks sharing the status. Ticket 08 remains unresolved and non-blocking. Proof ownership and implementation-task packaging are the next decisions.
- 2026-10-07: user accepted Q2's recommended production-defect boundary. Retain a minimal reproducible failing test or captured evidence, create a separate remediation task tied to the violated authoritative contract, and block E9-S2 completion until the fix and verification rerun pass. Do not silently expand the verification task into production changes. Acceptance scope and proof ownership remain pending.
- 2026-10-07: user directed E9-S2 to continue under current contracts. The potential requirement to satisfy direct dependencies before marking a task completed is deferred to [decision ticket 08](08-decide-completion-dependency-precondition.md), which remains open and is not an E9-S2 dependency. The verification matrix and remediation boundary still require agreement.
- Read-only audit: existing CLI suites cover cycles (`task_hierarchy.rs`, `task_dependency.rs`), concurrent opposite dependency additions, availability predicates and ordering (`task_available.rs`), recursive blockers/maps, status-completion impact, and unarchive impact. Files live in `crates/tbtm-cli/tests/`.
- Candidate gap: a consolidated multi-level dependency journey exercising archive, unarchive, and repository-wide completion-semantic changes with exact ordered availability after each transition. This is a proposed proof, not an approved decision.
- Baseline format, lint, and full workspace tests passed during charting.
- 2026-10-07 grilling round 1: remediation boundary and incremental proof scope are pending user decisions. Recommend separate remediation tasks for production defects and reuse of existing proofs plus a focused cross-operation journey. The audit found existing linked-worktree coverage for availability/claims, status/relationships/maps, unarchive, blockers, and read-only maps; adding another full worktree matrix is not yet justified. Candidate additions include exact post-transition availability, completed-direct-upstream stop semantics, both completion-semantic transition directions across tasks sharing a status, exact downstream claim preservation, and a forced creation-time tie for ID ordering. These remain proposals, not approved requirements or claims that all underlying core proofs are missing.

## Answer

Approve [E9-S2](../../../docs/epics/E9-S2-verify-graph-and-availability-invariants.md) and its single verification-only [implementation task E9-S2-T1](../../../docs/tasks/E9-S2-T1-verify-graph-and-availability-transitions.md). Their exact dependency sets, acceptance-to-test matrix, and focused/full verification commands are recorded in the authoritative documents. All prerequisites are done; planning is `done` and implementation is `ready`, matching `docs/STATUS.md`.

Reuse existing cycle, predicate, recursive blocker/map, and linked-worktree proofs. One real CLI journey owns normal chain eligibility, current completion exceptions, containment independence, exact availability after archive/unarchive and both repository status-completion directions, multiple tasks sharing a status, and exact downstream claim preservation. A focused availability test proves the final ID ordering tie-break. No new concurrency or linked-worktree matrix is required.

Retain reproducible evidence for production defects and require separately approved remediation; E9-S2 completion waits for fixes and passing reruns. [Decision ticket 08](08-decide-completion-dependency-precondition.md) remains open and non-blocking. Current public contracts govern verification. No new domain terms or architectural trade-offs require glossary entries or an ADR.
