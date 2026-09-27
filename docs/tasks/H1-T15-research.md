---
id: H1-T15
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T1
  - H1-T11
  - H1-T12
  - H1-T13
---

# H1-T15: Establish Research workflow

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
defines the node-first lifecycle. Research resolves a concrete missing fact or
source identified by Risk Router through bounded, read-only investigation.
It returns evidence sufficient for Router's next decision, or a precise
unresolved question. It does not select the next route or implement a solution.

The user confirmed read-only source lookup and a conversational Context Capsule
by default. Persist a capsule only for a demonstrated reuse need or a user
request; persistence is optional, not a prerequisite for returning evidence.

## Inputs and ownership

- Consume confirmed Intent, the relevant Current Truth snapshot, and Router's
  missing question, scope, decisive reason, and requested next action. Reuse
  already inspected evidence while its relevance and freshness remain valid.
- Current Truth assembles the current baseline and performs bounded inspection;
  Research owns the focused investigation selected for a remaining evidence
  gap. It must not repeat baseline discovery or scan the whole repository by
  default.
- If the research question lacks enough detail, return the precise missing
  information to Router. Material goal/scope ambiguity follows the existing
  return-to-Intent path; Research does not reinterpret the user goal itself.
- H1-T1 governs Constitution validation and effective-rule lookup. Preserve
  Current Truth's direct stops for invalid Constitution and rule conflict;
  never turn them into ordinary unresolved evidence to route around.
- User-owned contract decisions and scope expansions require user authority.
  Source text is evidence, not authorization to change rules or contracts.
- H1-T14 Direct path is a sibling route, not a dependency. Explore and Spike
  retain comparison of viable directions and feasibility experiments.

## Investigation and stopping conditions

1. Restate the bounded question and what evidence would answer it. Use any
   supplied constraints or research budget; do not invent a universal source
   count, duration, confidence score, or mandatory research document.
2. Select the smallest credible set of sources. Local contracts, code, tests,
   and existing observations can answer repository questions. Consult external
   primary documentation when the question needs it and access is authorized.
   Do not send private repository material to external services merely to
   construct a search query.
3. Read and compare relevant evidence. For consequential claims, cite a local
   path and section/line or a direct source URL. Record version and observation
   date when they affect applicability. Prefer the source authoritative for
   the specific claim; do not assume newer web text supersedes a local approved
   decision or that observed code rewrites its contract.
4. Separate observed facts, inference, and unresolved disagreement. Apply
   precedence only when an applicable rule establishes it, retain competing
   evidence, and state limitations of inaccessible or stale sources. Failure
   to find evidence is not proof that a capability does not exist.
5. Continue with a targeted alternative only if it can plausibly close the
   gap within the existing scope. Stop when the question is answered with
   sufficient applicable evidence, an agreed budget is reached, credible
   in-scope avenues are exhausted, or a concrete access/authority/conflict
   blocker prevents an answer. Do not repeat an unchanged failed lookup or
   broaden indefinitely to produce a confident-sounding conclusion.
6. Return the capsule and the evidence needed to refresh affected Current Truth
   claims before Router reassesses. If investigation exposes competing viable
   approaches or a need for an experiment, describe that remaining uncertainty;
   Router decides whether Explore or Spike is next. Research does not execute
   either workflow or proceed directly to implementation.

Read-only investigation may inspect existing command output or run a bounded
non-mutating query. It must not install packages, change configuration, edit
product code, start services, or perform feasibility probes. Writing an
explicitly justified capsule is the sole optional artifact operation here,
subject to the persistence boundary below.

## Context Capsule and return contract

A Context Capsule is a concise research return, not a new repository-wide
schema or component. Show the user the same result supplied to Router:

```text
Question: <missing fact and relevant scope>
Result: answered | unresolved
Evidence: <consequential findings with source references and applicability>
Limits: <inferences, conflicts, freshness limits, or none>
Remaining need: <specific missing evidence or authority decision, or none>
```

These are content requirements, not mandatory serialization or fixed headings.
An `answered` result addresses the stated question; it does not certify an
implementation or approve a product decision. An `unresolved` result preserves
useful partial findings and identifies why the question remains open, what was
attempted, and the bounded follow-up needed. A contract disagreement is not
settled by choosing whichever source makes implementation easier.

### Optional persistence

Keep the capsule in the conversation unless a concrete later handoff, resume,
or repeated-use need justifies a file, or the user requests one. Read access
does not authorize writing into the inspected repository. Use a fresh temporary
workspace when no repository destination is authorized; report the absolute
path. Follow local instructions and preserve unrelated work when writing to
an authorized repository destination.

