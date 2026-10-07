---
id: H1-T23
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T15
  - H1-T16
  - H1-T17
  - H1-T18
  - H1-T22
---

# H1-T23: Validate the integrated adaptive flow

## Parent and outcome

[H1](../epics/H1-build-adaptive-repository-harness.md) requires the installed
adaptive lifecycle to work across routine, novel, exploratory, diagnostic,
high-risk, rerouting and learning work, with justified short paths. This
checkpoint assesses the composed flow against that approved contract and
reports evidence, integration defects and unnecessary ceremony.

This is one engineering validation unit under H1, not a new product story or
workflow node. The product PRD supplies no H1 behavior and needs no change.
The user approved validation only, with remediation handled separately, and
authorized continuing the existing H1-T23 planning claim. Planning approval
does not authorize executing this checkpoint.

## Basis and prerequisites

- Use the current H1 lifecycle and effective canonical rules selected through
  Constitution lookup. Record the observed ruleset version, relevant revisions,
  date and working-tree state; a version alone does not identify uncommitted
  changes. Reinspect affected sources if that baseline changes.
- H1-T15, H1-T16, H1-T17, H1-T18 and H1-T22 must be implemented. Their
  transitive dependencies cover Constitution, Current Truth, Intent, Router,
  Direct, staged loading, Execute and Proof of Work; do not add sibling or
  historical tasks merely as scheduling blockers.
- Read historical contracts with their explicit revisions: TD-0008 and H1-T20
  establish Plan → Execute; the old Commitment Gate remains deferred.
  H1-T18B replaces eager core loading, and H1-T22 replaces PoW's previously
  absent learning successor. Historical text does not restore those old paths.
- The validator and `inspect-workflow` are deterministic structural tools,
  not lifecycle interpreters or proof of user approval. Node decisions and
  authority must also be assessed semantically.

## Scope and boundaries

Validate the existing nodes, selective lookups, information handoffs, returns,
stops, proportional proof and learning authority as one composed system.
Assess whether duplicated discovery, unnecessary approvals or artifacts add
cost without improving the approved outcome. Report concrete observations
rather than inventing a score or universal time/token budget.

Do not change runtime rules, manifest, index, tooling, product code or contracts
to make validation pass. Do not build a harness runner, session tracker,
shared capsule schema, scenario registry or mandatory per-task evidence ledger.
Do not create, edit, review or execute anything under `docs/testing/`, assign
product acceptance scenario IDs, or change its catalog or run summary.

H1-T24 rule management and general upgrade/reinstall stay deferred. Validate
the installed flow's precise handoff when a destination is unavailable, without
implementing or claiming to validate that destination. Findings needing
contract changes, rule revisions or enforcement enter their owning planning
and approval boundary separately.

## Deliverables and evidence

1. One dated results report at
   `docs/handoff/H1-T23-integrated-flow-validation-results.md`, linked from this
   task at checkpoint completion. It is evidence for this bounded validation,
   not a canonical rule, reusable scenario catalog or new source of policy.
2. A concise mapping from the coverage dimensions below to governing sources,
   actual observations and their limits. Distinguish:
   - command/test evidence, with exact command, exit code and relevant output;
   - semantic walkthrough evidence, with the supplied context, actual agent
     decision/handoff trace and independent assessment against cited rules;
   - actual execution evidence, when used, with environment, effects and checks.
   A described expected path alone is not an observed walkthrough. A scripted
   lookup pass does not prove an agent honored an authority or return boundary.
   Hypothetical user replies and simulated side effects must be labelled;
   neither authorizes a real write or proves that effect occurred.
3. Findings, if any, naming the failed obligation, source, evidence, consequence,
   owning node/artifact, blocker versus improvement, and proposed next boundary.
   Distinguish an installed integration defect from stale history or an
   environment/access limitation. Do not silently redefine expected behavior.
4. A bounded verdict and updated task/epic/dashboard state. Record actual
   acceptance impact at completion; planning does not preassign scenario impact.

