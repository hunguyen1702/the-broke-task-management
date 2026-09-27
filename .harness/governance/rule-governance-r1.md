---
schemaVersion: 1
kind: constitution-rule
id: rule-governance
revision: 1
title: Maintain bounded Constitution rules
category: governance
status: active
scope: {repository: true}
createdOn: 2026-09-21
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.6.0}
---
## Rule
Write short, scoped rules in this format. Keep all categories directly under `.harness/`; rule origin is recorded in frontmatter. Agents may propose project rules from repository evidence but may not activate them. Rebuild the index when canonical rules change and validate every Constitution change.

## Rationale
Canonical rules remain readable while the index remains a reliable projection.

## Application
Create proposals as drafts in the applicable `.harness/<category>/` directory with `origin: project`. Keep the three required sections and run the validator after rebuilding the index.
