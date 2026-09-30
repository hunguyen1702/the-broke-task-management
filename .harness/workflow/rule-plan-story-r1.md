---
schemaVersion: 1
kind: constitution-rule
id: rule-plan-story
revision: 1
title: Plan a bounded story package
category: workflow
status: active
scope: {repository: true}
contextLoading: on_demand
createdOn: 2026-09-28
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.11.0}
---
## Rule
Produce a Story package for one observable outcome with a bounded product scope and credible cohesive acceptance boundary. It may contain several technical units without becoming an Epic.

## Rationale
The story contract owns observable behavior while technical planning stays subordinate to that behavior and can be deferred when several units are needed.

## Application
**Input check:** confirm one observable outcome, the parent scope, source freshness, destination, and artifact-generation confirmation. **Procedure and template:** write a story contract with actor intent, outcome, scope, exclusions, decisions, interfaces and behavior, functional and non-functional requirements, observable acceptance criteria, verification, definition of done, and links to implementation planning units.

For one cohesive unit, load `rule-plan-task` through nested on-demand lookup and link the paired specification. For several units, write a story technical design with shared boundaries, interactions, constraints, risks, and verification, plus an implementation-planning board. Each board row contains stable ID, title, bounded objective or scope, `needed` or `planned` state, an optional document link, and only required ordering or dependency. **Verification checklist:** confirm acceptance and verification are observable, all paired documents link, and technical units do not expand the story. **Stop:** do not write detailed task specifications eagerly, add execution state or owners, expand epic scope, or resolve a user-owned behavior decision.

**Bad:** call multiple technical units multiple product outcomes. **Good:** keep one Story and defer their Task Planning packages. **Bad:** let a task design alter acceptance criteria. **Good:** return the behavior change to Story Planning or user authority.
