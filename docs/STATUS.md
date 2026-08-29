# Planning and implementation status

This dashboard is the current human-readable status index. Frontmatter in an existing epic or implementation-task document is authoritative for that document. Until repository tooling replaces this dashboard, update both places in the same change.

## Status values

- Planning: `needed` or `done`.
- Implementation: `not_planned`, `ready`, `blocked`, `in_progress`, or `done`.
- `ready` means the implementation plan exists and its implementation dependencies are done.
- `done` means implementation and its required verification are complete.

## Next work

- E2-S1-T1 is implemented.
- E3-S1-T1 is complete and provides the status-code contract required by task creation.
- E2-S2-T1 is implemented and verified.
- E2-S3 planning is complete; implementation is blocked until E4-S1 is planned and implemented.
- E2-S4-T1 is implemented with atomic structured task-context updates.
- E2-S5 planning is complete; implementation is blocked until E4-S3-T1 and E5-S3-T1 provide authoritative availability and claim-release behavior.
- E2-S6 planning is complete; implementation is blocked until E2-S5-T1 and E4-S4-T1 provide archive lifecycle and authoritative blocking explanations.
- E3-S2 custom statuses are intentionally deferred because the default statuses cover the core MVP workflow.
- E4-S2-T1 is implemented with transactional dependency management and cycle prevention.
- Plan next: E5-S1 to define atomic claiming of a specified task on top of E4-S2.
- E1-S4-T1 remains blocked until the task, status, dependency, and active-claim models exist.

## E1 — Repository foundation and identity

| ID | Kind | Planning | Implementation | Depends on |
|---|---|---|---|---|
| [E1-S1](epics/E1-S1-initialize-repository.md) | Epic | done | done | — |
| [E1-S1-T1](tasks/E1-S1-T1-implement-repository-initialization.md) | Task | done | done (`8721ecf`) | — |
| [E1-S2](epics/E1-S2-resolve-repository-configuration.md) | Epic | done | done | E1-S1 |
| [E1-S2-T1](tasks/E1-S2-T1-implement-repository-configuration-resolution.md) | Task | done | done (`f535803`) | E1-S1-T1 |
| [E1-S5](epics/E1-S5-share-repository-state-across-git-worktrees.md) | Epic | done | done | E1-S2 |
| [E1-S5-T1](tasks/E1-S5-T1-implement-shared-git-worktree-repository-resolution.md) | Task | done | done (`d45f852`) | E1-S2-T1 |
| [E1-S3](epics/E1-S3-register-an-agent.md) | Epic | done | done | E1-S1, E1-S5 implementation order |
| [E1-S3-T1](tasks/E1-S3-T1-implement-agent-registration.md) | Task | done | done (`015ffc8`) | E1-S1-T1, E1-S2-T1, E1-S5-T1 |
| [E1-S4](epics/E1-S4-list-agents-and-claims.md) | Epic | done | blocked | E1-S3, E5-S1 |
| [E1-S4-T1](tasks/E1-S4-T1-implement-agent-and-claim-listing.md) | Task | done | blocked | E1-S3-T1, E2-S1-T1, E3-S1-T1, E4-S2-T1, E5-S1-T1 |

## E2 — Task content and lifecycle

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| [E2-S1](epics/E2-S1-create-a-task.md) | done | done | E1-S1, E1-S3, E3-S1 |
| [E2-S1-T1](tasks/E2-S1-T1-implement-task-creation.md) | done | done | E1-S1-T1, E1-S3-T1, E3-S1-T1 |
| [E2-S2](epics/E2-S2-view-and-list-tasks.md) | done | done | E2-S1 |
| [E2-S2-T1](tasks/E2-S2-T1-implement-task-detail-and-listing.md) | done | done | E2-S1-T1 |
| [E2-S3](epics/E2-S3-update-task-content.md) | done | blocked | E2-S1, E4-S1 |
| [E2-S3-T1](tasks/E2-S3-T1-implement-task-content-updates.md) | done | blocked | E2-S1-T1, E2-S2-T1, E4-S1-T1 |
| [E2-S4](epics/E2-S4-manage-structured-task-context.md) | done | done | E2-S1 |
| [E2-S4-T1](tasks/E2-S4-T1-implement-structured-task-context-updates.md) | done | done | E2-S1-T1, E2-S2-T1 |
| [E2-S5](epics/E2-S5-archive-a-task.md) | done | blocked | E2-S1, E4-S3, E5-S3 |
| [E2-S5-T1](tasks/E2-S5-T1-implement-safe-task-archival.md) | done | blocked | E2-S1-T1, E2-S2-T1, E4-S3-T1, E5-S3-T1 |
| [E2-S6](epics/E2-S6-unarchive-a-task-safely.md) | done | blocked | E2-S5, E4-S4 |
| [E2-S6-T1](tasks/E2-S6-T1-implement-safe-task-unarchive.md) | done | blocked | E2-S5-T1, E4-S4-T1 |