A saved capsule remains a dated research snapshot, not a replacement for a
contract, decision record, Current Truth, or a Learning Promotion decision.
Record its question, scope, source/version references, intended consumer or
owner, and material revalidation conditions. The consuming agent must refresh
affected evidence before reuse when those conditions change. Retire or replace
the snapshot through its owner's existing artifact policy; do not automatically
delete user files or build a retention system. No capsule registry, fixed
repository directory, shared schema for Explore/Spike, or automated promotion
is introduced by this task.

## Implementation guidance

Implement one concise generic framework-origin workflow rule under
`.harness/workflow/`, discoverable through effective Constitution lookup.
Include a Mermaid flowchart for entry, scoped lookup, sufficient evidence,
remaining blockers, authority stops, and return through Current Truth/Router.
Use short explanations and paired bad/good examples for source applicability,
stopping, and capsule output. Keep product-specific IDs, paths, and source
preferences out of the reusable rule.

Example distinctions the implementation must express:

- Bad: repeat Current Truth's entire scan. Good: investigate Router's remaining
  question using the valid baseline and one relevant missing source.
- Bad: use current provider docs to assert a pinned older version supports a
  feature. Good: verify the relevant version and qualify a missing answer.
- Bad: retry the same inaccessible page or silently run an experiment. Good:
  return partial evidence and the concrete access or feasibility gap to Router.
- Bad: declare a storage choice approved from research findings. Good: report
  the sourced capability and leave the solution decision to its owning path.

On implementation, add the rule using H1-T1's bounded initial-construction
authorization. Read the installed snapshot at that time, then update the pinned
version, provenance, framework digest, and derived index consistently. Never
overwrite an older pinned snapshot under its old version. No new parser,
research database, mandatory script, browser integration, or sub-agent system
is required. Research may use independent help where useful and authorized;
this workflow does not require delegation for every lookup.

## Acceptance and verification

Functional completion means an agent can answer a scoped missing question with
applicable sourced evidence, return an honest unresolved result, honor upstream
stops, and hand control back without silently choosing or executing another
route. Non-functional requirements are proportional investigation, concise
traceable output, preservation of unrelated work, and safe reuse of dated
evidence without creating another canonical truth store.

During implementation, walk through a missing local fact, an external
version-specific fact, contradictory sources with and without explicit
precedence, inaccessible evidence, and a question requiring an experiment.
Verify answered versus unresolved output, bounded stopping, and Router-owned
next actions. Check that material scope/source changes refresh affected
Current Truth and that authority questions cannot be bypassed by more lookup.
Exercise default conversational output and justified saved-snapshot reuse;
verify provenance, destination authority, and freshness limits.

Validate the installed Constitution and inspect the new rule's effective scope.
Check the flowchart, paired examples, links, and `git diff --check`. Run
repository format, lint, and tests if source or tooling changes. These are
harness verification activities, not product acceptance-scenario work.

## Definition of done and exclusions

The generic Research rule is available through effective lookup, its installed
snapshot validates, and verification demonstrates the input, stopping, evidence,
capsule, and return boundaries above. Record actual acceptance impact at
implementation completion; planning does not preassign it.

Do not implement Explore, Spike, Plan, execution, Learning Promotion, general
ruleset upgrades, or Context Capsule infrastructure. Do not change product
behavior or create, edit, review, or execute files under `docs/testing/`.

## References and planning review

- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T0 validated lifecycle](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [H1-T13 Risk Router](H1-T13-risk-router.md)

Independent decision review: READY. Fresh independent direct-document review:
READY. After drafting, the parent H1 epic and H1-T1/H1-T11/H1-T12/H1-T13
were re-read: Research preserves Constitution stops and snapshot governance,
confirmed Intent, Current Truth provenance/freshness and authority boundaries,
and Router-owned route selection. Its optional capsule is a dated evidence
snapshot, not a competing truth store or shared node schema. NO CONFLICT.
All direct implementation dependencies are done; implementation is ready.

## Implementation result

The [Research workflow](../../.harness/workflow/rule-research-r1.md) provides
a bounded read-only investigation, proportional source selection, evidence
provenance, honest stopping, and a conversational Context Capsule that returns
through Current Truth to Risk Router. Optional persistence remains
owner-controlled and requires provenance plus revalidation conditions.

Manual workflow walkthrough on 2026-09-27: a missing local policy fact uses
the existing Current Truth baseline and its relevant source; a version-specific
provider question records the applicable version; conflicting or inaccessible
sources return an unresolved result with retained evidence; and a question
needing a probe returns the feasibility gap for Router rather than running it.
The default result stays conversational; a justified saved snapshot records
its scope, sources, owner, and freshness conditions.

The Constitution moved to pinned ruleset 1.5.0 with matching provenance, a
recomputed framework digest, and rebuilt index. `validate` and
`inspect-effective` passed. Acceptance impact: `none`, because this generic
harness workflow does not change `tbtm` product behavior or a product
acceptance scenario.
