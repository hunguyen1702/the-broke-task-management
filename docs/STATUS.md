# Planning and implementation status

This dashboard is the current human-readable status index. Frontmatter in an existing epic or implementation-task document is authoritative for that document. Until repository tooling replaces this dashboard, update both places in the same change.

## Acceptance testing

- Scenario catalog: `approved`
- Last simplified: 2026-09-04 (QA-oriented happy paths and likely user errors)
- Last revalidation: 2026-09-04 (simplified catalog approved after QA review)
- Last full execution: 2026-09-07
- Current result: passed (88 passed, 0 failed, 0 blocked in the latest full run)
- Latest targeted execution: 2026-09-05, AT-E4-S5-001 passed (4 cases)
- Detailed catalog: [Acceptance regression catalog](testing/README.md)

## Status values

- Planning: `needed`, `in_progress`, or `done`. `in_progress` is the
  repository-visible claim for one planning session.
- Implementation: `not_planned`, `ready`, `blocked`, `in_progress`, or `done`.
- `ready` means the implementation plan exists and its implementation dependencies are done.
- `in_progress` is the repository-visible claim for one active implementation session. Claim a `ready` task here and in its task/epic frontmatter before changing source code; other agents or sessions must not work on a task already marked `in_progress`.
- `done` means implementation and its required verification are complete.

## Dependency model

- Story/epic rows use **contract dependencies**. A story may begin planning only
  after every story named in its contract dependency set has
  `planning_status: done` and an approved story contract. Existing story files
  record the same set in frontmatter as `contract_depends_on`.
- Implementation-task rows use **implementation dependencies**. A task may be
  `ready` only after every task named in `depends_on` has
  `implementation_status: done`.
- Contract dependencies must name story IDs (`E<n>-S<n>`), while implementation
  dependencies must name task IDs (`E<n>-S<n>-T<n>`). Aggregate epic IDs such as
  `E2` are expanded below so parallel-planning eligibility is unambiguous.
- Two stories may be planned concurrently only when neither story is reachable
  from the other through `contract_depends_on`, all of each story's existing
  contract dependencies are already planned, and the sessions do not edit the
  same story files. Shared dashboard and handoff updates must be integrated
  without overwriting concurrent work.

## Next work

- **H1 is the first priority before all remaining product-story plans.**
  H1-T0 validated the node-first flow; H1-T1 Constitution is implemented and
  H1-T11 Current Truth is implemented and verified; acceptance impact is `none`.
  H1-T12 Intent is implemented and verified; acceptance impact is `none`. Plan later nodes from
  that flow, not the archived
  component roadmap. Do not begin E8 planning until H1 is complete or the
  user explicitly changes the priority.
- E1-S1-T2 is complete; repeat initialization now expects the authoritative
  exit `2`, and AT-E1-S1-001 passed its targeted rerun.
- E4-S5-T2 is complete with focused core and CLI regression coverage for child
  traversal in combined relationship maps.
