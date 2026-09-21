---
id: H1
kind: engineering_epic
planning_status: done
implementation_status: in_progress
contract_depends_on: []
priority: first
---

# H1: Build the adaptive repository harness

## Outcome

The repository gains a complete adaptive harness that helps coding agents select and execute the smallest credible workflow for routine changes, novel features, design exploration, technical spikes, planning, diagnosis, implementation, verification, and learning.

The harness runs as the repository's engineering operating layer. It does not change `tbtm` product behavior or become part of the CLI product contract.

## Priority

Plan and implement H1 before any remaining product-story planning, including E8. H1-T0 validated the working flow. Plan one node at a time so actual interfaces and needs constrain the next task without pre-authoring shared implementation detail.

## Adaptive lifecycle

The validated working flow for node-by-node development is:

```text
Intent + Constitution + Current Truth → Risk Router
  → Direct → Execute → Proof of Work → Learning Promotion
  → Research / Explore / Spike → Context Capsule → Risk Router
  → Plan → Commitment Gate → Execute → Proof of Work → Learning Promotion
Learning Promotion may update Current Truth, Constitution, or checks through
their respective authority boundaries.
```

Simple work may take a short path through the lifecycle. The complete architecture must support every stage without forcing every request through every artifact, approval, review, or automation.

## Design artifact

The interactive [Adaptive Repository Harness visualization](../artifact/adaptive-repository-harness.html) records earlier exploration. Its old lifecycle and pattern map are historical, non-authoritative references; H1-T0 and subsequently approved node contracts govern the current direction.

## Design principles

- Build the harness inside this repository, while separating reusable protocol from `tbtm`-specific policy and learned enforcement.
- Use the H1-T0 flow as the working backbone and plan each node from its actual inputs, decisions, actions, outputs, and return/stop conditions.
- Treat researched patterns as optional design references, not mandatory components or a coverage checklist.
- Let the router select the smallest credible route and reroute when risk, uncertainty, scope, or evidence changes.
- Keep semantic rules canonical in human-readable contracts. Scripts enforce only deterministic projections of those rules and emit traceable evidence.
- Start runtime checks ephemerally. Promote a check into a durable script, test, lint, hook, skill, or instruction only through the learning policy and supported evidence.
- Preserve repository-specific authority, product-contract, implementation, acceptance-scenario, concurrency, and verification boundaries.
- Introduce subagents only when independent context, evidence, or review materially improves the selected route.

## Earlier design vocabulary

The following is a reference to prior research, not a task breakdown or a
requirement to create shared artifacts. Adopt a pattern only when an owning
node demonstrates a concrete need.

| Category | Patterns in scope |
|---|---|
| Governance | Repository Constitution, authority boundaries, stable core and adaptive edge |
| Reasoning | Intent and Risk Router, uncertainty and reversibility assessment, smallest credible route |
| Knowledge | Progressive Artifact Graph, Current Truth plus Change Delta, Context Capsule, Evidence Ledger |
| Adaptive work | Composable Playbooks, bounded spikes, conditional subagents, rerouting |
| Control | Risk-Based Gates, commitment boundary, Drift Check, Stop Conditions |
| Verification | Proof-of-Work Contract, automated checks, runtime observation |
| Learning | Learning Promotion, precedent, durable enforcement, stale-guidance retirement |

## Active node-first roadmap

| Task | Node or checkpoint | Planning state |
|---|---|---|
| H1-T0 | Validate the working flow and node-first boundary | done |
| H1-T1 | Constitution | done; implementation done |
| H1-T11 | Current Truth | done; implementation done |
| H1-T12 | Intent | needed |
| H1-T13 | Risk Router | needed |
| H1-T14 | Direct path | needed |
| H1-T15 | Research | needed |
| H1-T16 | Explore | needed |
| H1-T17 | Spike | needed |
| H1-T18 | Plan | needed |
| H1-T19 | Commitment Gate | needed |
| H1-T20 | Execute | needed |
| H1-T21 | Proof of Work | needed |
| H1-T22 | Learning Promotion | needed |
| H1-T23 | Integrated flow validation | needed |
| H1-T24 | Manage Constitution rules (deferred workflow) | needed |

