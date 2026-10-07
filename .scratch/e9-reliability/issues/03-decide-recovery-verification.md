# Decide interruption proof and the safe backup procedure

Type: grilling
Status: resolved
Parent: [E9 reliability and guidance implementation route](../map.md)
Blocked by: 01

## Question

How should recovery verification prove interrupted claim and relationship mutations leave no partial state, document a safe backup/restore procedure, and establish actionable failures without adding automatic repair or a new public backup command?

## Context

- [Data recovery requirements](../../../docs/PRD.md#story-e9-s3-verify-data-recovery-behavior).
- Claim, dependency, and hierarchy writes are owned by core SQLite transactions. Distinguish deterministic transaction rollback evidence from process-termination evidence; neither a sleep nor killing before a command starts proves interruption inside a mutation.
- Audit existing rollback tests and repository-health diagnostics before choosing new harness work or production hooks.
- [Repository configuration resolution](../../../docs/epics/E1-S2-resolve-repository-configuration.md) classifies unavailable repositories, missing initialization, invalid configuration, database failures, and permission failures; health inspection is read-only and does not repair.
- A backup must preserve matching configuration and database identity, account for SQLite journals and active writers, and target the canonical main-worktree store. Choose between an offline procedure and SQLite-supported snapshotting based on safety and maintenance cost.
- Forced-initialization backups are a different operation; uninstall explicitly removes matching backup directories. Account for that behavior when deciding where durable backups live.

## Resolution criterion

Agree on deterministic interruption evidence, a tested backup/restore procedure, the failure matrix, and the narrow story/task boundary. Produce an approved story/task plan with exact dependencies and verification commands; update its authoritative frontmatter and dashboard together.

## Comments

- Planning publication checks passed on 2026-10-07: exact story/task/dashboard metadata and dependency matching, completed prerequisites, resolved-ticket state, and local-link validation; `rtk git diff --check`; `rtk mise run format`; `rtk mise run lint`; `rtk mise run test`. These establish the current planning handoff baseline, not the new E9-S3 recovery acceptance evidence.

- 2026-10-07: user confirmed the complete shared understanding. Published the approved E9-S3 story and E9-S3-T1 implementation task, with planning `done` and implementation `ready`, synchronized with the dashboard. Implementation remains unclaimed and unstarted. Earlier pending comments below record interview history.

- Complete proposed story/task boundary, dependencies, proof matrix, backup/restore safety, failure matrix, and verification commands are recorded in [the final-confirmation draft](../recovery-plan-review.md). No implementation has started; final confirmation will authorize publishing approved story/task metadata and resolving this ticket.

- 2026-10-07: user accepted the second-round recommendations: CLI interruption for claim acquisition, dependency addition, and parent replacement; focused rollback coverage for claim acquisition, dependency add/remove, and parent set/replace/remove; validate bounded mutation-specific journal evidence first and reopen the design decision if unreliable, without production hooks or weaker substitutes; no guaranteed hot-journal replay requirement. Offline backup copies the complete recovered/closed canonical workspace externally; offline restoration preserves displaced state externally and verifies identity, fixture data, integrity, and foreign keys without manually deleting journals. Failure matrix covers unavailable repository, missing initialization, malformed/unsafe configuration, identity mismatch, missing/non-SQLite/corrupt database, and actual permission denial, with error classification, actionable direction, streams, and no-repair evidence. Final shared-understanding confirmation remains pending.

- 2026-10-07: user accepted all first-round recommendations: actual CLI process termination after writes begin and before commit plus focused rollback proofs where missing; exclude power-loss durability; tested offline backup/restore; retain reproducible production-defect evidence in a separate remediation task and block E9-S3 completion pending a fix and passing verification.
- Audit refinement: claim/dependency/hierarchy writes use immediate transactions, but current tests lack injected post-write rollback coverage. A populated rollback journal must be attributable to the requested mutation; mere existence/size does not prove the boundary, and interruption of a small cached transaction does not necessarily prove hot-journal replay. Validate the gate without production hooks; writable SQLite reopening must precede read-only health inspection when recovery is needed. Permission fixtures require actual denied access; unsupported platform evidence must be reported unavailable, never passed.

- 2026-10-07: planning claimed. First grilling round awaits decisions on actual CLI process-termination evidence versus rollback-only evidence, offline backup versus live SQLite snapshotting, and separate production-defect remediation. Recommendations: process termination plus reused rollback proofs (excluding power-loss claims), a tested offline backup/restore procedure outside uninstall's deletion scope, and separate remediation that blocks story completion. A read-only audit is checking existing proof ownership and harness feasibility; no implementation or approval is implied.

- Read-only audit found no process-interruption tests or executable safe-backup/restore procedure. Existing repository-health tests cover malformed config and unavailable databases; actionable error suggestions already exist in core.
- Claim and dependency mutations in `crates/tbtm-core/src/task.rs` already hold immediate transactions through mutation, metadata changes, response hydration, and commit. Initialization in `crates/tbtm-core/src/lib.rs` selects DELETE journaling and FULL synchronous mode.
- Candidate hook-free process-interruption proof: hold a parent read transaction on a fully migrated fixture, invoke the real CLI child, wait with a bounded deadline for a populated rollback journal while the reader blocks commit, kill the child before releasing the reader, then reopen and check original rows/metadata, integrity, and foreign keys. Validate that journal evidence belongs to the requested mutation. This approach remains unimplemented and requires harness validation; it does not prove power-loss durability.

## Answer

Approve [E9-S3](../../../docs/epics/E9-S3-verify-data-recovery-behavior.md) and one [implementation task](../../../docs/tasks/E9-S3-T1-verify-interruption-and-offline-recovery.md). The task owns the exact proof matrix, implementation dependencies, safe offline procedure requirements, failure classifications, and verification commands.

Require real CLI pre-commit termination evidence for claim acquisition, dependency addition, and parent replacement; focused post-write rollback tests cover claim acquisition, dependency add/remove, and parent set/replace/remove. Validate the bounded mutation-specific journal gate first. If unreliable, retain feasibility evidence and reopen the decision rather than weakening proof or adding production hooks. Reopen SQLite writable for recovery when needed before read-only health inspection; require original state, integrity, and foreign-key evidence. Power-loss durability and guaranteed hot-journal replay are excluded.

Document and test offline backup/restore of the complete canonical workspace with matching configuration/database identity, stopped clients across worktrees, closed/recovered SQLite, external durable backups, preserved displaced state, overwrite refusal, and post-restore identity/data/health checks. External backups must survive uninstall; never delete journals manually.

Verify actionable unavailable-repository, missing-initialization, malformed/unsafe-config, identity-mismatch, missing/non-SQLite/corrupt-database, and genuine permission-denial failures. Reuse current evidence and close only gaps in codes/exits, affected checks/paths, useful next steps, streams, and no-repair assertions. Report unsupported platform evidence unavailable.

Production defects require separate remediation with reproducible evidence and block completion until passing verification. All implementation dependencies are done; planning is `done`, implementation is `ready`, unclaimed and unstarted. No glossary change or ADR is needed.