- E2-S1-T1 is implemented.
- E3-S1-T1 is complete and provides the status-code contract required by task creation.
- E2-S2-T1 is implemented and verified.
- E2-S3-T1 is implemented and verified with atomic scalar patches, hierarchy-safe type changes, validated no-ops, actor attribution, and shared full-task output.
- E2-S4-T1 is implemented with atomic structured task-context updates.
- E2-S5-T1 is implemented with durable archive reasons, transactional claim release, force-confirmation claim binding, and effective-completion semantics.
- E2-S6-T1 is implemented with transactional unarchive, deterministic direct-impact reporting, explicit confirmation, and preserved claims.
- E3-S2-T1 is implemented and verified with ASCII case-insensitive name uniqueness, atomic custom status creation/rename/reordering, stable output and typed errors. Acceptance impact `add` is queued for the separate scenario workflow.
- E3-S3-T1 is implemented and verified with repository-scoped completion changes, deterministic before/after task impact, transactional confirmation, preserved claims, and immediate availability reuse. Acceptance impact `add` is queued for the separate scenario workflow.
- E3-S4-T1 is implemented and verified with atomic unused-custom-status deletion, active and archived task-usage protection, deterministic order compaction, and linked-worktree concurrency safety. Acceptance impact `add` is queued for the separate scenario workflow.
- E4-S2-T1 is implemented with transactional dependency management and cycle prevention.
- E5-S1-T1 is implemented with atomic specified-task claiming, stable conflict and unavailable errors, and claim hydration.
- E5-S3-T1 is implemented with transactional owner-controlled unclaim behavior and shared-worktree claim release.
- E4-S3-T1 is implemented with authoritative read-only available-task querying and a reusable caller-owned selector.
- E4-S4-T1 is implemented and verified with deterministic multi-reason explanations, bounded unresolved-graph traversal, snapshot consistency, read-only behavior, and linked-worktree coverage.
- E4-S1-T1 is implemented with transactional single-parent mutations, type/cycle validation, direct hydration, and deterministic recursive reads.
- E4-S5-T1 is implemented with deterministic cycle-safe recursive maps, snapshot-consistent node state, and stable human/JSON output.
- E1-S4-T1 is implemented with deterministic read-only agent and active-claim listing.
- E5-S2-T1 is implemented and verified with atomic next-available-task selection and claiming.
- E5-S4-T1 is implemented and verified with observed-claim confirmation binding, exact claim release, and authoritative post-release availability.
- E5-S5-T1 is implemented and verified with cross-command status/claim, actor, archive/unarchive, concurrency, and linked-worktree regression coverage.
- E6-S1-T1 is implemented and verified with durable actor-attributed comments, deterministic read-only listing, collision handling, migration coverage, and linked-worktree concurrency coverage.
- E6-S2-T1 is implemented and verified with ownership-aware atomic hard deletion, stable errors, rollback coverage, and linked-worktree concurrency coverage.
- E6-S3-T1 is implemented and verified with insert/list/delete architectural guards, immutable-field regression coverage, non-atomic correction boundaries, active/archived lifecycle parity, and linked-worktree coverage. Acceptance impact `add` is queued for the separate scenario workflow.
- E7-S1-T1 is implemented and verified with inherited idempotent JSON mode, JSON-aware parse failures, stream discipline, non-interactive confirmations, and one-envelope partial-uninstall errors. Acceptance impact `add` is queued for the separate scenario workflow.
- E7-S2-T1 is implemented and verified with command/help inventory, actor-boundary checks, and a canonical-store lifecycle across main and linked worktrees. Acceptance impact `none`.
- E7-S3-T1 is implemented and verified with explicit help for status confirmation and relationship roles, actor/confirmation boundary coverage, all five map directions, and a canonical-store journey across main and linked worktrees. Acceptance impact `revalidate` is queued for the separate scenario workflow.
- E7-S4-T1 is implemented and verified with availability/acquisition help, actor/force boundary coverage, claim conflict and empty-result checks, and a canonical-store lifecycle across main and linked worktrees. Acceptance impact is `none` because the audit found no user-visible behavior change.
- E7-S5-T1 is implemented and verified with immutable correction help, exact command/actor/input boundary checks, repeatable JSON transport coverage, and a canonical-store add/list/delete journey across main and linked worktrees. Acceptance impact `add` for E7-S5 is queued for the separate scenario workflow.
- E9-S1-T1 is implemented and verified with a deterministic process gate, 120 repeated race rounds across main-to-linked and linked-to-linked worktrees, exact command and canonical-row assertions, and SQLite integrity checks. Acceptance impact is `none`.

## H1 — Adaptive repository harness (first priority)

H1 is an engineering-enablement epic outside the `tbtm` product contract.
H1-T0 validated the node-first flow; H1-T1 Constitution is implemented and
verified. H1-T11 Current Truth is implemented as a generic Constitution workflow.
Acceptance impact for both tasks is `none`.
H1-T12 Intent planning and implementation are complete. H1-T13 Risk Router
planning is complete and implementation is ready. H1-T14–H1-T24
are unplanned placeholders with no approved implementation contract. Former
H1-T2–H1-T10 are
[archived for reference](handoff/H1-archived-component-roadmap.md), not active tasks.

