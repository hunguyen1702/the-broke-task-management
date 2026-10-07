# E9 graph, availability, and recovery verification

Status: implemented
Source: [Reliability map](map.md)

## Problem Statement

Humans and coding agents rely on repository-local task data to choose eligible work and preserve ownership. Existing individual-operation tests do not yet provide the agreed cross-operation evidence that graph rules, effective completion, availability, and claims remain consistent through lifecycle changes. Recovery evidence also needs to establish that interrupted mutations preserve original state and that a human can safely back up and restore the canonical workspace.

Without these proofs and an executable offline recovery procedure, users cannot confidently distinguish a safe workflow from an untested assumption about SQLite, dependency traversal, or backup contents.

## Solution

Complete the approved E9-S2 and E9-S3 verification scope using existing CLI and core interfaces. Add a real CLI lifecycle journey and deterministic ordering proof; bounded real CLI interruption cases and focused post-write rollback proofs; a tested offline backup/restore procedure; and evidence that common failures provide actionable guidance without modifying repository data.

Reuse existing cycle, concurrency, health, output, and linked-worktree evidence. Record acceptance-to-test matrices and require passing verification before completion. Preserve current product semantics and route discovered production defects into separate approved remediation tasks.

## User Stories

1. As a human user, I want hierarchy cycles rejected without partial writes, so that scope organization remains trustworthy.
2. As a human user, I want dependency cycles rejected without partial writes, so that execution relationships remain valid.
3. As a coding agent, I want only available tasks returned, so that I can select eligible work.
4. As a coding agent, I want a multi-level dependency chain to become claimable in order as direct upstream tasks become effectively completed, so that I do not start blocked work.
5. As a human user, I want current status-update semantics preserved when upstream work is unresolved, so that verification does not change my workflow.
6. As a coding agent, I want a completed direct upstream task to satisfy its dependency even when its own upstream remains unresolved, so that eligibility follows the existing contract.
7. As a coding agent, I want an archived direct upstream task to satisfy its dependency, so that archived work does not unnecessarily block selection.
8. As a coding agent, I want blocker explanations to stop at effectively completed upstream tasks, so that they identify unresolved execution requirements.
9. As a human user, I want relationship maps to traverse effectively completed tasks, so that I retain structural context.
10. As a human user, I want hierarchy alone to leave availability unchanged, so that containment does not introduce execution requirements.
11. As a coding agent, I want availability to reflect archive and unarchive immediately, so that my next query observes committed state.
12. As a human user, I want changing repository-wide status completion in either direction to affect every task using that status, so that workflow semantics remain consistent.
13. As a human user, I want archive to retain effective completion independently of status completion changes, so that archive semantics remain stable.
14. As a coding agent, I want my exact claim owner and claim timestamp preserved when a lifecycle change introduces an unresolved dependency, so that eligibility changes do not silently alter ownership.
15. As a human user, I want successful archive to release the target claim and unarchive to leave it unclaimed, so that former ownership is not restored implicitly.
16. As a coding agent, I want deterministic priority, creation-time, and task-ID ordering, so that tied candidates have a predictable order.
17. As a human user, I want interrupted claim acquisition to preserve original claims and task metadata, so that process termination does not leave partial ownership.
18. As a human user, I want interrupted dependency addition to preserve original relationships and metadata, so that task readiness remains trustworthy.
19. As a human user, I want interrupted parent replacement to preserve the original hierarchy and metadata, so that containment remains trustworthy.
20. As a maintainer, I want post-write failures to roll back claim and relationship mutations, so that errors do not leave partially applied state.
21. As a human user, I want recovered data to pass integrity, foreign-key, and repository-health checks, so that preserved rows are usable.
22. As a human user, I want exact offline backup steps for the complete canonical workspace, so that configuration and database identity stay matched.
23. As a human user, I want backups stored outside the repository to survive uninstall, so that removing the live workspace does not remove recovery data.
24. As a human user, I want restoration to preserve displaced state and refuse destination overwrite, so that a recovery attempt does not destroy another recovery option.
25. As a human user, I want restored agents, tasks, claims, and relationships to match saved state after live changes, so that restoration demonstrably recovers the backup.
26. As a coding agent in a linked worktree, I want restored access to resolve to the canonical store, so that all worktrees observe the same recovered repository.
27. As a human user, I want configuration, database, initialization, repository-access, and permission failures to identify safe next steps, so that I can act without accidental repair or replacement.
28. As a coding agent, I want stable failure codes, exit statuses, and JSON streams, so that automation can handle recovery failures correctly.
29. As a maintainer, I want interruption proofs to establish that the requested write began and could not commit, so that test success represents the required boundary.
30. As a maintainer, I want unsupported platform evidence and unreliable gating reported explicitly, so that missing proof is never mistaken for acceptance.
31. As a human user, I want discovered production defects retained as reproducible evidence and separately remediated, so that verification completion means the existing contracts actually pass.

