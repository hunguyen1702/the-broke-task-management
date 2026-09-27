---
id: H1-T18
kind: implementation_task
planning_status: done
implementation_status: not_planned
depends_on:
  - H1-T18A
  - H1-T18B
  - H1-T1
  - H1-T11
  - H1-T12
  - H1-T13
  - H1-T15
  - H1-T16
  - H1-T17
---

# H1-T18: Establish Plan workflow

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
defines Plan as the path for substantial or coupled work whose goal and scope
are sufficiently clear to structure before execution. Given the confirmed
Intent, relevant Current Truth, the requirement, and any applicable Context
Capsules returned by Research, Explore, or Spike, Plan produces a coherent
planning package and hands it to Commitment Gate.

Plan remains one top-level lifecycle node. Internally it dispatches to three
separate, on-demand subprocess workflows: Epic Planning, Story Planning, and
Implementation Task Planning. Their instructions and templates live in
separate files so an agent reads only the subprocesses required by the current
planning route.

Plan does not approve its own package, execute it, implement Commitment Gate,
or absorb Research, Explore, or Spike.

## Inputs and shared boundaries

Plan consumes:

- the confirmed Context–Task–Format Intent and its authority boundary;
- the requirement and the Router handoff that selected Plan;
- relevant Current Truth claims with their sources and freshness limits; and
- relevant Context Capsules, when earlier Research, Explore, or Spike work
  exists.

The top-level workflow reconciles these inputs without silently promoting a
recommendation, assumption, or inconclusive probe to a confirmed decision. It
preserves provenance for consequential constraints and decisions. Material
goal or scope drift returns to Intent. A missing fact, unresolved choice needed
to define scope or design, or unproven feasibility assumption is stated
precisely for Router rather than causing Plan to invoke a sibling workflow
directly. Only an explicitly deferred, non-blocking question may remain
labelled in a planning package; Commitment Gate later decides whether it can
remain deferred for the proposed execution boundary.

Before writing artifacts, Plan presents the selected planning route, exact
artifacts, absolute output location, confirmed decisions that will be recorded
as settled facts, and assumptions or open questions that will remain explicitly
labelled, then waits for explicit user confirmation. This
artifact-generation confirmation is not Commitment Gate: it authorizes the
planning package, while Commitment Gate later decides whether that package is
ready to enter Execute.

## Node and subprocess structure

The implementation adds one concise, active, on-demand framework-origin
Constitution rule for the top-level Plan node. That rule owns entry, input
reconciliation, scope classification, shared authority and drift behavior,
selective subprocess loading, cross-document review, and the Commitment Gate
handoff.

Three more active, on-demand framework workflow rules implement the
subprocesses:

1. **Epic Planning** creates an epic-level contract and a small task-board
   document containing future planning units.
2. **Story Planning** creates a bounded story contract and identifies its
   implementation units.
3. **Implementation Task Planning** creates the actionable technical
   specification for one implementation unit.

Each subprocess rule is a separate file containing its own input checks,
procedure, output templates, verification checklist, and stop conditions. The
Plan rule selects a subprocess by stable workflow ID before loading it through
H1-T18A's nested on-demand lookup. It must not load Epic Planning during a
Story-only route or load Story/Task Planning merely to enrich an Epic
decomposition.

A route may compose subprocesses only when their outputs are required for the
same confirmed package. A bounded story with one cohesive implementation unit
must run Story Planning followed by Implementation Task Planning in one
session. A story whose single observable outcome requires multiple technical
implementation units creates a story-level technical design and an
implementation-planning board, then leaves each detailed Task Planning package
for later work. Epic Planning never pre-plans all child stories.

## Scope classification

Plan selects the smallest planning level that represents the confirmed work:

- choose **Epic Planning** when there are multiple independently deliverable
  outcomes, multiple subsystems or user journeys, or implementation and
  verification cannot form one cohesive delivery. A blocking unresolved
  product or architecture choice returns to Router or user authority rather
  than becoming a reason for Plan to decide it; an explicitly deferred,
  non-blocking question may remain visible in the Epic package;
- choose **Story Planning** when there is one observable outcome and bounded
  product scope with a credible cohesive acceptance boundary, even when its
  implementation requires several separately schedulable technical units
  that have no independent product outcome; and
- choose **Implementation Task Planning** when an approved story contract
  already fixes observable behavior and one implementation unit needs
  technical specification.

Estimated duration, document length, or a desired number of child items does
not determine the route. If a Task route reveals multiple observable outcomes,
it stops as mis-scoped and returns to Plan classification rather than
recursively decomposing implementation tasks without a bounded end.

## Subprocess output contracts

### Epic Planning