| ID | Kind | Planning | Implementation | Depends on |
|---|---|---|---|---|
| [H1](epics/H1-build-adaptive-repository-harness.md) | Engineering epic | done | in_progress | — |
| [H1-T0](tasks/H1-T0-validate-working-flow.md) Validate working flow | Task | done | done | — |
| [H1-T1](tasks/H1-T1-implement-constitution.md) Constitution | Task | done | done | H1-T0 |
| [H1-T11](tasks/H1-T11-current-truth.md) Current Truth | Task | done | done | H1-T1 |
| [H1-T12](tasks/H1-T12-intent.md) Intent | Task | done | done | H1-T1, H1-T11 |
| [H1-T13](tasks/H1-T13-risk-router.md) Risk Router | Task | done | ready | H1-T1, H1-T11, H1-T12 |
| H1-T14 Direct path | Node placeholder | needed | not_planned | — |
| H1-T15 Research | Node placeholder | needed | not_planned | — |
| H1-T16 Explore | Node placeholder | needed | not_planned | — |
| H1-T17 Spike | Node placeholder | needed | not_planned | — |
| H1-T18 Plan | Node placeholder | needed | not_planned | — |
| H1-T19 Commitment Gate | Node placeholder | needed | not_planned | — |
| H1-T20 Execute | Node placeholder | needed | not_planned | — |
| H1-T21 Proof of Work | Node placeholder | needed | not_planned | — |
| H1-T22 Learning Promotion | Node placeholder | needed | not_planned | — |
| H1-T23 Integrated flow validation | Checkpoint placeholder | needed | not_planned | — |
| H1-T24 Manage Constitution rules | Deferred workflow placeholder | needed | not_planned | — |

## E1 — Repository foundation and identity

| ID | Kind | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|---|
| [E1-S1](epics/E1-S1-initialize-repository.md) | Epic | done | done | — |
| [E1-S1-T1](tasks/E1-S1-T1-implement-repository-initialization.md) | Task | done | done (`8721ecf`) | — |
| [E1-S1-T2](tasks/E1-S1-T2-align-repeat-init-acceptance-exit.md) | Task | done | done | E1-S1-T1 |
| [E1-S2](epics/E1-S2-resolve-repository-configuration.md) | Epic | done | done | E1-S1 |
| [E1-S2-T1](tasks/E1-S2-T1-implement-repository-configuration-resolution.md) | Task | done | done (`f535803`) | E1-S1-T1 |
| [E1-S5](epics/E1-S5-share-repository-state-across-git-worktrees.md) | Epic | done | done | E1-S2 |
| [E1-S5-T1](tasks/E1-S5-T1-implement-shared-git-worktree-repository-resolution.md) | Task | done | done (`d45f852`) | E1-S2-T1 |
| [E1-S3](epics/E1-S3-register-an-agent.md) | Epic | done | done | E1-S1 |
| [E1-S3-T1](tasks/E1-S3-T1-implement-agent-registration.md) | Task | done | done (`015ffc8`) | E1-S1-T1, E1-S2-T1, E1-S5-T1 |
| [E1-S4](epics/E1-S4-list-agents-and-claims.md) | Epic | done | done | E1-S3, E5-S1 |
| [E1-S4-T1](tasks/E1-S4-T1-implement-agent-and-claim-listing.md) | Task | done | done | E1-S3-T1, E2-S1-T1, E3-S1-T1, E4-S2-T1, E5-S1-T1 |

## E2 — Task content and lifecycle