H1-T12–H1-T24 are backlog placeholders, not approved implementation contracts.
Their precise scope, sequence, and dependencies are decided when each node is
planned. A Context Capsule is the candidate return output of Research, Explore,
and Spike, not a preselected separate component. Tests, lints, and skills are
possible Learning Promotion targets, not folders or systems prescribed here.
Extract common mechanics only after concrete nodes demonstrate repeated need.
The [former H1-T2–H1-T10 roadmap](../handoff/H1-archived-component-roadmap.md)
is archived for reference and must not be claimed.

## Deferred work

- H1-T24 owns the future workflow for proposing, reviewing, approving,
  activating, revising, superseding, and retiring Constitution rules. H1-T1
  supplies the rule data contract and validator, not this operational workflow.
- Framework ruleset upgrade/reinstall and review of project references are
  deferred. They are recorded in the [deferred-work register](../handoff/H1-deferred-work.md)
  but do not yet need implementation task IDs or a planned dependency graph.

## Completion criteria

1. The repository can execute and explain the full adaptive lifecycle while allowing justified short paths.
2. Routing uses observable intent, risk, uncertainty, reversibility, and repository-state signals.
3. Every durable artifact has one owner, lifecycle, source-of-truth relationship, and retirement rule.
4. Gates are proportional to commitment and risk rather than globally mandatory.
5. Completion claims are backed by an explicit proof-of-work contract and recorded evidence.
6. Runtime discoveries can remain ephemeral or be promoted through a controlled learning process.
7. Durable checks trace back to canonical semantic rules and do not silently redefine them.
8. The integrated flow is validated against routine, novel, exploratory, diagnostic, high-risk, rerouting, and learning scenarios.
9. Evaluation identifies and removes ceremony or duplicated artifacts that do not improve outcomes.
10. Existing `tbtm` product contracts and acceptance-scenario ownership remain unchanged unless separately planned and approved.

## Out of scope

- Applying the unfinished harness to E8 planning or implementation.
- Changing `tbtm` CLI behavior, persistence, migrations, or public output as part of H1.
- Prebuilding a large catalog of repository scripts, hooks, or skills before runtime evidence supports them.
- Requiring orchestration or subagents for ordinary work.
- Creating, editing, revalidating, approving, or executing acceptance scenarios during H1 planning or implementation.
- Extracting a standalone cross-repository harness before reuse and interface stability have been demonstrated.

## Planning record

- The user approved building the adaptive flow node by node; researched patterns remain optional references.
- The user approved adding H1-T0 to validate the lifecycle, representative workflows, stage requirements, and component decomposition before detailed H1-T1 planning.
- H1-T0 validated the user-approved working flow and node-first development boundary; Constitution is the first independently planned component.
- H1-T1 Constitution planning is complete in `../tasks/H1-T1-implement-constitution.md`; its rule format and index apply to Constitution alone, with no preselected folder structure for other nodes. Dependency cross-check against H1-T0: NO CONFLICT.
- H1-T1 implementation is complete. The pinned Constitution validates locally; acceptance impact is `none`.
- H1-T11 Current Truth planning is complete in `../tasks/H1-T11-current-truth.md`. Intent supplies a sufficiently specific information need; the node builds a sourced, ephemeral view or an unresolved result without selecting a route or creating a second ADR store. Dependency cross-check against H1-T0 and H1-T1: NO CONFLICT.
- H1-T11 implementation is complete in `.harness/workflow/rule-current-truth-r1.md`. The generic Constitution workflow supplies a request-scoped, source-backed `ready` or `unresolved` handoff; acceptance impact is `none`.
- The former H1-T2–H1-T10 component roadmap was archived; H1-T12–H1-T23 remain node-first planning placeholders and H1-T24 tracks the deferred Constitution rule-management workflow.
- H1 is the first planning priority ahead of all remaining product-story plans.
- The epic is approved at the architecture and decomposition level. Remaining H1 placeholders stay `planning_status: needed` until their dedicated planning sessions begin, then follow the normal `in_progress` and `done` planning lifecycle.