Reuse fresh existing evidence only when its exact coverage and baseline are
applicable. Missing observations remain gaps. The checkpoint report's owner is
H1-T23; a relevant rule, contract or environment change makes its affected
conclusions require revalidation. Retain the dated report as history and link
later evidence rather than treating an old pass as current truth.

## Coverage and acceptance criteria

The following are harness verification dimensions, not product acceptance
cases. Choose the smallest set of coherent walkthroughs that demonstrates all
dimensions without running every path as a separate artifact.

| Dimension | Required integrated evidence |
|---|---|
| Entry and loading | New intent and ordinary continuation are distinguished; bootstrap keeps Intent, then loads Current Truth and Router at their permitted handoffs. Each selected workflow/subprocess is looked up independently. Already-read context is not claimed evicted. |
| Routine short path | Router → Direct → Execute → PoW → Learning works from an established bounded basis, preserving constraints, authority and outcome proof without a mandatory plan, extra ordinary approval or saved capsule. |
| Novel planned work | Clear coupled work selects Plan at the smallest appropriate level, preserves source/assumption distinctions, confirms artifact generation, reviews the right package and reaches Execute only for an authorized actionable unit. Planning-only delivery stops before Execute/PoW/Learning. Epic, Story and Task package boundaries and conditional Story-to-Task composition remain intact. |
| Investigation and exploration | Research's bounded missing-fact investigation and Explore's viable-choice comparison each return sourced evidence or a precise gap through affected Current Truth to Router. Neither chooses a sibling route or executes its recommendation; user-owned choices stop for authority. |
| Feasibility and diagnosis | Spike's single bounded probe, Docker isolation, fallback approval, conclusion limits and retention rules survive the return to Current Truth/Router. Diagnostic evidence leads to the owning next action, not automatic production changes or a claim that POC evidence completes implementation. |
| Risk, stops and rerouting | High-risk work exposes action-specific authority; invalid Constitution and rule conflict stop at their owner, insufficient query returns to Intent, and only ready/routable unresolved Current Truth enters Router. Material evidence/risk changes refresh truth and reroute; material goal/scope drift returns to Intent. No route bypasses a stop. |
| Execute and proof loop | Both action bases reach the same PoW boundary. Local defects return to Execute, missing proof is obtained only with applicable authority, stale evidence is revalidated proportionally, required unavailable proof stays blocked, and partial effects are preserved/disclosed. Green checks cannot waive an unmet outcome. |
| Learning and completion | Only accepted proof enters Learning. None or existing coverage permits completion without a saved lesson; useful proposals await concrete approval before every durable write. Pending/rejected proposals do not undo accepted unit completion. An unavailable owner produces a precise handoff, and new evidence undermining proof returns to its owner. |
| Proportionality and ownership | Consequential claims have a source and owner; capsules/results remain ephemeral unless justified and authorized. Report repeated discovery, duplicated facts, avoidable gates or artifacts with their concrete cost and governing boundary. No automatic promotion or policy rewrite is inferred. |

Functional completion requires traceable evidence or an explicit gap for every
dimension, an honest verdict, and owned actionable findings. Non-functional
requirements are bounded validation, concise reproducible evidence, preservation
of shared dirty-worktree changes, clear simulated-versus-observed claims and
no competing semantic authority or generic infrastructure.

An independent reviewer checks the recorded semantic traces against the actual
effective rules and H1, including forbidden transitions and authority claims.
Agent agreement is a review signal, not a substitute for runtime observations.
This review belongs to the integration checkpoint; it does not introduce a
mandatory reviewer into ordinary harness work.

## Execution approach

1. In a separately authorized execution session, claim this ready task in its
   frontmatter, H1 and dashboard, and reread the claim before work. Capture the
   existing dirty-tree baseline; preserve pre-existing changes throughout.
2. Validate Constitution and inspect context with the relevant target paths.
   Follow the staged handoffs and load only each required effective chain.
   Reconcile H1, terminal dependency contracts and explicit later decisions;
   record unresolved conflicts rather than choosing a convenient interpretation.
3. Run the existing deterministic checks. The Constitution suite already uses
   isolated temporary copies for invalid-input and lookup tests; reuse it rather
   than corrupting the installed snapshot or adding tests that mirror rule prose.