| Story | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|
| [E2-S1](epics/E2-S1-create-a-task.md) | done | done | E1-S1, E1-S3, E3-S1 |
| [E2-S1-T1](tasks/E2-S1-T1-implement-task-creation.md) | done | done | E1-S1-T1, E1-S3-T1, E3-S1-T1 |
| [E2-S2](epics/E2-S2-view-and-list-tasks.md) | done | done | E2-S1 |
| [E2-S2-T1](tasks/E2-S2-T1-implement-task-detail-and-listing.md) | done | done | E2-S1-T1 |
| [E2-S3](epics/E2-S3-update-task-content.md) | done | done | E2-S1, E4-S1 |
| [E2-S3-T1](tasks/E2-S3-T1-implement-task-content-updates.md) | done | done | E2-S1-T1, E2-S2-T1, E4-S1-T1 |
| [E2-S4](epics/E2-S4-manage-structured-task-context.md) | done | done | E2-S1 |
| [E2-S4-T1](tasks/E2-S4-T1-implement-structured-task-context-updates.md) | done | done | E2-S1-T1, E2-S2-T1 |
| [E2-S5](epics/E2-S5-archive-a-task.md) | done | done | E2-S1, E4-S3, E5-S3 |
| [E2-S5-T1](tasks/E2-S5-T1-implement-safe-task-archival.md) | done | done | E2-S1-T1, E2-S2-T1, E4-S3-T1, E5-S3-T1 |
| [E2-S6](epics/E2-S6-unarchive-a-task-safely.md) | done | done | E2-S5, E4-S4 |
| [E2-S6-T1](tasks/E2-S6-T1-implement-safe-task-unarchive.md) | done | done | E2-S5-T1, E4-S4-T1 |

## E3 — Status workflow

| Story | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|
| [E3-S1](epics/E3-S1-use-default-statuses.md) | done | done | E1-S1 |
| [E3-S1-T1](tasks/E3-S1-T1-implement-default-status-codes.md) | done | done (`4d8ebb2`) | E1-S1-T1 |
| [E3-S2](epics/E3-S2-create-and-organize-custom-statuses.md) | done | done | E3-S1 |
| [E3-S2-T1](tasks/E3-S2-T1-implement-custom-status-creation-and-ordering.md) | done | done | E3-S1-T1 |
| [E3-S3](epics/E3-S3-change-status-completion-semantics.md) | done | done | E3-S2, E4-S3 |
| [E3-S3-T1](tasks/E3-S3-T1-implement-repository-status-completion-changes.md) | done | done | E3-S2-T1, E4-S3-T1 |
| [E3-S4](epics/E3-S4-delete-an-unused-custom-status.md) | done | done | E3-S2 |
| [E3-S4-T1](tasks/E3-S4-T1-implement-safe-custom-status-deletion.md) | done | done | E3-S2-T1, E2-S1-T1 |

## E4 — Hierarchy, dependencies, and availability

| Story | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|
| [E4-S1](epics/E4-S1-manage-task-hierarchy.md) | done | done | E2-S1 |
| [E4-S1-T1](tasks/E4-S1-T1-implement-task-hierarchy-management.md) | done | done | E2-S1-T1, E2-S2-T1 |
| [E4-S2](epics/E4-S2-manage-dependencies.md) | done | done | E2-S1 |
| [E4-S2-T1](tasks/E4-S2-T1-implement-dependency-management.md) | done | done | E2-S1-T1, E2-S2-T1 |
| [E4-S3](epics/E4-S3-query-available-tasks.md) | done | done | E3-S1, E4-S2, E5-S1 |
| [E4-S3-T1](tasks/E4-S3-T1-implement-available-task-querying.md) | done | done | E3-S1-T1, E4-S2-T1, E5-S1-T1 |
| [E4-S4](epics/E4-S4-explain-blocking.md) | done | done | E4-S2, E4-S3 |
| [E4-S4-T1](tasks/E4-S4-T1-implement-blocking-explanations.md) | done | done | E4-S2-T1, E4-S3-T1 |
| [E4-S5](epics/E4-S5-view-relationship-maps.md) | done | done | E4-S1, E4-S2 |
| [E4-S5-T1](tasks/E4-S5-T1-implement-relationship-maps.md) | done | done | E4-S1-T1, E4-S2-T1, E4-S3-T1, E5-S1-T1 |
| [E4-S5-T2](tasks/E4-S5-T2-restore-child-traversal-in-all-maps.md) | done | done | E4-S5-T1 |

