---
id: E9-S3
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E1-S1
  - E4-S2
  - E5-S1
---

# E9-S3: Verify data recovery behavior

Approved on 2026-10-07 after user confirmation of the shared understanding.

## Outcome

Automated evidence proves interrupted claim and relationship mutations preserve original state, a tested offline procedure preserves recoverable repository data, and existing failure messages identify safe next steps.

## User story

As a user, I want interrupted mutations to remain safe so that local task data is trustworthy.

## Scope and decisions

- Use real CLI process termination after mutation-specific write evidence and before commit for claim acquisition, dependency addition, and parent replacement.
- Add focused post-write rollback evidence for claim acquisition, dependency add/remove, and parent set/replace/remove. Reuse existing validation, no-op, concurrency, and health proofs.
- Validate the bounded hook-free journal gate first. Mere journal existence or size is insufficient. If the gate is unreliable, retain evidence and reopen the decision; weaker evidence cannot replace it.
- Recover through writable SQLite when needed before read-only health inspection. Process-termination safety does not establish power-loss durability or require hot-journal replay.
- Document and execute an offline backup/restore procedure for the complete canonical workspace. Stop clients across all worktrees, preserve matching configuration/database identity, close SQLite before copying, and store durable backups outside the repository.
- Preserve displaced state externally during restoration, refuse destination overwrite, and retain artifacts on failed checks. Never manually delete journals. Test that external backups survive uninstall.
- Verify actionable repository/configuration/database/permission failures against E1-S2 and E7-S1, preserving their codes, exits, streams, and no-repair boundary.
- Retain reproducible production-defect evidence in a separate remediation task. Completion waits for approved fixes and passing verification.

## Acceptance criteria

1. CLI termination cases prove the requested mutation began before termination and could not commit; original relevant rows, complete claim summaries, actor/timestamp metadata, and unrelated state remain intact afterward.
2. Focused rollback tests exercise meaningful post-write failure boundaries for all listed operations and compare original state, rather than only errors.
3. Reopened fixtures pass SQLite integrity and foreign-key checks plus read-only repository health inspection.
4. `docs/recovery.md` gives exact runnable offline backup/restore steps, and automated verification executes those steps against a populated temporary repository.
5. Restoration recovers saved identity, agents, tasks, claims, and relationships after deliberate live-state changes. Linked-worktree access targets the restored canonical store; displaced state and external backups remain recoverable.
6. The failure matrix covers unavailable repository, missing initialization, malformed/unsafe configuration, identity mismatch, missing/non-SQLite/corrupt database, and genuine permission denial. Evidence verifies classification, affected check/path where available, useful recovery direction, human/JSON streams, and no repair or creation.
7. Tests are bounded, deterministic, network-free, and temporary-repository based. Child termination/reaping and reader-lock cleanup work on failures. Unsupported platform evidence is recorded unavailable, never passed.
8. The final acceptance-to-test matrix records executed platform evidence, focused results, and passing format, lint, and workspace tests. Unreliable interruption gating or unresolved production defects block completion.

## Out of scope

Production behavior changes, new commands, schema/dependency changes, production hooks, automatic repair, live-writer backup snapshots, power-loss durability, guaranteed hot-journal replay, E9-S2 work, E8 implementation, and E9-S4 workflow-guidance planning.

## Implementation task

[E9-S3-T1: Verify interruption and offline recovery](../tasks/E9-S3-T1-verify-interruption-and-offline-recovery.md) owns exact proof details, dependencies, and verification commands.

## Planning review

Contract dependencies remain exactly those in the PRD and dashboard. User confirmed the shared understanding on 2026-10-07; implementation completed with the E9-S3-T1 evidence and required checks on 2026-10-07.

No new domain term or hard-to-reverse architectural trade-off requires a glossary entry or ADR.