The epic route produces three artifacts. The epic contract records outcome,
problem and context, actors when relevant, scope and exclusions, confirmed
product decisions, cross-cutting constraints and invariants, success criteria,
and explicitly deferred open questions. A separate epic technical-design
document records the cross-cutting architecture, component boundaries, data or
control flow, integration and migration concerns when relevant, technical
risks, and verification strategy without designing every child story.

The task board lists each future planning unit by ID, title, outcome,
scope summary, and planning state. It does not include detailed story
contracts, implementation specifications, claims, owners, or execution state.

### Story Planning

The story contract records one observable outcome, the actor or operator
intent, scope and exclusions, confirmed decisions, interfaces and behavior,
functional requirements, non-functional requirements, acceptance criteria,
verification approach, story-level definition of done, and links to its
implementation planning units.

For one cohesive implementation unit, the paired technical artifact is the
Implementation Task specification produced by mandatory composition with Task
Planning. For multiple technical units, Story Planning instead produces a
story-level technical-design document covering shared boundaries, interactions,
constraints, risks, and verification, plus an implementation-planning board
listing each future Task Planning unit. It does not eagerly fill those task
specifications.

The implementation-planning board uses one row per unit with stable ID, title,
bounded objective or scope, planning state (`needed` or `planned`), and a
document link when a Task specification exists. It records dependency or
ordering only when the Story contract actually requires it; it does not add
claim, owner, implementation status, or execution state.

### Implementation Task Planning

The task specification records its parent story, objective, deliverables,
evidence-backed proposed structure, technical choices and boundaries,
implementation flow at coding-agent granularity, applicable state,
persistence, concurrency, output and error contracts, test plan, verification
commands or checks, references, and definition of done.

It is actionable without becoming a line-by-line patch. It cannot revise the
story's observable behavior; any such need returns to Story Planning or user
authority as appropriate.

## Review, package, and handoff

Before handoff, Plan verifies that:

- the selected scope and artifact set match;
- requirement and technical documents do not contradict each other;
- every consequential decision or constraint retains its source;
- assumptions and open questions are not presented as confirmed decisions;
- acceptance criteria and verification are observable and sufficient;
- paired documents link to each other;
- Epic Planning has not pre-planned child stories;
- Story Planning has not silently expanded the epic or requirement; and
- Task Planning satisfies, but does not rewrite, its parent story contract.

The completed package is durable planning output. Plan reports its artifacts,
review outcome, unresolved blockers, and the exact package proposed for
Commitment Gate. Plan does not claim that review or document creation commits
the user to execution.

## Workflow and stop behavior

The top-level rule must contain a concise Mermaid flowchart covering input
reconciliation, scope classification, selective subprocess loading, conditional
Story-to-Task composition, cross-document review, unresolved-return paths, and
handoff to Commitment Gate.

1. Recheck confirmed Intent, Router handoff, Current Truth, requirement, and
   applicable capsules.
2. Classify the target as Epic, Story, or Implementation Task using observable
   delivery boundaries.
3. Present route, artifacts, destination, settled decisions, and explicitly
   labelled assumptions or open questions; wait for explicit
   artifact-generation confirmation.
4. Load only the selected subprocess rule. Load a second rule only when the
   confirmed package requires a valid composition.
5. Produce the subprocess artifacts using that rule's templates and preserve
   source links and open decisions.
6. Review the whole package against upstream inputs and parent/child contracts.
7. Return material drift or a precise evidence/decision gap to its owning
   boundary. Otherwise hand the reviewed package to Commitment Gate.

Include paired bad/good examples for scope classification, selective context
loading, Epic decomposition, Story-to-Task composition, provenance handling,
and Commitment Gate handoff.

## Implementation guidance

Add exactly four active framework-origin Constitution workflow rules under
`.harness/workflow/` using H1-T1's pinned ruleset process and initial H1
construction authorization:

- `rule-plan-r1.md` for the top-level Plan coordinator;
- `rule-plan-epic-r1.md` for Epic Planning;
- `rule-plan-story-r1.md` for Story Planning; and
- `rule-plan-task-r1.md` for Implementation Task Planning.

All four use H1-T18A's `on_demand` loading classification. The coordinator
selects a subprocess by stable rule ID and uses nested `inspect-workflow`
lookup; it does not read sibling rule files. Keep each subprocess's
route-specific templates in its own rule. H1-T18 depends on H1-T18A and
H1-T18B and does not reimplement or revise the loading mechanism. Do not connect subprocess
rules to the coordinator with `extends`, because H1-T18A intentionally returns
all effective extenders of a selected workflow family and would therefore load
siblings together. The already-loaded coordinator supplies the shared context;
each subprocess is selected independently by its own stable ID.