## E5 — Multi-agent claiming

| Story | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|
| [E5-S1](epics/E5-S1-claim-a-specified-task-atomically.md) | done | done | E1-S3, E2-S1, E3-S1, E4-S2 |
| [E5-S1-T1](tasks/E5-S1-T1-implement-atomic-specified-task-claiming.md) | done | done | E1-S3-T1, E1-S5-T1, E2-S1-T1, E2-S2-T1, E3-S1-T1, E4-S2-T1 |
| [E5-S2](epics/E5-S2-claim-the-next-available-task-atomically.md) | done | done | E4-S3, E5-S1 |
| [E5-S2-T1](tasks/E5-S2-T1-implement-atomic-next-available-task-claiming.md) | done | done | E4-S3-T1, E5-S1-T1 |
| [E5-S3](epics/E5-S3-unclaim-owned-work.md) | done | done | E5-S1 |
| [E5-S3-T1](tasks/E5-S3-T1-implement-owner-controlled-task-unclaim.md) | done | done | E5-S1-T1 |
| [E5-S4](epics/E5-S4-force-unclaim-stale-work.md) | done | done | E5-S1 |
| [E5-S4-T1](tasks/E5-S4-T1-implement-user-force-unclaim.md) | done | done | E5-S1-T1, E5-S3-T1, E4-S3-T1, E2-S2-T1, E2-S5-T1 |
| [E5-S5](epics/E5-S5-preserve-claim-status-independence.md) | done | done | E3-S1, E5-S1, E5-S3, E2-S5 |
| [E5-S5-T1](tasks/E5-S5-T1-harden-claim-status-independence.md) | done | done | E3-S1-T1, E2-S3-T1, E5-S1-T1, E5-S3-T1, E2-S5-T1, E2-S6-T1 |

## E6 — Task comments and collaboration context

| Story | Planning | Implementation | Contract or implementation depends on |
|---|---|---|---|
| [E6-S1](epics/E6-S1-add-and-view-comments.md) | done | done | E1-S3, E2-S1 |
| [E6-S1-T1](tasks/E6-S1-T1-implement-task-comments.md) | done | done | E1-S3-T1, E2-S1-T1 |
| [E6-S2](epics/E6-S2-delete-comments-under-ownership-rules.md) | done | done | E6-S1 |
| [E6-S2-T1](tasks/E6-S2-T1-implement-ownership-aware-comment-deletion.md) | done | done | E6-S1-T1 |
| [E6-S3](epics/E6-S3-keep-comments-immutable.md) | done | done | E6-S1, E6-S2 |
| [E6-S3-T1](tasks/E6-S3-T1-harden-comment-immutability.md) | done | done | E6-S1-T1, E6-S2-T1 |

## E7 — Agent-first CLI

