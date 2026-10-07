---
id: E9-S3-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S1-T1
  - E1-S2-T1
  - E1-S3-T1
  - E1-S5-T1
  - E2-S1-T1
  - E4-S1-T1
  - E4-S2-T1
  - E5-S1-T1
  - E7-S1-T1
---

# E9-S3-T1: Verify interruption and offline recovery

Approved on 2026-10-07 after user confirmation of the shared understanding.

## Objective and scope

Implement the proof matrix and tested offline procedure for [E9-S3](../epics/E9-S3-verify-data-recovery-behavior.md). This task owns test-only recovery evidence and `docs/recovery.md`. Existing public contracts remain authoritative.

Production defects require separate remediation. Production hooks, commands, migrations, dependencies, automatic repair, power-loss durability, and guaranteed hot-journal replay are outside scope.

## Proof matrix

| Requirement | Proof ownership |
|---|---|
| Interrupted claim acquisition | New real-CLI `recovery_interruption.rs` case; exact original claims, task metadata, and unrelated state |
| Interrupted dependency addition | Same CLI harness; original dependency rows and downstream metadata |
| Interrupted parent replacement | Same CLI harness; original parent edge and child metadata |
| Post-write failures roll back | Focused core tests beside existing behavior, grouped with `recovery_rollback` in their names: claim acquisition, dependency add/remove, parent set/replace/remove |
| Safe offline backup/restore | New `recovery_backup_restore.rs`; executable steps in `docs/recovery.md`, verified with a populated temporary repository and linked-worktree canonical resolution |
| Actionable failures without repair | Reuse `repo_status.rs` and core health tests; extend only missing classifications, guidance, stream, and unchanged-state assertions |

## Implementation evidence (Darwin, 2026-10-07)

| Stories | Status | Test or evidence |
|---|---|---|
| 17: real CLI interrupted claim | Passed | `recovery_interruption::killed_claim_writer_preserves_original_claims_and_repository_health` passed after approved remediation 14. Historical feasibility evidence remains in `docs/testing/E9-S3-claim-interruption-feasibility.md`. |
| 18–19: real CLI interrupted dependency add and parent replacement | Passed | `recovery_interruption::killed_dependency_add_writer_preserves_original_edges_and_repository_health`, `killed_parent_replace_writer_preserves_original_edge_and_repository_health`. Both compare real existing claim owner and timestamp after recovery. |
| 20: post-write rollback | Passed | `tbtm-core::recovery_rollback_claim_post_write_failure_preserves_claims_and_tasks`, `recovery_rollback_relationship_post_write_failure_preserves_original_state` (dependency add/remove and parent set/replace/remove). |
| 21: recovered integrity, foreign keys, health | Passed | All three `recovery_interruption` cases reopen the same database writable, then pass integrity, foreign-key, and `repo status` checks; the backup/restore case passes the same checks. |
| 22–26: offline backup, external survival, displaced-state preservation, saved data, linked worktree | Passed | `recovery_backup_restore::documented_offline_backup_restores_saved_state_and_survives_uninstall` executes the exact `docs/recovery.md` script. |
| 27–28: actionable classified failures, codes, streams, unchanged data | Passed | `repo_status::repository_failures_give_recovery_steps_without_repairing_data`, `repo_status_unreadable_database_exits_five`, and reused `repo_status`/core health cases. |
| 29: mutation-specific before-commit proof | Passed | Each `recovery_interruption` case gates on a marker in the rollback journal page of its requested mutation's table, with a three-second deadline below the five-second busy timeout. |
| 30: unsupported platforms and unreliable gating | Recorded | Darwin permission denial and claim interruption passed; Windows ACL and interruption behavior are unavailable on this host. |
| 31: production defects retained separately | Passed | The no-op migration lock was retained as reproducible evidence and remediated separately in approved ticket 14. No unresolved production-contract violation remains. |

Passing focused results at final integration state: `recovery_interruption`: 3, `tbtm-core recovery_rollback`: 2, `recovery_backup_restore` + `repo_status`: 10, core quick-check: 1, and reused claim/dependency/hierarchy suites: 18. `rtk mise run format`, `rtk mise run lint`, and `rtk mise run test` passed on Darwin.

Core rollback tests exercise public core operations using test-owned SQLite failure fixtures, such as triggers failing after an earlier write. Cover meaningful partial-write boundaries and post-write result hydration where feasible without production hooks; compare original rows/metadata, not just returned errors. Reuse validation/no-op proofs rather than duplicating them.

