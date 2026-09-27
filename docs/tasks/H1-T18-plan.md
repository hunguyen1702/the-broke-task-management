---
id: H1-T18
kind: implementation_task
planning_status: in_progress
implementation_status: not_planned
depends_on:
  - H1-T18A
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
goal or scope drift returns to Intent. A missing fact, unresolved choice, or
unproven feasibility assumption is stated precisely for Router rather than
causing Plan to invoke a sibling workflow directly.

Before writing artifacts, Plan presents the selected planning route, exact
artifacts, absolute output location, and decisions or assumptions that will be
recorded as settled facts, then waits for explicit user confirmation. This
artifact-generation confirmation is not Commitment Gate: it authorizes the
planning package, while Commitment Gate later decides whether that package is
ready to enter Execute.

## Node and subprocess structure

The implementation adds one concise framework-origin Constitution rule for the
top-level Plan node. That rule owns entry, input reconciliation, scope
classification, shared authority and drift behavior, selective guide loading,
cross-document review, and the Commitment Gate handoff.

Three canonical subprocess workflow guides remain separate from the active
repository-wide Constitution rule set:

1. **Epic Planning** creates an epic-level contract and a small task-board
   document containing future planning units.
2. **Story Planning** creates a bounded story contract and identifies its
   implementation units.
3. **Implementation Task Planning** creates the actionable technical
   specification for one implementation unit.

Each guide contains its own input checks, procedure, output template,
verification checklist, and stop conditions. The Plan rule selects a guide
before reading it. It must not load Epic Planning during a Story-only route or
load Story/Task Planning merely to enrich an Epic decomposition.

A route may compose subprocesses only when their outputs are required for the
same confirmed package. A bounded story with one cohesive implementation unit
may run Story Planning followed by Implementation Task Planning in one
session. A story with independently schedulable implementation units produces
their planning-unit records and leaves their detailed Task Planning for later
work. Epic Planning never pre-plans all child stories.

## Scope classification

Plan selects the smallest planning level that represents the confirmed work:

- choose **Epic Planning** when there are multiple independently deliverable
  outcomes, multiple subsystems or user journeys, a major unresolved product
  or architectural boundary, or implementation and verification cannot form
  one cohesive delivery;
- choose **Story Planning** only when there is one observable outcome, bounded
  scope, no independently schedulable decomposition requirement, and a
  credible cohesive implementation and verification boundary; and
- choose **Implementation Task Planning** when an approved story contract
  already fixes observable behavior and one implementation unit needs
  technical specification.

Estimated duration, document length, or a desired number of child items does
not determine the route. If a Task route reveals multiple observable outcomes,
it stops as mis-scoped and returns to Plan classification rather than
recursively decomposing implementation tasks without a bounded end.

## Subprocess output contracts

### Epic Planning

The epic contract records outcome, problem and context, actors when relevant,
scope and exclusions, confirmed product and solution decisions, cross-cutting
constraints and invariants, success criteria, and explicitly deferred open
questions. Its technical content stays at cross-cutting architecture,
integration, migration, risk, and verification boundaries.

The paired task board lists each future planning unit by ID, title, outcome,
scope summary, and planning state. It does not include detailed story
contracts, implementation specifications, claims, owners, or execution state.

### Story Planning

The story contract records one observable outcome, the actor or operator
intent, scope and exclusions, confirmed decisions, interfaces and behavior,
functional requirements, non-functional requirements, acceptance criteria,
verification approach, and links to its implementation planning units.

Story Planning identifies only implementation units required by the story. It
does not fill their technical specifications unless the same confirmed Plan
package intentionally composes Implementation Task Planning.

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
reconciliation, scope classification, selective subprocess loading, optional
Story-to-Task composition, cross-document review, unresolved-return paths, and
handoff to Commitment Gate.

1. Recheck confirmed Intent, Router handoff, Current Truth, requirement, and
   applicable capsules.
2. Classify the target as Epic, Story, or Implementation Task using observable
   delivery boundaries.
