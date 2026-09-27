---
id: TD-0005
status: accepted
date: 2026-09-28
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Load route-specific Constitution workflows on demand

## Context

Every effective framework rule has repository-wide scope, so the former
repository-entry lookup loaded Direct, Research, Explore, and Spike even when
Router had not selected them. Constitution validation must remain exhaustive.

## Decision

Add `contextLoading` to the Constitution schema. Workflow rules declare
`always` or `on_demand`; non-workflow rules derive `always`. Bootstrap uses
`inspect-context`, while Router-selected workflows use
`inspect-workflow <stable-id>`. The derived index records the classification;
`inspect-effective` keeps its complete diagnostic behavior. Loading class must
match across every extension and replacement relationship.

## Observable behavior

Repository agents initially read only the effective core context. A selected
Direct, Research, Explore, or Spike workflow is loaded explicitly, after its
effective replacement chain and path applicability are checked. The `tbtm`
CLI and its public output do not change.

## Consequences

The 1.8.0 pinned framework snapshot keeps all rules subject to validation and
digest checks while reducing unrelated workflow context at startup. Future
nested on-demand workflows can be selected independently without a registry
or semantic routing in the validator.

## Acceptance scenario impact

`none`: this is internal repository-harness behavior covered by the
Constitution suite and does not change a product user workflow.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T18A](../tasks/H1-T18A-lazy-workflow-loading.md)
- [Constitution lookup](../../.harness/README.md)
