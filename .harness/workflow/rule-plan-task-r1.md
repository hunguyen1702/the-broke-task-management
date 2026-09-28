---
schemaVersion: 1
kind: constitution-rule
id: rule-plan-task
revision: 1
title: Plan one implementation task
category: workflow
status: active
scope: {repository: true}
contextLoading: on_demand
createdOn: 2026-09-28
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.10.0}
---
## Rule
Produce one actionable technical specification only for an approved story whose observable behavior is fixed and whose work is one implementation unit. Return a mis-scoped unit to Plan classification and behavior changes to Story Planning or user authority.

## Rationale
Implementation needs concrete technical guidance without a line-by-line patch or a second source of product behavior.

## Application
**Input check:** confirm an approved parent story, one implementation unit, source freshness, destination, and artifact-generation confirmation. **Procedure and template:** write a specification that links its parent story and states objective, deliverables, evidence-backed structure, technical choices and boundaries, coding-agent implementation flow, applicable state, persistence, concurrency, output and error contracts, test plan, verification commands, references, and definition of done. Check each choice against the story and retain sources for consequential constraints.

**Verification checklist:** confirm every deliverable and technical choice satisfies the parent contract and every required verification command is named. **Stop:** do not revise observable story behavior, recursively decompose multiple outcomes, claim execution approval, or define Commitment Gate. Flag missing facts, feasibility assumptions, blocking choices, or scope drift precisely for the owning upstream boundary.

**Bad:** prescribe a patch that changes the parent story's output. **Good:** specify the technical path that satisfies its existing output contract. **Bad:** split a multi-outcome task into hidden child tasks. **Good:** return it to Plan classification for a Story or Epic boundary.