4. Perform bounded recorded semantic walkthroughs over the coverage dimensions.
   Supply explicit fixture context and capture the agent's actual decisions and
   handoffs. Use isolated copies for fixture mutations and retain no production
   side effects. Simulate authority stops without inventing human approval.
   Follow Spike's actual Docker/fallback and retention rules if a real probe is
   used; inability to obtain required execution evidence is a disclosed gap.
5. Map fresh existing or newly obtained evidence to the dimensions. Identify
   precise findings and their owners. Do not repair defects or expand scope;
   disclose any partial effects and record the separate remediation handoff.
6. Obtain independent evidence review, resolve report inaccuracies, and rerun
   only affected validation when the baseline or evidence changes. A true
   implementation defect remains a finding rather than being edited here.
7. Write the final report, verify repository preservation and report links,
   classify actual acceptance impact, and update completion state consistently.
   Do not mark H1 complete or begin E8 merely because this task is done.

## Verification commands

Run at checkpoint handoff, recording actual outcomes:

```bash
rtk proxy .harness/scripts/validate-constitution validate
rtk mise run format
rtk mise run lint
rtk mise run test
rtk git diff --check
```

`mise run test` runs workspace tests and the Ruby Constitution suite. No
separate duplicate suite run is required unless focused follow-up is justified.
Use `inspect-context` and `inspect-workflow` with the current relevant relative
paths at actual handoffs; deterministic selection does not assert lifecycle
ordering. Compare final status/diff with the captured baseline and allow only
this task's report and authorized lifecycle-status updates.

## Verdict and definition of done

Checkpoint evidence: [2026-10-06 integrated-flow validation results](../handoff/H1-T23-integrated-flow-validation-results.md).
Acceptance impact: `none` (harness-only validation and documentation; no product behavior changed).

Report `verified` only when required checks and coverage have sufficient current
evidence and no unresolved blocking integration defect. Report `findings` when
validation establishes a defect, and `blocked` when required evidence cannot
be obtained. These are report meanings, not a new stored state machine.

The checkpoint task may be `done` once bounded validation, independent review,
evidence/gap reporting and owned handoffs are complete, even with a findings or
blocked verdict. Unattempted required work is not completion. Required check
failures remain explicit failed evidence; they cannot yield `verified`. This
distinguishes finishing an assessment from proving the lifecycle ready.

H1 remains `in_progress` unless its complete governing criteria are separately
assessed and satisfied. Deferred H1-T24 and general upgrade/reinstall are not
claimed complete. No runtime fix, product behavior change or acceptance-scenario
work is part of H1-T23.

## References and planning review

- [Parent H1](../epics/H1-build-adaptive-repository-harness.md)
- [Research](H1-T15-research.md), [Explore](H1-T16-explore.md),
  [Spike](H1-T17-spike.md), [Plan](H1-T18-plan.md),
  [Learning Promotion](H1-T22-learning-promotion.md)
- [Staged loading](H1-T18B-staged-core-workflow-loading.md),
  [Execute](H1-T20-execute.md), [Proof of Work](H1-T21-proof-of-work.md)
- [TD-0008: deferred Commitment Gate](../decisions/TD-0008-defer-h1-commitment-gate.md)
- [Deferred work](../handoff/H1-deferred-work.md)
- [Constitution lookup](../../.harness/README.md)
- Existing deterministic suite: `.harness/scripts/test-constitution`;
  repository verification: `mise.toml`.

The user confirmed the validation-only scope, artifact set and continuation
of the existing claim. Independent decision review: `READY`. After drafting,
H1-T15, H1-T16, H1-T17, H1-T18 and H1-T22 were re-read against parent H1.
Read-only research, recommendation versus authority, Docker/fallback/retention,
planning package ownership, proof and learning boundaries remain unchanged.
TD-0008 and the later staged-loading/Execute/PoW contracts reconcile historical
handoffs. Dependency cross-check: `NO CONFLICT`. Fresh independent
direct-document review: `READY`, with no material blocker. All direct
dependencies are implemented, so H1-T23 is `ready`. No integration walkthrough
or checkpoint execution was performed during planning.