3. Present route, artifacts, destination, and settled assumptions; wait for
   explicit artifact-generation confirmation.
4. Load only the selected subprocess guide. Load a second guide only when the
   confirmed package requires a valid composition.
5. Produce the subprocess artifacts using that guide's template and preserve
   source links and open decisions.
6. Review the whole package against upstream inputs and parent/child contracts.
7. Return material drift or a precise evidence/decision gap to its owning
   boundary. Otherwise hand the reviewed package to Commitment Gate.

Include paired bad/good examples for scope classification, selective context
loading, Epic decomposition, Story-to-Task composition, provenance handling,
and Commitment Gate handoff.

## Implementation guidance

Add one concise framework-origin Plan Constitution rule using H1-T1's pinned
ruleset process and initial H1 construction authorization. Add exactly three
on-demand subprocess workflow-guide files for Epic, Story, and Implementation
Task Planning. Keep their route-specific templates with their respective
guides so an unused template is not loaded.

The guide files must not become active repository-wide rules returned by every
effective Constitution lookup. The Plan rule is the discoverable entry and
names the selected guide-loading contract. Determine safe guide paths from the
current validator and repository layout during implementation; if supporting
guides require a new governed manifest or schema concept, stop and return that
scope change rather than silently altering Constitution semantics.

Update the pinned ruleset version, framework-rule installation provenance,
framework digest, derived index, and isolated Constitution tests consistently.
Do not implement Commitment Gate, Execute, product behavior, task
materialization, a shared workflow-node schema, mandatory multi-agent
orchestration, or acceptance scenarios.

## Test and verification plan

- Walk through an initiative with multiple independent outcomes; verify Epic
  Planning produces only the epic contract and future-planning task board.
- Walk through one bounded story; verify Story Planning produces complete
  functional and non-functional requirements, acceptance criteria, and
  verification, then composes Task Planning only when the confirmed package
  includes one cohesive implementation unit.
- Walk through a story with multiple independently schedulable implementation
  units; verify it records later Task Planning work without eagerly loading or
  generating every technical specification.
- Walk through an approved story with one implementation unit; verify the Task
  specification is actionable and cannot change observable story behavior.
- Verify duration, document size, or arbitrary child-count thresholds do not
  select the route; verify a mis-scoped Task returns to classification.
- Supply Research, Explore, and Spike capsules containing sourced facts,
  recommendations, assumptions, and an inconclusive result; verify Plan keeps
  their provenance and confidence distinctions.
- Remove a necessary fact, leave a consequential choice unresolved, and change
  the confirmed goal; verify Plan returns the exact need to Router or Intent
  without invoking sibling workflows.
- Verify an Epic route does not read Story or Task guides, a Story-only route
  does not read the Epic guide, and valid composition loads only the second
  guide it actually needs.
- Verify route, artifact list, absolute destination, and settled assumptions
  receive explicit confirmation before any artifact write.
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
planning level, load only the needed subprocess workflow files, and produce a
coherent Epic, Story, or Implementation Task planning package using the
appropriate template. Bounded Story and Task planning may compose without
forcing unrelated context into other routes. Epic decomposition remains
shallow, technical specifications remain subordinate to approved requirements,
and the reviewed package reaches Commitment Gate without implying execution
approval. The Constitution remains valid and discoverable. Record acceptance
impact at implementation completion; do not create, modify, or execute product
acceptance scenarios in this task.

## References and planning review

- [H1-T0 validated lifecycle](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [H1-T13 Risk Router](H1-T13-risk-router.md)
- [H1-T15 Research](H1-T15-research.md)
- [H1-T16 Explore](H1-T16-explore.md)
- [H1-T17 Spike](H1-T17-spike.md)
- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)
- [Constitution lookup](../../.harness/README.md)

Planning decisions confirmed by the user: Plan remains one top-level node with
three separate, selectively loaded subprocess workflow files. The subprocesses
own Epic, Story, and Implementation Task planning and their distinct templates.
Plan consumes requirements, Current Truth, and applicable Context Capsules;
it produces planning documents and stays independent of later execution or
integration concerns.