| Story | Planning | Implementation | Contract depends on |
|---|---|---|---|
| [E7-S1](epics/E7-S1-provide-consistent-command-output.md) | done | done | E1-S1 |
| [E7-S1-T1](tasks/E7-S1-T1-standardize-cli-output-boundary.md) | done | done | E1-S1-T1 |
| [E7-S2](epics/E7-S2-expose-repository-agent-and-task-operations.md) | done | done | E1-S1, E1-S2, E1-S3, E1-S4, E1-S5, E2-S1, E2-S2, E2-S3, E2-S4, E2-S5, E2-S6, E7-S1 |
| [E7-S2-T1](tasks/E7-S2-T1-harden-repository-agent-and-task-cli-surface.md) | done | done | E1-S1-T1, E1-S1-T2, E1-S2-T1, E1-S3-T1, E1-S4-T1, E1-S5-T1, E2-S1-T1, E2-S2-T1, E2-S3-T1, E2-S4-T1, E2-S5-T1, E2-S6-T1, E7-S1-T1 |
| [E7-S3](epics/E7-S3-expose-planning-relationships-and-status-operations.md) | done | done | E3-S1, E3-S2, E3-S3, E3-S4, E4-S1, E4-S2, E4-S3, E4-S4, E4-S5, E7-S1 |
| [E7-S3-T1](tasks/E7-S3-T1-harden-planning-relationship-and-status-cli-surface.md) | done | done | E3-S1-T1, E3-S2-T1, E3-S3-T1, E3-S4-T1, E4-S1-T1, E4-S2-T1, E4-S3-T1, E4-S4-T1, E4-S5-T1, E4-S5-T2, E7-S1-T1 |
| [E7-S4](epics/E7-S4-expose-availability-and-claim-operations.md) | done | done | E4-S3, E4-S4, E5-S1, E5-S2, E5-S3, E5-S4, E5-S5, E7-S1 |
| [E7-S4-T1](tasks/E7-S4-T1-harden-availability-and-claim-cli-surface.md) | done | done | E4-S3-T1, E4-S4-T1, E5-S1-T1, E5-S2-T1, E5-S3-T1, E5-S4-T1, E5-S5-T1, E7-S1-T1 |
| [E7-S5](epics/E7-S5-expose-comment-operations.md) | done | done | E6-S1, E6-S2, E6-S3, E7-S1 |
| [E7-S5-T1](tasks/E7-S5-T1-harden-comment-cli-surface.md) | done | done | E6-S1-T1, E6-S2-T1, E6-S3-T1, E7-S1-T1 |

## E8 — Visual Studio Code scrum experience

| Story | Planning | Implementation | Contract depends on |
|---|---|---|---|
| E8-S1 Connect the extension to repository state | needed | not_planned | E1-S2, E7-S1 |
| E8-S2 View and filter the scrum board | needed | not_planned | E2-S2, E3-S2, E4-S3, E5-S1, E8-S1 |
| E8-S3 Change status using the board | needed | not_planned | E3-S3, E8-S2 |
| E8-S4 Manage task details | needed | not_planned | E2-S1, E2-S2, E2-S3, E2-S4, E2-S5, E2-S6, E4-S1, E4-S2, E8-S1 |
| E8-S5 Manage claims from the UI | needed | not_planned | E5-S1, E5-S2, E5-S3, E5-S4, E5-S5, E8-S4 |
| E8-S6 Manage comments from the UI | needed | not_planned | E6-S1, E6-S2, E6-S3, E8-S4 |
| E8-S7 Explore relationship maps | needed | not_planned | E4-S5, E8-S4 |
| E8-S8 Configure statuses and inspect agents | needed | not_planned | E1-S4, E3-S1, E3-S2, E3-S3, E3-S4, E8-S1 |

## E9 — Reliability, verification, and product guidance

| Story | Planning | Implementation | Contract depends on |
|---|---|---|---|
| [E9-S1](epics/E9-S1-verify-concurrent-claim-safety.md) | done | done | E1-S5, E5-S1, E5-S2 |
| [E9-S1-T1](tasks/E9-S1-T1-harden-concurrent-claim-verification.md) | done | done | E1-S5-T1, E5-S1-T1, E5-S2-T1 |
| E9-S2 Verify graph and availability invariants | needed | not_planned | E3-S3, E4-S1, E4-S2, E4-S3, E4-S4, E4-S5, E2-S5, E2-S6 |
| E9-S3 Verify data recovery behavior | needed | not_planned | E1-S1, E4-S2, E5-S1 |
| E9-S4 Document agent and human workflows | needed | not_planned | E7-S1, E7-S2, E7-S3, E7-S4, E7-S5, E8-S1, E8-S2, E8-S3, E8-S4, E8-S5, E8-S6, E8-S7, E8-S8 |
