---
id: H1-T11
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - H1-T1
---

# H1-T11: Establish Current Truth workflow

## Parent and readiness

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md).
H1-T0 approved the node-first flow, and H1-T1 supplies the Constitution lookup
boundary. This task owns Current Truth only. It does not implement Intent,
Risk Router, Research, Context Capsule persistence, or Learning Promotion.

## Outcome

Given a sufficiently specific information need and scope from Intent, an agent
can assemble a concise, source-backed view of the repository's relevant
current facts, decisions, work state, observations, unknowns, and conflicts.
The view informs the Router without becoming a competing source of truth.

## Input contract

- A concrete question or information need, plus the relevant target scope
  known so far. Intent owns interpreting the user's request and supplying
  these inputs; Current Truth does not infer a goal from an empty or vague
  prompt.
- Applicable Constitution rules obtained through the H1-T1 validation and
  effective-rule lookup for all relevant paths.
- Read access to relevant canonical documents, decision records, task status,
  code, tests, working-tree state, and runtime observations where available.
  Selection is request-specific rather than a mandatory full-repository scan.

If the question or scope is too vague to select sources responsibly, return
`unresolved` with reason `insufficient_query` and the precise clarification
needed by Intent. Do not start a broad speculative search or ask the user
directly from this node.

## Workflow

1. Check that the information need and scope are specific enough to choose
   relevant sources. Reject insufficient input to Intent.
2. Select and read the smallest credible source set. Distinguish normative
   contract/policy/ADR decisions from observed code, tests, command output,
   Git state, and planning/implementation status.
3. Extract only claims needed to answer the information need. Attach a source
   reference to every consequential claim and identify when it was observed
   if freshness matters.
4. Reconcile claims semantically. Apply source precedence only when an
   applicable canonical rule explicitly defines it. Do not infer that code
   behavior overrides an approved contract, or that a passing structural
   check resolves a semantic disagreement.
5. Mark unanswered questions, stale observations, and conflicts. If a missing
   fact can be found by bounded in-scope inspection, continue that inspection
   before reporting an unresolved result. Use deterministic commands or
   scripts for objective predicates when useful; semantic reasoning owns
   relevance and reconciliation.
6. Return a compact result. Re-query the affected sources when the request
   scope or material repository state changes; do not present an old snapshot
   as current.

## Output contract

There are two top-level outcomes; neither requires a durable file per request.

- `ready`: relevant facts and decisions with sources, observed implementation
  or work state where needed, and nonblocking unknowns. This output can feed
  Risk Router.
- `unresolved`: a reason, the evidence/source references already found, the
  missing question or conflicting claims, and the type of follow-up needed.
  `insufficient_query` returns to Intent. A remaining inspectable evidence
  gap is reported to Risk Router, which chooses whether Research, Diagnose,
  or another path is warranted. An authority-bound contract or governance
  conflict requires an explicit user decision. Current Truth describes these
  needs but does not itself choose the next route or claim user approval.

The output is an ephemeral request-scoped snapshot by default. A later node
may persist a Context Capsule for handoff, resume, or audit; this task does not
define its storage, schema, or lifecycle. ADRs and approved contracts retain
their own canonical ownership. A Current Truth result may summarize and link
to them but must not duplicate or silently revise their rationale.

## Ownership boundaries

- Intent owns user-goal clarification, including questions returned as
  `insufficient_query`.
- Current Truth owns source selection, extraction, semantic reconciliation,
  provenance, freshness checks, and unresolved-reason reporting.
- Risk Router owns route selection and rerouting from the result.
- The user owns decisions that expand scope or alter approved product,
  architecture, or governance contracts; agents may resolve in-scope facts
  and implementation details without unnecessary user interruption.
- Decision records preserve durable rationale; Current Truth is a view over
  the currently applicable records and observations, not another ADR store.

## Implementation guidance

Implement the node as a concise agent workflow or guide using existing
repository tools and H1-T1 Constitution lookup. Include a small, concrete
output template and examples for a ready result, ambiguous Intent, missing
inspectable evidence, and conflicting canonical sources. Do not require a
new parser, repository-wide index, persistent database, or script merely to
give the node a tangible artifact. Add deterministic checks only for a stable,
objective predicate demonstrated by this node's needs.

The workflow must remain usable from a dirty worktree and must preserve
unrelated changes. It must report inaccessible sources or unverified claims
rather than inventing facts. If a material source changes during assembly,
refresh the affected claims before handing off the result.

## Verification

- Walk through a request with a clear information need and relevant ADR,
  contract, status, and code observations; verify separation of normative
  decisions from observed behavior and source-backed output.
- Give an underspecified request; verify `insufficient_query` returns to
  Intent without broad extraction or direct user questioning.
- Give a missing but inspectable fact; verify bounded further inspection
  rather than an immediate user prompt.
- Give contradictory approved sources without explicit precedence; verify
  `unresolved` preserves both claims and identifies the authority boundary.
- Give sources with an explicit applicable precedence rule; verify the result
  applies and cites that rule, retains the lower-priority claim as an
  observation rather than silently dropping it, and reports any unresolved
  remainder.
- Change the request scope or a relevant source; verify the affected result
  is refreshed rather than reused as current.
- Verify no new product CLI behavior, acceptance scenario file, mandatory
  script, or competing durable truth store is introduced.
- Run repository format, lint, and test commands if implementation changes
  source or tooling; for documentation-only implementation, verify links,
  examples, and `git diff --check` proportionally.

## Completion and exclusions

H1-T11 is complete when an agent can execute the documented workflow and
produce both outcome types with evidence, distinguish fact from decision and
observation, detect stale or conflicting context, and hand off without taking
Intent, Router, ADR, or user-decision authority. Record acceptance impact at
implementation completion based on actual changes; do not modify
`docs/testing/` during this task.

Do not predesign a universal node schema, implement Router or Context Capsule,
write new ADRs to mirror extracted facts, or promote speculative scripts.

## Planning review and dependency cross-check

Independent review of the confirmed decisions: READY. Re-read H1-T0 and
H1-T1 after drafting. H1-T0 requires node-first development and no universal
artifact schema; H1-T1 owns Constitution validation and rule lookup but not
Current Truth, Intent, or Router. This task consumes those boundaries and does
not generalize the Constitution index to other nodes. NO CONFLICT.

Final direct-document review: READY after clarifying that Risk Router selects
the next path for an evidence gap and that explicit precedence retains the
lower-priority claim as a sourced observation.
