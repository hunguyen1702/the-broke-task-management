---
id: H1-T18B
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T1
  - H1-T11
  - H1-T12
  - H1-T13
  - H1-T18A
---

# H1-T18B: Stage core workflow loading

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
currently loads Workflow Entry, Intent, Current Truth, and Risk Router together
through `inspect-context`. This task changes the entry protocol so the model
receives only the next lifecycle node when it is needed:

```text
repository startup → Intent
confirmed Intent → Current Truth
Current Truth ready or routable unresolved → Risk Router
Router-selected route → selected workflow family
```

Constitution integrity remains exhaustive. This task changes context loading,
not Intent, Current Truth, or Router semantics.

## Loading contract

Keep every non-workflow rule and Workflow Entry in `always` bootstrap context.
Keep Intent in bootstrap context so a new session can interpret the first work
request. Reclassify Current Truth and Risk Router as `on_demand`, alongside
Direct, Research, Explore, and Spike.

After Intent returns confirmed `ready`, the Intent rule must invoke
`inspect-workflow rule-current-truth` with the current relevant paths, then
read only the returned chain. After Current Truth returns `ready` or a
routable `unresolved` result such as an inspectable evidence gap, that rule
invokes `inspect-workflow rule-risk-router` with those paths. An
`insufficient_query` returns to the same Intent; `invalid_constitution` and
`rule_conflict` stop and notify the user; an authority-bound conflict waits
for the user's decision. None of those outcomes loads Router. After Router
selects a route, it invokes the existing lookup for that selected workflow ID.
A route cannot make a sibling node enter context.

`inspect-workflow` remains a deterministic canonical lookup. It does not infer
whether a caller has completed Intent or Current Truth, and it does not become
a semantic lifecycle engine. The workflow rules and entry protocol own that
ordering. Existing scope, effective replacement-chain, extender, approval,
and exit-code behavior remain unchanged.

## Boundaries

Do not add stateful session tracking, context eviction, a workflow registry,
or a new validator command. Do not implement Plan, Commitment Gate, Execute,
or product behavior. The existing `inspect-effective` diagnostic result stays
complete, and `inspect-context` continues to return all applicable `always`
rules.

The change must preserve the honest incremental-context limitation: a rule
already read earlier in a conversation cannot be removed; rerouting only adds
the newly selected rule family.

## Implementation guidance

Install a new pinned framework snapshot. Update the Current Truth and Risk
Router workflow frontmatter to `contextLoading: on_demand`. Update Intent's
`ready` handoff to invoke `inspect-workflow rule-current-truth`; update Current
Truth's `ready` and routable `unresolved` handoffs to invoke
`inspect-workflow rule-risk-router`, while preserving its return and stop
outcomes. Update Workflow Entry, `AGENTS.md`, and `.harness/README.md` to state
the staged lookup sequence. Recompute framework provenance and digest, rebuild the
derived index, and retain `contextLoading` compatibility validation across
extensions and replacements.

Do not alter the lookup algorithm merely to encode lifecycle order. The
existing stable-ID lookup is deliberately reusable for both core successor
nodes and Router-selected routes.

## Test and verification plan

- Verify `inspect-context` contains Intent and Workflow Entry but excludes
  Current Truth, Risk Router, and every routed workflow.
- Verify `inspect-workflow rule-current-truth` and
  `inspect-workflow rule-risk-router` independently return exactly their
  selected effective families, without siblings or bootstrap rules.
- Verify `inspect-effective` still contains all effective core and routed
  rules, and loading-class/index validation still covers both new on-demand
  rules.
- Walk through Intent → Current Truth → Router → each existing route from a
  fresh bootstrap context; verify documentation loads only the next required
  workflow at every handoff. Check both Current Truth `ready` and routable
  `unresolved` results, plus `insufficient_query`, direct stop outcomes, and
  authority-bound conflicts; only `ready` and routable `unresolved` load Router.
- Verify path mismatch, replacement, extension, invalid ID, and `always`
  workflow selection retain H1-T18A's deterministic errors and no partial
  stdout.
- Run Constitution validation, Constitution tests, format, lint, and workspace
  tests.

## Definition of done

A new session loads Intent but not Current Truth, Risk Router, or a routed
workflow. Confirmed Intent loads Current Truth; Current Truth loads Router only
for `ready` or routable `unresolved` results. All rules remain fully
validated, indexed, pinned, and diagnosable.
Record acceptance impact after implementation; do not create or execute
acceptance scenarios during this task.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T18A](H1-T18A-lazy-workflow-loading.md)
- [H1-T11](H1-T11-current-truth.md)
- [H1-T12](H1-T12-intent.md)
- [H1-T13](H1-T13-risk-router.md)
- [H1-T18](H1-T18-plan.md)

## Planning review (2026-09-28)

The user requested a full contract review and clarified that a routable Current
Truth `unresolved` result must load Risk Router. The handoff, guidance,
verification, and completion criteria above now cover that path while preserving
the existing return and stop boundaries. No other material design choice was
reopened. Structured self-review found the direction, outcome branches, tests,
references, and definition of done complete. The paired planning handoff
records the dependency cross-check.

## Implementation completion

Implemented in pinned Constitution snapshot 1.9.0. Bootstrap keeps Intent and
excludes Current Truth, Risk Router, and routed workflows. Confirmed Intent now
looks up Current Truth; Current Truth looks up Router only for `ready` and
routable `unresolved` results. The deterministic lookup algorithm, exhaustive
diagnostics, and loading compatibility checks remain unchanged.

Constitution validation, focused lookup walkthroughs, the Constitution suite,
format, lint, and workspace tests passed. Acceptance impact: `none`, because
this changes repository harness behavior only and no `tbtm` product workflow.
