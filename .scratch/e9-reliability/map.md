# E9 reliability and guidance implementation route

Label: wayfinder:map

## Destination

Track implementation of E9-S2 graph/availability verification and E9-S3 recovery verification in [the PRD](../../docs/PRD.md#epic-e9-reliability-verification-and-product-guidance). Retain the existing E8 dependency gate for E9-S4 workflow-guidance planning.

## Notes

- Grilling tickets 01–03 and implementation tickets 09–14 are resolved. The [combined implementation spec](spec.md) is implemented. The prematurely generated scope spec and tickets 04–07 were removed on 2026-10-07.
- Requested outcome: implement E9. This map prepares the remaining decisions; implementation claims and approved contracts belong in `docs/epics/`, `docs/tasks/`, and `docs/STATUS.md`.
- Consult wayfinder, grilling, and domain-modeling when resolving decisions. Use the repository's local Markdown tracker and root glossary.
- Concurrent claim verification is already implemented: [Verify concurrent claim safety](../../docs/epics/E9-S1-verify-concurrent-claim-safety.md).
- Graph/availability and recovery verification are implemented with focused and workspace evidence recorded in their authoritative tasks.
- Full workflow guidance depends on all eight unfinished VS Code stories and remains deferred. Early CLI workflow guidance and E8 implementation are outside this effort.
- Scope ticket 01, graph/availability ticket 02, and recovery ticket 03 were resolved with user confirmation. E9-S2-T1 and E9-S3-T1 have passing implementation evidence.
- Keep verification deterministic, temporary-repository based, and bounded. Reuse existing behavior and tests before adding coverage. Existing contracts remain authoritative for public behavior.
- A discovered defect needs an explicit remediation boundary before a verification task broadens into production changes.

## Decisions so far

- [Relationship interruption and rollback evidence](issues/11-verify-interrupted-relationship-mutations.md#answer) passed on Darwin, completing the five-ticket verification graph and final integration gates.

- [Validated claim interruption gate](issues/10-verify-interrupted-claim-acquisition.md#answer) passed after remediation 14.

- [No-op migration remediation](issues/14-skip-no-op-migration-transaction.md#answer) was approved on 2026-10-07 after [claim interruption feasibility evidence](../../docs/testing/E9-S3-claim-interruption-feasibility.md) showed the old CLI could not reach the requested mutation under the reader-lock gate. The gate passed after remediation.

- [Offline backup and restore implementation](issues/12-verify-offline-backup-and-restoration.md#answer) passed through the documented procedure on Darwin; [repository-failure evidence](issues/13-verify-actionable-repository-failures.md#answer) passed with genuine local permission denial.

- [Graph and availability implementation](issues/09-verify-graph-and-availability-transitions.md#answer): E9-S2 verification passed on Darwin; E9-S2 and E9-S2-T1 are done.

- [Combined E9-S2/S3 verification spec](spec.md) published and implemented on 2026-10-07 from the confirmed scope and test seams; E9-S4 retains its E8 planning gate.

- [Confirmed E9-S3 recovery boundary](issues/03-decide-recovery-verification.md#answer): bounded real-CLI interruption evidence plus focused rollback proofs, tested offline backup/restore, and actionable failures; unreliable gating or production defects block completion. The [approved story](../../docs/epics/E9-S3-verify-data-recovery-behavior.md) and [ready task](../../docs/tasks/E9-S3-T1-verify-interruption-and-offline-recovery.md) record exact dependencies and commands.

- [Confirmed E9-S2 verification boundary](issues/02-decide-invariant-verification.md#answer): one CLI journey, a focused ordering test, and one verification-only task; reuse existing concurrency/worktree evidence. The [approved story](../../docs/epics/E9-S2-verify-graph-and-availability-invariants.md) and [ready task](../../docs/tasks/E9-S2-T1-verify-graph-and-availability-transitions.md) record the matrix, exact dependencies, and commands.
- E9-S2 proof scope is agreed: reuse existing proofs and close the cross-operation availability, allowed completion exception, exact claim-preservation, and final ordering-tie gaps. [Scope recorded in ticket 02](issues/02-decide-invariant-verification.md#answer).
- E9-S2 production defects require retained reproducible evidence and a separate remediation task; completion waits for the fix and passing verification. [Agreement recorded in ticket 02](issues/02-decide-invariant-verification.md#comments).
- [Completion precondition deferred](issues/08-decide-completion-dependency-precondition.md): E9-S2 continues under current contracts. Whether marking a task completed should require effectively completed direct upstream tasks is a separate open decision and does not block E9-S2.
- [Confirmed guidance scope](issues/01-set-guidance-scope.md#answer): verification first (A), preparing E9-S2/S3 for implementation while retaining E9-S4's existing E8 planning gate. E9-S3 includes safe-backup documentation; early CLI workflow guidance and E8 implementation are outside this effort.

## Not yet specified

- [Ticket 08](issues/08-decide-completion-dependency-precondition.md) retains the unresolved completion-precondition question outside the E9 verification scope.
- Production remediation questions may emerge from the verification audit; retain reproducible evidence before deciding their scope.

## Out of scope

New product semantics, remote coordination, automatic repair, and performance service levels are beyond the existing E9 requirements.
