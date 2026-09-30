---
id: TD-0008
status: accepted
date: 2026-09-29
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Defer the H1 Commitment Gate

## Context

The approved Plan workflow already requires observable acceptance criteria and
verification in its reviewed package. A separate Commitment Gate was planned
between Plan and Execute, but its additional decision was not established.

## Decision

Remove H1-T19 Commitment Gate from the active H1 lifecycle and task board.
The target path is Plan → Execute. Preserve H1-T19 in the deferred-work
register so it can be reconsidered only after a new user decision.

This decision updates the target architecture, not the installed Constitution
snapshot. A later implementation task must align the versioned Router and Plan
rules before agents can rely on the new path at runtime.

## Observable behavior

The intended repository-agent flow no longer requires a separate gate after a
reviewed Plan package. Existing user-owned authority and action-specific
approval boundaries still apply. The `tbtm` CLI contract does not change.

## Consequences

H1-T20 Execute becomes the next active node to plan. Historical H1-T0,
H1-T13, and H1-T18 contracts continue to record their original decisions;
the H1 epic records the revised target flow. The installed rules retain the
earlier handoff until a versioned implementation aligns them.

## Acceptance scenario impact

`none`: this is an H1 planning decision, with no implemented product behavior.

## References

- [H1 target lifecycle](../epics/H1-build-adaptive-repository-harness.md)
- [H1 status](../STATUS.md)
- [Deferred-work register](../handoff/H1-deferred-work.md)
- [Prior Plan decision](plan-workflow-package-TD-0007.md)
