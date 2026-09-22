---
id: H1-T13
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - H1-T1
  - H1-T11
  - H1-T12
---

# H1-T13: Establish Risk Router workflow

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
defines the node-first lifecycle. Given confirmed Intent and relevant Current
Truth, Risk Router selects the smallest credible next workflow and briefly
explains why. It revisits that choice when evidence, risk, authority, or the
confirmed goal and scope change. Router does not perform the next workflow,
approve a commitment, or turn routing into a mandatory document.

## Inputs and boundaries

- Consume H1-T12's confirmed goal, scope, constraints, and information need,
  plus H1-T11's sourced `ready` or routable `unresolved` view, including
  nonblocking unknowns and evidence gaps. Do not substitute Router's own
  unsourced repository summary for Current Truth.
- `insufficient_query` returns from Current Truth to the same Intent before
  Router. `invalid_constitution` and `rule_conflict` stop at Current Truth and
  are reported to the user; Router never scores through these stops.
- An authority-bound conflict reaching Router pauses for the user's decision.
  Router may identify a newly exposed authority question but cannot grant
  approval or change an approved contract.
- Lack of an implementation approach is not unclear Intent. Return to Intent
  only when the confirmed goal or scope is materially wrong, insufficient, or
  has changed. A routine solution choice remains in the downstream workflow.

## Six decision signals

Use these questions as a compact reasoning matrix, not a questionnaire that
the user must answer or a weighted algorithm. Draw on the relevant Current
Truth evidence; mark a concern `low`, `medium`, or `high` only when the level
helps explain the route. No total score, threshold, or fixed tie-break chooses
the path. The agent names the decisive signal and exercises judgment.

| Question | What it distinguishes |
|---|---|
| Is the confirmed goal, scope, and outcome still the work at hand? | Material drift returns to Intent; uncertainty about how to implement does not. |
| Is the work within current authority and approved contracts? | A user-owned decision stops before further action. |
| Is there a demonstrated repository pattern and evidence that it fits here? | Proven fit supports Direct; a superficial resemblance does not. |
| Which boundaries will change: module, interface, data, consumer, or rule? | Broad or coupled change favors Plan even when the mechanism is familiar. |
| What is the main unknown, and can reading sources or a bounded experiment resolve it? | Missing facts favor Research; competing approaches favor Explore; feasibility requires Spike. |
| How reversible is the work, and how will success or failure be verified? | Difficult recovery or weak proof favors more preparation or a bounded probe. |

Do not make a new module automatically require Plan or a familiar pattern
automatically qualify for Direct. Weigh scope, uncertainty, consequence, and
verification together. A high-consequence signal cannot be canceled by several
low-concern signals. If new evidence changes the assessment, route again.

## Routes and handoffs

| Path | Next work and expected return |
|---|---|
| Direct | H1-T14 Direct path frames bounded action, then H1-T20 Execute and Proof of Work. Suitable when the approach is established and relevant unknowns are nonblocking. |
| Research | H1-T15 finds a missing fact or source; a concise sourced result or unresolved question returns through its Context Capsule to Router. |
| Explore | H1-T16 compares materially viable approaches and trade-offs; its Context Capsule returns to Router. |
| Spike | H1-T17 tests a consequential feasibility assumption with a bounded probe; evidence and limitations return through its Context Capsule to Router. A POC is a possible probe, not a separate route or automatically production code. |
| Plan | H1-T18 structures substantial or coupled work, then H1-T19 Commitment Gate decides whether H1-T20 Execute may begin. A clear but consequential task can take Plan without preliminary Research or Spike. |
| Return to Intent | H1-T12 clarifies or reconfirms a materially changed goal or scope, then Current Truth refreshes affected context before Router runs again. |
| Stop for user authority | State the exact decision and applicable boundary, do no downstream work, and resume from the affected upstream point after the user decides. This is not a substitute for Current Truth's direct Constitution stops. |

Research, Explore, and Spike are different ways to reduce uncertainty, not
universal prerequisites to Plan. Their output is a candidate Context Capsule;
H1-T13 does not prescribe its storage or schema. If a new route needs different
sources or a material source changes, refresh the affected Current Truth
claims before treating the next choice as current.

## User-visible and downstream output

Show the user the same short routing handoff given to the next workflow. A
single sentence or compact two-line form is normally enough:

```text
Path: <next path> — <decisive reason grounded in current evidence>.
Next: <specific question, probe, decision, or bounded work for that path>.
```

Omit a separate risk score unless `low`/`medium`/`high` materially clarifies
the reason. Do not print the whole six-question matrix, create a per-request
routing document, or require a machine-readable schema. A route is a current
recommendation, not user approval, proof of completion, or permission to
bypass a downstream gate. A receiving workflow can ask Router to reconsider
when its assumptions fail or new material evidence appears.

## Implementation guidance

Add one concise framework-origin workflow rule under `.harness/workflow/`,
using the H1-T1 Constitution format and bounded initial-construction
authorization. Use an embedded Mermaid flowchart for entry, decisive checks,
route handoffs, and return/stop paths. Pair bad/good examples for intent drift
versus solution uncertainty, Direct versus investigation/Plan, and a concise
routing handoff. Keep the rule generic: repository-specific paths, scoring
weights, mandatory scripts, a new routing database, and a universal node
schema are out of scope.

The implementation must update the pinned ruleset version, framework-rule
provenance, digest, and derived index consistently, then validate the
Constitution. It must not edit an older pinned snapshot in place or activate
project-origin drafts. Do not implement the downstream H1-T14–H1-T22 nodes
within this task.

## Verification

- Walk through a familiar reversible change (Direct), a missing inspectable
  fact (Research), competing viable approaches (Explore), a blocking
  feasibility assumption (Spike), and a clear but cross-boundary change
  (Plan). For each, check the decisive evidence, next workflow, and concise
  user-facing handoff.
- Walk through material goal/scope drift (Return to Intent) versus a clear
  request with an unknown implementation method (not Return to Intent).
- Walk through a user-owned decision reaching Router (stop) and H1-T11's
  `insufficient_query`, `invalid_constitution`, and `rule_conflict` (never
  routed through Router).
- Change a material fact or uncover a new dependency after routing; verify
  affected Current Truth is refreshed where needed and Router can choose a
  different path without defending its previous choice.
- Verify no per-request document, numeric route threshold, extra universal
  approval, product CLI change, or acceptance-scenario change is introduced.
  For documentation-only implementation, check links, examples, validator,
  and `git diff --check`; run repository format, lint, and test commands if
  source or tooling changes.

## Definition of done

An agent can use confirmed Intent and sourced Current Truth to choose and
briefly explain a credible next path, hand off only what that path needs,
honor upstream stops and user authority, and reroute when material evidence
changes. The workflow is discoverable through effective Constitution lookup
and passes its validation. Record actual acceptance impact at implementation
completion; do not create or execute product acceptance scenarios here.

## References and planning review

- [H1-T0 validated lifecycle](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)

The user chose the original six questions as reasoning signals, qualitative
levels rather than a route-scoring algorithm, agent judgment for the route,
and a short ephemeral user-visible/downstream handoff. Independent decision
review: READY. After drafting, H1-T0, H1-T1, H1-T11, and H1-T12 were re-read:
this task preserves their node-first flow, Constitution pinning, Current
Truth's direct stop/return behavior, and Intent's goal/scope boundary without
creating a competing source or universal node schema. NO CONFLICT. Final
direct-document review: READY. All direct implementation dependencies are
`done`, so H1-T13 implementation is `ready`.
