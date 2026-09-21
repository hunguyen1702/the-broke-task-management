---
id: TD-0004
status: accepted
date: 2026-09-21
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Enter Intent through effective rules and confirm its interpretation

## Context

The harness needs a reliable entry to Intent without duplicating workflow
logic in `AGENTS.md` or invoking a hook on every conversation turn. Current
Truth needs a specific information need and scope. The user wants to inspect
the agent's interpretation before downstream analysis or action.

## Decision

Keep `AGENTS.md` as a short Constitution lookup pointer. A framework-origin
workflow entry rule triggers Intent for new work and material goal/scope
changes. A separate Intent workflow rule clarifies material ambiguity and
shows a concise Context–Task–Format interpretation for explicit user
confirmation. Only confirmed Intent yields `ready` for Current Truth. During
initial H1 construction, install these rules as a new pinned 1.2.0 snapshot
under the bounded authorization recorded in H1-T1; update provenance, digest,
and index, then validate. General rule upgrade management remains deferred.

## Observable behavior

Agents using the harness show and confirm each new work interpretation before
Current Truth, routing, or execution. Ordinary continuation does not repeat
Intent. `tbtm` product behavior and public CLI output do not change.

## Consequences

Rule lookup is required at entry, but full rule loading is not repeated for
every message. The two workflow rules are discoverable through
`inspect-effective`; an invalid or stale Constitution remains detectable by
the validator. The confirmation is distinct from later commitment approval.

## Acceptance scenario impact

`none`: the change affects repository-agent workflow, not product scenarios.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T1](../tasks/H1-T1-implement-constitution.md)
- [H1-T12](../tasks/H1-T12-intent.md)
- [Workflow entry](../../.harness/workflow/rule-workflow-entry-r1.md)
- [Intent](../../.harness/workflow/rule-intent-r1.md)
