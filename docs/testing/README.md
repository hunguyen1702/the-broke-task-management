# Acceptance regression catalog

## Status

- Catalog state: `approved`
- Approved: 2026-09-04
- Last simplified: 2026-09-04
- Last full execution: 2026-09-07 (88 passed, 0 failed, 0 blocked)
- Latest targeted execution: 2026-09-05, AT-E4-S5-001 passed (4 cases)
- Current result: passed (88 passed, 0 failed, 0 blocked in the latest full run)

This catalog is a concise QA suite for common user workflows. It checks public
CLI inputs, outputs, and likely errors. Exhaustive validation, transactions,
locking, concurrency races, migrations, and internal invariants belong in Rust
unit and integration tests.

Only scenarios with `status: approved` may be executed.

## Scenario structure

Each scenario contains a small setup, a table of input and expected output, and
links to its approved story contract. Directories and filenames use searchable
English names plus epic/story IDs.

## Implemented story coverage

| Story | User workflow | Scenario |
|---|---|---|
| E1-S1 | Initialize and uninstall a repository | [scenario](repository-foundation-and-identity-E1/initialize-and-uninstall-repository-E1-S1-001.md) |
| E1-S2 | Resolve and inspect repository state | [scenario](repository-foundation-and-identity-E1/resolve-and-inspect-repository-E1-S2-001.md) |
| E1-S3 | Register an agent | [scenario](repository-foundation-and-identity-E1/register-distinct-agent-identities-E1-S3-001.md) |
| E1-S4 | List agents and their claims | [scenario](repository-foundation-and-identity-E1/list-agents-and-active-claims-E1-S4-001.md) |
| E1-S5 | Share state across Git worktrees | [scenario](repository-foundation-and-identity-E1/share-state-across-git-worktrees-E1-S5-001.md) |
| E2-S1 | Create a task | [scenario](task-content-and-lifecycle-E2/create-task-with-complete-context-E2-S1-001.md) |
| E2-S2 | View and filter tasks | [scenario](task-content-and-lifecycle-E2/view-filter-and-order-tasks-E2-S2-001.md) |
| E2-S3 | Update task fields | [scenario](task-content-and-lifecycle-E2/update-task-content-atomically-E2-S3-001.md) |
| E2-S4 | Replace structured task context | [scenario](task-content-and-lifecycle-E2/replace-structured-task-context-E2-S4-001.md) |
| E2-S5 | Archive a task | [scenario](task-content-and-lifecycle-E2/archive-task-with-claim-safety-E2-S5-001.md) |
| E2-S6 | Unarchive a task | [scenario](task-content-and-lifecycle-E2/unarchive-task-with-impact-confirmation-E2-S6-001.md) |
| E3-S1 | Use default statuses | [scenario](status-workflow-E3/use-immutable-default-statuses-E3-S1-001.md) |
| E4-S1 | Manage task hierarchy | [scenario](hierarchy-dependencies-and-availability-E4/manage-and-read-task-hierarchy-E4-S1-001.md) |
| E4-S2 | Manage dependencies | [scenario](hierarchy-dependencies-and-availability-E4/manage-cycle-safe-dependencies-E4-S2-001.md) |
| E4-S3 | Find available tasks | [scenario](hierarchy-dependencies-and-availability-E4/query-filtered-available-tasks-E4-S3-001.md) |
| E4-S4 | Explain why a task is blocked | [scenario](hierarchy-dependencies-and-availability-E4/explain-direct-and-recursive-blockers-E4-S4-001.md) |
| E4-S5 | View relationship maps | [scenario](hierarchy-dependencies-and-availability-E4/view-deterministic-relationship-maps-E4-S5-001.md) |
| E5-S1 | Claim a specified task | [scenario](multi-agent-claiming-E5/claim-specified-available-task-E5-S1-001.md) |
| E5-S2 | Claim the next task | [scenario](multi-agent-claiming-E5/claim-next-highest-ranked-task-E5-S2-001.md) |
| E5-S3 | Unclaim owned work | [scenario](multi-agent-claiming-E5/unclaim-only-owned-work-E5-S3-001.md) |
| E5-S4 | Force-unclaim stale work | [scenario](multi-agent-claiming-E5/force-unclaim-observed-stale-work-E5-S4-001.md) |
| E5-S5 | Preserve claim while status changes | [scenario](multi-agent-claiming-E5/preserve-claim-status-independence-E5-S5-001.md) |
| E6-S1 | Add and list comments | [scenario](task-comments-and-collaboration-E6/add-and-list-task-comments-E6-S1-001.md) |

Stories not marked `done` in `docs/STATUS.md` are outside the catalog.