## E3 — Status workflow

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| [E3-S1](epics/E3-S1-use-default-statuses.md) | done | done | E1-S1 |
| [E3-S1-T1](tasks/E3-S1-T1-implement-default-status-codes.md) | done | done (`4d8ebb2`) | E1-S1-T1 |
| E3-S2 Create and organize custom statuses | needed | not_planned | E3-S1 |
| E3-S3 Change status completion semantics | needed | not_planned | E3-S2, E4-S3 |
| E3-S4 Delete an unused custom status | needed | not_planned | E3-S2 |

## E4 — Hierarchy, dependencies, and availability

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E4-S1 Manage task hierarchy | needed | not_planned | E2-S1 |
| [E4-S2](epics/E4-S2-manage-dependencies.md) | done | done | E2-S1 |
| [E4-S2-T1](tasks/E4-S2-T1-implement-dependency-management.md) | done | done | E2-S1-T1, E2-S2-T1 |
| E4-S3 Query available tasks | needed | not_planned | E3-S1, E4-S2, E5-S1 |
| E4-S4 Explain blocking | needed | not_planned | E4-S2, E4-S3 |
| E4-S5 View relationship maps | needed | not_planned | E4-S1, E4-S2 |

## E5 — Multi-agent claiming

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E5-S1 Claim a specified task atomically | needed | not_planned | E1-S3, E2-S1, E3-S1, E4-S2 |
| E5-S2 Claim the next available task atomically | needed | not_planned | E4-S3, E5-S1 |
| E5-S3 Unclaim owned work | needed | not_planned | E5-S1 |
| E5-S4 Force-unclaim stale work | needed | not_planned | E5-S1 |
| E5-S5 Preserve claim/status independence | needed | not_planned | E3-S1, E5-S1, E5-S3, E2-S5 |

## E6 — Task comments and collaboration context

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E6-S1 Add and view comments | needed | not_planned | E1-S3, E2-S1 |
| E6-S2 Delete a comment under ownership rules | needed | not_planned | E6-S1 |
| E6-S3 Keep comments immutable | needed | not_planned | E6-S1, E6-S2 |

## E7 — Agent-first CLI

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E7-S1 Provide consistent command output | needed | not_planned | E1-S1 |
| E7-S2 Expose repository, agent, and task operations | needed | not_planned | E1, E2, E7-S1 |
| E7-S3 Expose planning relationships and status operations | needed | not_planned | E3, E4, E7-S1 |
| E7-S4 Expose availability and claim operations | needed | not_planned | E4-S3, E4-S4, E5, E7-S1 |
| E7-S5 Expose comment operations | needed | not_planned | E6, E7-S1 |

## E8 — Visual Studio Code scrum experience

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E8-S1 Connect the extension to repository state | needed | not_planned | E1-S2, E7-S1 |
| E8-S2 View and filter the scrum board | needed | not_planned | E2-S2, E3-S2, E4-S3, E5-S1, E8-S1 |
| E8-S3 Change status using the board | needed | not_planned | E3-S3, E8-S2 |
| E8-S4 Manage task details | needed | not_planned | E2, E4-S1, E4-S2, E8-S1 |
| E8-S5 Manage claims from the UI | needed | not_planned | E5, E8-S4 |
| E8-S6 Manage comments from the UI | needed | not_planned | E6, E8-S4 |
| E8-S7 Explore relationship maps | needed | not_planned | E4-S5, E8-S4 |
| E8-S8 Configure statuses and inspect agents | needed | not_planned | E1-S4, E3, E8-S1 |

## E9 — Reliability, verification, and product guidance

| Story | Planning | Implementation | Depends on |
|---|---|---|---|
| E9-S1 Verify concurrent claim safety | needed | not_planned | E1-S5, E5-S1, E5-S2 |
| E9-S2 Verify graph and availability invariants | needed | not_planned | E3-S3, E4, E2-S5, E2-S6 |
| E9-S3 Verify data recovery behavior | needed | not_planned | E1-S1, E4-S2, E5-S1 |
| E9-S4 Document agent and human workflows | needed | not_planned | E7, E8 |
