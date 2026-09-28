---
id: TD-0007
status: accepted
date: 2026-09-28
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Keep Plan coordination and packages independently loadable

## Context

Plan needs shared lifecycle behavior plus three distinct planning outputs. H1-T18A
requires nested on-demand loading to avoid introducing unrelated planning
instructions into the active context.

## Decision

Install `rule-plan` as an on-demand coordinator and install independent
on-demand rules for Epic, Story, and Implementation Task Planning. The
coordinator selects a subprocess by stable ID. Story Planning loads Task
Planning only for one cohesive implementation unit; no subprocess uses an
extension relationship with the coordinator or a sibling.

## Observable behavior

Repository agents can load only the selected Plan rule family. A Plan package
requires explicit artifact-generation confirmation and proceeds to Commitment
Gate without authorizing execution. The `tbtm` CLI and its public output do
not change.

## Consequences

The pinned 1.10.0 snapshot adds four framework workflow rules while keeping
validation and deterministic lookup unchanged. Each planning level retains its
own templates and stop conditions.

## Acceptance scenario impact

`none`: this changes repository-harness behavior and is covered by the
Constitution suite without changing a product user workflow.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T18](../tasks/H1-T18-plan.md)
- [TD-0005](lazy-workflow-loading-TD-0005.md)
- [TD-0006](staged-core-workflow-loading-TD-0006.md)