Update the pinned ruleset version, framework-rule installation provenance,
framework digest, derived index, and isolated Constitution tests consistently.
Do not implement Commitment Gate, Execute, product behavior, a shared
workflow-node schema, mandatory multi-agent orchestration, or acceptance
scenarios.

## Test and verification plan

- Walk through an initiative with multiple independent outcomes; verify Epic
  Planning produces an epic contract, separate epic technical design, and
  future-planning task board without detailed child-story plans.
- Walk through one bounded story with one implementation unit; verify Story
  Planning produces complete functional and non-functional requirements,
  acceptance criteria, and verification, then must compose Task Planning to
  produce the paired technical specification.
- Walk through a story with one product outcome and multiple technical units;
  verify it remains a Story route, produces a story-level technical design and
  implementation-planning board, and defers detailed Task Planning without
  treating technical scheduling as multiple product outcomes.
- Walk through an approved story with one implementation unit; verify the Task
  specification is actionable and cannot change observable story behavior.
- Verify duration, document size, or arbitrary child-count thresholds do not
  select the route; verify a mis-scoped Task returns to classification.
- Supply Research, Explore, and Spike capsules containing sourced facts,
  recommendations, assumptions, and an inconclusive result; verify Plan keeps
  their provenance and confidence distinctions.
- Remove a necessary fact, leave a scope- or design-blocking choice unresolved,
  and change the confirmed goal; verify Plan returns the exact need to Router,
  user authority, or Intent without invoking sibling workflows. Verify only an
  explicitly deferred non-blocking question remains labelled in an Epic and
  that Commitment Gate must assess its effect on execution readiness.
- Verify an Epic route does not read Story or Task rules, a Story route does
  not read the Epic rule, and valid composition loads only the second rule it
  actually needs through nested on-demand lookup. Verify the subprocess rules
  do not use relationships that make sibling rules part of one workflow family.
- Verify route, artifact list, absolute destination, settled decisions, and
  explicitly labelled assumptions or open questions receive confirmation
  before any artifact write.
- Verify cross-document review catches contradictions, missing links,
  unverifiable acceptance criteria, unsupported settled claims, and child
  detail that exceeds the selected planning level.
- Verify the final report distinguishes a reviewed planning package from
  approval to execute and hands the package to Commitment Gate without
  implementing that node.
- Validate the Constitution, inspect the effective Plan rule, and exercise the
  isolated Constitution test suite. Run repository format, lint, and tests
  required at implementation handoff.

## Definition of done

An agent entering Plan can reconcile upstream inputs, select the correct
planning level, load only the needed active subprocess workflow rules, and
produce a coherent Epic, Story, or Implementation Task planning package using
the appropriate templates. Every Epic has a separate technical design; every
bounded single-unit Story has a paired Task specification; and a multi-unit
Story has a shared technical design plus bounded future Task Planning units.
Epic decomposition remains shallow, technical specifications remain
subordinate to approved requirements, and the reviewed package reaches
Commitment Gate without implying execution approval. The Constitution remains
valid and discoverable. Record acceptance impact at implementation completion;
do not create, modify, or execute product acceptance scenarios in this task.

## References and planning review

- [H1-T0 validated lifecycle](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [H1-T13 Risk Router](H1-T13-risk-router.md)
- [H1-T15 Research](H1-T15-research.md)
- [H1-T16 Explore](H1-T16-explore.md)
- [H1-T17 Spike](H1-T17-spike.md)
- [H1-T18A lazy workflow loading](H1-T18A-lazy-workflow-loading.md)
- [H1-T18B staged core workflow loading](H1-T18B-staged-core-workflow-loading.md)
- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)
- [Constitution lookup](../../.harness/README.md)

Planning decisions confirmed by the user: Plan remains one top-level node with
three separate active, on-demand subprocess workflow rules. The subprocesses
own Epic, Story, and Implementation Task planning and their distinct templates.
Plan consumes requirements, Current Truth, and applicable Context Capsules and
produces planning documents for later Commitment Gate review.

After drafting, H1-T1, H1-T11, H1-T12, H1-T13, H1-T15, H1-T16, H1-T17,
H1-T18A, H1-T18B, and parent H1 were re-read. Plan preserves Constitution pinning,
sourced Current Truth, confirmed Intent, Router ownership, and the provenance
and uncertainty boundaries of all three Context Capsule producers. Its four
active rules use H1-T18A's stable-ID nested lookup without `extends`, so the
coordinator and selected subprocess load without sibling subprocesses.
Dependency cross-check: `NO CONFLICT`.

Independent review found and then verified fixes for subprocess governance,
Story classification, companion technical-design artifacts, provenance
wording, blocking versus deferred decisions, Story definition of done, the
implementation-planning-board schema, and conditional Story-to-Task
composition. Final review: `READY`. All direct implementation dependencies are
`done`, so H1-T18 implementation is `ready`.