## Interruption gate and acceptance

Validate the hook-free harness before extending the matrix. Use a fully migrated isolated fixture, establish a parent read transaction that actually holds a SHARED lock, and start one real CLI writer. Require journal evidence attributable to the requested mutation before its commit; journal existence or size alone is insufficient. Use a deadline shorter than the writer's busy timeout, detect premature exit, capture command diagnostics, and always kill/reap the writer before releasing the reader, including failure cleanup. No sleep-based timing assumption.

After termination, reopen the existing database writable without creating a replacement, allowing SQLite recovery where needed. Then compare original relevant rows, complete claim summaries, actor/timestamp metadata and unrelated state; require `integrity_check` success, empty `foreign_key_check`, and healthy read-only `repo status`. Do not require database byte equality after recovery or claim proof of power-loss durability or hot-journal replay.

If mutation-specific gating cannot be established reliably, retain feasibility evidence and reopen the design decision. The task remains blocked on that decision; it cannot substitute rollback-only evidence or introduce a production hook silently.

## Offline backup and restoration

The implementation must provide exact runnable steps and verify those same steps, rather than merely test an independent copy helper. Stop every TBTM client/writer across main and linked worktrees. Resolve the canonical workspace, open the existing database writable if journal recovery is needed, check it, then close all SQLite connections. If health/canonical identity cannot be established, stop the normal procedure and preserve existing artifacts; do not overwrite or force-initialize them.

Copy the complete canonical `.tbtm` directory, including matching configuration and database, to a new durable destination outside the repository. Do not use root `.tbtm.backup-*`/`.tbtm.staging-*` locations: uninstall deletes them. Never manually delete journal files. External backups must survive actual uninstall in the disposable test fixture.

Restore only while clients remain stopped. Preserve the displaced workspace externally; refuse accidental overwrite of an existing backup/displaced destination. Restore the complete matching workspace, not the database alone. Verify configuration/database identity, repository health, representative agents/tasks/claims/relationships, SQLite integrity and foreign keys. Deliberately change live fixture data between backup and restore to prove the saved state is restored; verify access through a linked worktree targets that restored canonical store. Stop on failed checks and retain both saved and displaced state.

## Failure matrix

Cover unavailable repository (exit 1), missing initialization (3), malformed/unsafe configuration and config/database identity mismatch (2), missing/non-SQLite/corrupt database (1), and permission denial (5). Stable codes come from E1-S2, including `PERMISSION_DENIED` for access denial. Require affected check/path where available, useful recovery direction, JSON error on stdout with empty stderr, human error/guidance on stderr, and unchanged repository data with no creation, migration, or repair.

Reuse current platform-specific permission fixtures only where denial is real; root-bypassed fixtures cannot pass. Report unsupported platform evidence as unavailable, distinct from passing, failing, or blocked. The final matrix states the executed platform and any unexecuted platform checks.

## Defect boundary

Preserve minimal reproducible evidence of any existing production-contract violation. Create a separate remediation task; do not fix production under E9-S3-T1. Story completion waits for approved remediation and passing verification at the resulting repository state.

## Verification commands

New targets and filter become available during implementation. Tests must exist and execute; a zero-test filter is not acceptance evidence.

```bash
rtk cargo test -p tbtm --test recovery_interruption
rtk cargo test -p tbtm-core recovery_rollback
rtk cargo test -p tbtm --test recovery_backup_restore --test repo_status
rtk cargo test -p tbtm-core failed_quick_check_is_read_only_and_database_unavailable
rtk cargo test -p tbtm --test task_claim --test task_dependency --test task_hierarchy
rtk mise run format
rtk mise run lint
rtk mise run test
```

Implementation completion requires an acceptance-to-test matrix, successful backup/restore execution, bounded deterministic interruption proofs, focused rollback and failure evidence, passing required gates, and synchronized story/task/dashboard metadata. Planning approval is not acceptance proof.

## Acceptance impact

`none`: verification and recovery guidance under existing contracts; no public behavior or acceptance-scenario catalog change.

## Dependencies and approval

All listed implementation prerequisites are done. They supply workspace lifecycle, health resolution, agent/task fixtures, canonical worktree storage, hierarchy/dependency mutations, claim acquisition, and output conventions. E9-S2 is not a dependency.

Planning and implementation are `done`. Final focused checks, format, lint, and workspace tests passed on Darwin on 2026-10-07.
