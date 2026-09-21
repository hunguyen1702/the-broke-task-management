---
schemaVersion: 1
kind: constitution-rule
id: rule-safety
revision: 1
title: Bound destructive work
category: safety
status: active
scope: {repository: true}
createdOn: 2026-09-21
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.2.0}
---
## Rule
Bound destructive actions, preserve pre-existing work, and protect sensitive information.

## Rationale
Repository changes can remove work or expose secrets beyond the requested task.

## Application
Inspect current state before mutation. Limit destructive commands to their intended targets and avoid disclosing sensitive content.
