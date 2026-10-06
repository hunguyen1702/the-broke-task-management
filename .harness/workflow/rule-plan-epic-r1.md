---
schemaVersion: 1
kind: constitution-rule
id: rule-plan-epic
revision: 1
title: Plan an epic package
category: workflow
status: active
scope: {repository: true}
contextLoading: on_demand
createdOn: 2026-09-28
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.13.0}
---
## Rule
Produce an Epic package only when confirmed work has multiple independently deliverable outcomes, multiple subsystems or user journeys, or no cohesive implementation and verification boundary. Preserve blocking uncertainty for its owner; do not decide it during decomposition.

## Rationale
Epic planning establishes product and technical boundaries while leaving child outcomes available for later bounded planning.

## Application
**Input check:** confirm the Epic route, source freshness, output destination, and artifact-generation confirmation. **Procedure and template:** write an epic contract with outcome, context, actors, scope, exclusions, settled product decisions, constraints, invariants, success criteria, and labelled deferred questions; a separate epic technical design covering architecture, boundaries, data or control flow, integrations, migration concerns, risks, and verification; and a future-planning task board. Each board row has stable ID, title, outcome, scope summary, and planning state only. Link the artifacts and check that every consequential decision has a source.

**Verification checklist:** confirm the three artifacts link together, every settled constraint has provenance, and the board contains no child detail beyond its stated fields. **Stop:** do not add detailed child-story contracts, task specifications, execution state, owners, claims, or a fixed child count. A blocking product or architecture choice returns to Router or user authority; only explicitly deferred non-blocking questions may remain labelled.

**Bad:** create detailed stories for every board row. **Good:** record future planning units and let later Story Planning bound each outcome. **Bad:** fold cross-cutting architecture into an epic contract. **Good:** link a distinct technical design that does not design every child.