## Implementation Decisions

- Keep two separate verification-only implementation tasks under E9-S2 and E9-S3. Their approved contracts, dependency sets, and verification commands remain authoritative; this combined spec synthesizes them without replacing them. Neither task is claimed or started by publishing this spec.
- Core owns rules, SQLite transactions, and repository resolution; CLI owns parsing, prompts, output, and exit codes. Extend existing test patterns rather than adding production verification interfaces, hooks, commands, migrations, or dependencies.
- Availability requires an active task with incomplete status, no claim, and every direct upstream dependency effectively completed. Effective completion means archive or completed status. Claims remain independent of status and dependency satisfaction.
- Use one real CLI journey for cross-operation graph/availability coverage. Start with C depending on B and B depending on A; prove normal eligibility and actual claim outcomes, then separately prove the permitted completed-B exception with A unresolved. Compare available IDs, blocker explanations, and map state according to their distinct contracts.
- Cover containment independence; incomplete-upstream archive/unarchive; both directions of status-completion changes across multiple active tasks sharing a status; and an archived task using that status. Assert exact ordered available IDs after each commit and explicitly account for existing claims.
- Compare complete downstream claim summaries, including owner identity and original claim timestamp, before and after blocking changes. Verify archive releases its target claim and unarchive does not restore it.
- Extend the existing ordering proof with deliberately equal priority and creation time, using test-owned SQLite fixture setup for timestamps and the CLI for the query. Direct database writes in the graph journey scope are confined to this deterministic ordering setup.
- Validate the hook-free interruption harness before expanding it. On a fully migrated isolated fixture, establish a reader transaction that actually holds a SQLite SHARED lock, launch one real CLI writer, and establish journal evidence attributable to its requested mutation before commit. Journal existence or size alone does not establish this boundary.
- Bound the gate with a deadline shorter than the writer busy timeout, detect premature writer exit, and always kill and reap the writer before releasing the reader, including failed-test cleanup. Avoid sleep-based timing assumptions. Cover claim acquisition, dependency addition, and parent replacement.
- Reopen the existing database writable without creating a replacement to permit SQLite recovery where needed, then inspect original relevant rows, complete claims, actor/timestamp metadata, and unrelated state. Require successful integrity checks, empty foreign-key checks, and healthy read-only repository status. Do not require database byte equality.
- Exercise public core operations with test-owned post-write failure fixtures, such as SQLite triggers, for claim acquisition, dependency add/remove, and parent set/replace/remove. Cover meaningful partial-write and result-hydration boundaries where feasible without production hooks.
- Document exact runnable offline backup and restore steps and execute those same steps in verification. Stop all clients/writers across main and linked worktrees; resolve and validate the canonical workspace; recover/check existing SQLite where needed; close connections before copying the complete matching workspace to a new durable external destination.
- If health or canonical identity cannot be established, stop the normal procedure and preserve artifacts. Never force-initialize over them or manually delete journals. Avoid repository-root backup/staging locations removed by uninstall; prove the external backup survives actual uninstall in a disposable fixture.
- Restore only with clients stopped, preserve the displaced workspace externally, and refuse accidental overwrite of backup/displaced destinations. Restore matching configuration and database together, verify identity, representative saved data, integrity, foreign keys, health, and linked-worktree resolution. Deliberately mutate live fixture data between backup and restore. Retain both saved and displaced state on failed checks.
- Verify failure classifications for unavailable repository (exit 1), missing initialization (3), malformed/unsafe configuration and identity mismatch (2), missing/non-SQLite/corrupt database (1), and permission denial (5). Preserve authoritative stable codes, including `PERMISSION_DENIED`; require affected check/path where available and useful recovery direction. JSON errors use stdout with empty stderr; human errors/guidance use stderr. No creation, migration, or repair is permitted during failure inspection.
- An unreliable mutation-specific interruption gate blocks completion and reopens the design decision with retained feasibility evidence. Rollback-only evidence or production hooks cannot silently substitute for required CLI termination proof.
- Retain minimal reproducible evidence for production-contract violations and create separate remediation tasks. Verification completion waits for separately approved remediation and passing reruns.

