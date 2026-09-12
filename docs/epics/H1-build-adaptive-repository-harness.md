---
id: H1
kind: engineering_epic
planning_status: done
implementation_status: not_planned
contract_depends_on: []
priority: first
---

# H1: Build the adaptive repository harness

## Outcome

The repository gains a complete adaptive harness that helps coding agents select and execute the smallest credible workflow for routine changes, novel features, design exploration, technical spikes, planning, diagnosis, implementation, verification, and learning.

The harness runs as the repository's engineering operating layer. It does not change `tbtm` product behavior or become part of the CLI product contract.

## Priority

Plan and implement H1 before any remaining product-story planning, including E8. Plan one H1 task at a time so decisions from each component can constrain the next task without pre-authoring speculative implementation detail.

## Adaptive lifecycle

The complete lifecycle is:

```text
Sense → Route → Explore → Decide → Commit → Verify → Learn
          ↑________________ reroute as evidence changes ________________|
```

Simple work may take a short path through the lifecycle. The complete architecture must support every stage without forcing every request through every artifact, approval, review, or automation.

## Design artifact

The interactive [Adaptive Repository Harness visualization](../artifact/adaptive-repository-harness.html) is the shared design aid for the lifecycle, categories, patterns, and candidate repository surfaces. It supports planning discussion but is not a canonical rule source; approved H1 task contracts and the resulting harness documents remain authoritative.

## Design principles

- Build the harness inside this repository, while separating reusable protocol from `tbtm`-specific policy and learned enforcement.
- Use the adaptive lifecycle as the runtime backbone, patterns as composable capability modules, and categories as an architectural coverage check.
- Apply every researched pattern only where it has a defined responsibility and runtime interface; a pattern does not require a dedicated file, script, hook, or mandatory gate.
- Let the router select the smallest credible route and reroute when risk, uncertainty, scope, or evidence changes.
- Keep semantic rules canonical in human-readable contracts. Scripts enforce only deterministic projections of those rules and emit traceable evidence.
- Start runtime checks ephemerally. Promote a check into a durable script, test, lint, hook, skill, or instruction only through the learning policy and supported evidence.
- Preserve repository-specific authority, product-contract, implementation, acceptance-scenario, concurrency, and verification boundaries.
- Introduce subagents only when independent context, evidence, or review materially improves the selected route.

## Harness categories and patterns

| Category | Patterns in scope |
|---|---|
| Governance | Repository Constitution, authority boundaries, stable core and adaptive edge |
| Reasoning | Intent and Risk Router, uncertainty and reversibility assessment, smallest credible route |
| Knowledge | Progressive Artifact Graph, Current Truth plus Change Delta, Context Capsule, Evidence Ledger |
| Adaptive work | Composable Playbooks, bounded spikes, conditional subagents, rerouting |
| Control | Risk-Based Gates, commitment boundary, Drift Check, Stop Conditions |
| Verification | Proof-of-Work Contract, automated checks, runtime observation |
| Learning | Learning Promotion, precedent, durable enforcement, stale-guidance retirement |

## Component task roadmap

| Task | Component | Primary deliverable | Depends on |
|---|---|---|---|
| H1-T1 | Constitution and harness architecture | Canonical governance, authority, source-of-truth, and generic-versus-repository boundaries | — |
| H1-T2 | Sense and context assembly | Current truth, change delta, artifact graph, conflict discovery, and context capsule | H1-T1 |
| H1-T3 | Intent and Risk Router | Observable routing signals, routes, rationale, and rerouting contract | H1-T1, H1-T2 |
| H1-T4 | Adaptive exploration and playbooks | Composable direct, research, explore, spike, plan, and diagnosis playbooks | H1-T3 |
| H1-T5 | Decisions and risk gates | Decision artifacts, evidence ledger, contract delta, and proportional gates | H1-T2, H1-T3, H1-T4 |
| H1-T6 | Commitment and drift control | Authorized scope, commitment boundary, stop conditions, and deterministic drift projections | H1-T1, H1-T5 |
| H1-T7 | Verification and proof of work | Change classification, proof contracts, checks, observation, and evidence collection | H1-T3, H1-T6 |
| H1-T8 | Learning and promotion lifecycle | Lesson classification, promotion/retirement criteria, and provenance | H1-T7 |
| H1-T9 | Complete runtime integration | End-to-end state, handoffs, short paths, rerouting, recovery, and traceability | H1-T2–H1-T8 |
| H1-T10 | Scenario validation and refinement | Contrasting scenario evaluation and evidence-based simplification | H1-T9 |

Each task receives its own planning session and task contract before implementation. The roadmap fixes responsibility and dependency order but deliberately leaves file layout, schemas, commands, and automation choices to those sessions.

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

- The user approved building the complete adaptive flow and applying the researched patterns through component tasks.
- H1 is the first planning priority ahead of all remaining product-story plans.
- The epic is approved only at the architecture and decomposition level. Every H1 task remains `planning_status: needed` until its dedicated planning session is completed.
