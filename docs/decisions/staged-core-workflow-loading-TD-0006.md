---
id: TD-0006
status: accepted
date: 2026-09-28
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Stage Constitution core workflow loading

## Context

Ruleset 1.8.0 made route-specific workflows lazy but still loaded Current Truth
and Risk Router at repository startup. Those core nodes have a strict lifecycle
order and do not need to be available before their predecessor completes.

## Decision

Keep Intent in bootstrap context. Make Current Truth and Risk Router
`on_demand`; Intent looks up Current Truth after confirmed `ready`, and Current
Truth looks up Router only for `ready` or a routable `unresolved`. The validator
continues to provide deterministic stable-ID lookup and does not track lifecycle
state or infer a caller's progress.

## Observable behavior

Repository agents load only bootstrap rules at startup, then add the next core
workflow when its predecessor hands off. `inspect-effective` remains complete.
The `tbtm` CLI and its public output do not change.

## Consequences

The pinned 1.9.0 snapshot reduces unrelated startup context while preserving
validation, replacement, extension, path, and diagnostic behavior. Workflow
rules, rather than the lookup command, own lifecycle ordering.

## Acceptance scenario impact

`none`: this changes only repository-harness behavior, covered by the
Constitution suite, and does not change a product user workflow.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T18B](../tasks/H1-T18B-staged-core-workflow-loading.md)
- [TD-0005](lazy-workflow-loading-TD-0005.md)
- [Constitution lookup](../../.harness/README.md)
