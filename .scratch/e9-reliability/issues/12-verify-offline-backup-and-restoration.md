# 12: Verify offline backup and restoration

Type: task
Status: resolved
Blocked by: None

**What to build:** Give the human exact runnable offline recovery guidance and execute those same steps to prove complete saved repository state can be restored safely, including access through linked worktrees. This is a slice of approved E9-S3-T1.

**Blocked by:** None (can start immediately). Existing authoritative implementation prerequisites are done; the procedure does not depend on the interruption harness.

- [ ] Document and verify the same exact runnable steps, rather than testing an independent copy helper. Stop all clients/writers across main and linked worktrees, resolve the canonical workspace, establish configuration/database identity and health, recover existing SQLite writable where needed, and close all connections before copying.
- [ ] Copy the complete matching canonical workspace to a new durable destination outside the repository. Refuse existing destination overwrite and avoid repository-root backup/staging locations removed by uninstall. Never manually delete journals.
- [ ] If health or canonical identity cannot be established, stop the normal procedure and preserve existing artifacts without force-initialization, replacement, or automatic repair.
- [ ] Populate a temporary fixture with representative agents, tasks, claims, and relationships. Deliberately change live data after backup, then restore matching configuration and database together while clients remain stopped.
- [ ] Preserve displaced live state externally and refuse overwrite of backup/displaced destinations. On failed checks, retain saved and displaced state and stop.
- [ ] Verify restored identity and representative saved data, SQLite integrity and foreign keys, repository health, and linked-worktree access to the restored canonical store.
- [ ] Prove external backups survive actual uninstall in the disposable fixture. Keep test artifacts temporary; demonstrate a durable external destination in user guidance.
- [ ] Record backup/restore acceptance evidence in the E9-S3 matrix; tests are bounded, deterministic, offline, and isolated. Run the authoritative focused checks and pass format, lint, and workspace gates.
- [ ] Preserve reproducible production defects for separate approved remediation. Follow E9-S3 claim rules; no new commands, dependencies, schema, live-writer snapshot promises, or automatic repair are introduced. This slice alone cannot complete E9-S3.

## Comments

- 2026-10-07: User confirmed this independent end-to-end recovery slice.

## Answer

`docs/recovery.md` gives runnable offline backup and restore steps. `crates/tbtm-cli/tests/recovery_backup_restore.rs::documented_offline_backup_restores_saved_state_and_survives_uninstall` extracts and executes that script against populated data, changed live state, a linked worktree, actual uninstall, and overwrite refusal. Focused test, format, lint, and workspace tests passed on Darwin. Aggregate E9-S3 remains incomplete because ticket 10's interruption gate is blocked.
