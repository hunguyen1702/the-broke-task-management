---
id: H1-T0
kind: implementation_task
planning_status: done
implementation_status: done
depends_on: []
---

# H1-T0: Validate the working flow

## Parent

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md).

## Result

The user approved the flow recorded in H1: Intent, Constitution and Current
Truth feed a Risk Router; direct work proceeds to execution; missing context,
multiple viable directions or blocking unknowns take Research, Explore or
Spike and return a Context Capsule to the Router; substantial clear work takes
Plan and a Commitment Gate; execution produces Proof of Work; Learning
Promotion may feed Current Truth, Constitution or automated checks.

Build the nodes one at a time. A logical output need not have dedicated durable
storage. Do not standardize output inventories, metadata, or shared components
ahead of a demonstrated need. Constitution is the first component; its indexed
rule format is local to Constitution, not a schema for all nodes. The
former H1-T2–H1-T10 component roadmap is archived, not an active source of
implementation work. New node entries are planning placeholders until their
own contracts are approved.

Short paths, rerouting, stop/approval boundaries, and failures remain subjects
for each owning node's plan. This validation changes no product or source
behavior and does not create or execute acceptance scenarios.
`implementation_status: done` means this architecture-validation task was
completed by an approved documented decision; it does not assert that harness
runtime code or the workspace test suite was implemented or run.

## Dependency cross-check

There are no direct H1-T0 dependencies. H1-T1 consumes only the approved flow
and the Constitution boundary, not the old universal output taxonomy. NO CONFLICT.

## Acceptance impact

`none`: planning-only architecture decision, with no implemented user-facing
behavior.