## Testing Decisions

- Test externally observable behavior and preserved state rather than private implementation structure. Use explicit expected IDs and original-state comparisons; do not derive expected values by duplicating production availability predicates or merely assert that an error occurred.
- Prefer existing high seams: the real CLI for graph transitions, interruption, backup/restore, and failure output. Use the public core-operation seam only for focused post-write rollback boundaries that CLI tests cannot reliably inject. These seams were already confirmed in resolved decision tickets 02 and 03; no new seam approval is outstanding.
- Reuse existing CLI suites for hierarchy/dependency cycles, availability, blockers, relationship maps, archive/unarchive, status completion, claim/status independence, command surfaces, repository status, and linked-worktree resolution. Reuse completed E9-S1 concurrency evidence rather than building a new concurrency matrix.
- Follow existing temporary-repository, real-binary, JSON parsing, and deterministic SQLite fixture patterns in the availability suite. Keep fixture ownership alive throughout each test and include operation, expected/actual state, exit status, stdout, and stderr in failures.
- Add a focused ordering proof for the final task-ID tie-break and one CLI lifecycle journey for cross-operation gaps. Core rollback evidence must execute nonzero tests covering the listed mutations and compare original rows and metadata.
- Keep all new proofs bounded, deterministic, network-free, and isolated. Assert cleanup of child processes and locks even on failures. Use genuine permission denial; fixtures bypassed by root privileges cannot count as passing.
- Verify the documented offline procedure itself against populated saved data, deliberate live changes, linked-worktree canonical resolution, overwrite refusal, displaced-state preservation, and backup survival through uninstall.
- Record an acceptance-to-test matrix for each story, including reused proof ownership, final test names, focused results, executed platform, and any unavailable platform checks. Distinguish unavailable evidence from passing, failing, and blocked evidence.
- Run the exact focused commands in each approved implementation task and the required `rtk mise run format`, `rtk mise run lint`, and `rtk mise run test` gates before implementation handoff. Publication checks are planning evidence, not acceptance of unimplemented tests.

## Out of Scope

New product semantics or completion preconditions; remote coordination; automatic repair; new production APIs, commands, hooks, migrations, or dependencies; performance service levels; new concurrency/worktree matrices; live-writer backup snapshots; power-loss durability; guaranteed hot-journal replay; and acceptance-scenario catalog changes.

E8 implementation, early CLI workflow guidance, and E9-S4 workflow-guidance planning remain outside this effort. E9-S4 retains its existing dependency on all eight unfinished E8 stories. Open decision ticket 08 about completion preconditions remains separate and does not block E9-S2.

## Further Notes

- Approved graph story: [E9-S2](../../docs/epics/E9-S2-verify-graph-and-availability-invariants.md); implementation task: [E9-S2-T1](../../docs/tasks/E9-S2-T1-verify-graph-and-availability-transitions.md).
- Approved recovery story: [E9-S3](../../docs/epics/E9-S3-verify-data-recovery-behavior.md); implementation task: [E9-S3-T1](../../docs/tasks/E9-S3-T1-verify-interruption-and-offline-recovery.md).
- Prior confirmed decisions: [scope](issues/01-set-guidance-scope.md#answer), [graph/availability](issues/02-decide-invariant-verification.md#answer), and [recovery](issues/03-decide-recovery-verification.md#answer).
- Implementation completed on 2026-10-07. E9-S2 and E9-S3 acceptance matrices, focused results, and platform evidence are recorded in their authoritative tasks. The separately approved no-op migration remediation is recorded in ticket 14.
- Existing contracts remain authoritative. No new domain term or architectural trade-off requires a glossary change or ADR.
