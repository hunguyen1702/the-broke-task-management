# Set the guidance scope around unfinished VS Code work

Type: grilling
Status: resolved
Parent: [E9 reliability and guidance implementation route](../map.md)
Blocked by: none

## Question

Should this effort prepare graph/availability and recovery verification while deferring full workflow guidance until E8, deliver CLI guidance early while keeping VS Code guidance blocked, or include E8 prerequisites in the destination?

The PRD and dashboard require all eight E8 stories before full workflow-guidance planning. They are currently unplanned. Recommended route: prepare the two eligible verification stories now and retain the existing guidance dependency gate.

## Comments

- Scope question presented to the user during map charting; answer pending. The user wants implementation, so the resulting route must lead to approved, claimable implementation tasks rather than replace them with decision tickets.
- 2026-10-07 grilling: user selected A, verification first. Prepare E9-S2 and E9-S3 for implementation; defer E9-S4 planning under its existing E8 dependency gate. Do not add early CLI workflow guidance or E8 implementation to this effort. E9-S3 still includes its required safe-backup documentation and actionable failure verification. Final shared-understanding confirmation is pending; issues 02 and 03 retain the detailed verification and remediation decisions.
- 2026-10-07: user confirmed the shared understanding below.

## Answer

Proceed with verification first: prepare approved story contracts and claimable implementation tasks for E9-S2 (graph and availability invariants) and E9-S3 (data recovery). Resolve the detailed proof, test-layer, and production-remediation boundaries through issues 02 and 03 before those plans are approved.

Retain E9-S4's existing dependency gate: full workflow-guidance planning remains deferred until all eight E8 prerequisites have approved, completed planning. Early CLI workflow guidance and E8 implementation are outside this effort. E9-S3 still requires safe-backup documentation and actionable configuration/database failure verification.

This scope choice preserves the PRD and dashboard contracts; it does not approve implementation plans or complete E9-S2/S3. No new domain terms or architectural trade-offs require glossary entries or an ADR.
