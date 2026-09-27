---
id: H1-T17
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T1
  - H1-T11
  - H1-T12
  - H1-T13
---

# H1-T17: Establish Spike workflow

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
defines Spike as the uncertainty-reduction path for a consequential feasibility
assumption. Given a confirmed Intent, relevant Current Truth, and a Risk Router
handoff naming that assumption, an agent runs one bounded, isolated probe and
returns evidence, limitations, and a conclusion to Router. Spike neither
compares competing solutions nor turns proof-of-concept code into production
code.

## Inputs and boundaries

Before execution, state:

- one consequential feasibility assumption and one probe direction;
- the evidence that would support or refute the assumption;
- the time, resource, and scope bounds; and
- the stop conditions, including evidence that would make the result
  inconclusive.

The agent may run an in-scope probe without another ordinary approval gate.
It must stop for user authority when the probe would expand the confirmed
scope, use unavailable authority, affect an external system, or cross another
applicable safety boundary. Several viable approaches requiring comparison
belong to Explore: return that finding to Router rather than spawning agents
to spike multiple solutions. A subagent may assist with independent evidence
or review when that materially helps, but H1-T17 does not fan out one solution
probe per option and does not treat agent agreement as feasibility evidence.

## Isolation and repository integrity

Docker is the required default isolation boundary. Use a disposable container
workspace and either mount the repository read-only or copy only the inputs
needed by the probe into that workspace. Probe code, generated output, caches,
and mutations stay outside the repository mount. Capture repository status or
diff evidence before and after the probe to show that the working tree was not
changed by Spike.

If Docker is unavailable or cannot provide the required isolation, stop before
running the probe. Tell the user why Docker cannot be used, what isolation is
lost, and the proposed fallback; continue only after explicit confirmation.
The confirmed fallback creates a fresh directory with `mktemp -d` under
`/tmp`, verifies that its resolved path is outside the repository, and keeps
all proof-of-concept code and output there. Silence is not confirmation.

Spike never writes production changes to the repository. Its result returns
through affected Current Truth to Risk Router. Only a later workflow that
reaches Execute may modify the working tree.

## Workflow and stop behavior

1. Recheck that the Router handoff names a feasibility assumption rather than
   a missing fact or a choice among viable options. Return to Router when the
   uncertainty instead belongs to Research or Explore.
2. Frame the single probe, evidence thresholds, bounds, and stop conditions.
   Do not broaden the probe merely because related questions appear.
3. Establish and record Docker isolation. If that is unavailable, stop and
   obtain explicit confirmation before creating the `/tmp` fallback.
4. Run the bounded probe, preserving commands and observations sufficient to
   support the conclusion. Stop when a declared limit is reached or continuing
   would cross scope, authority, or safety boundaries.
5. Check repository integrity and classify the result as `supported`,
   `refuted`, or `inconclusive`. A partial observation does not become a
   confident conclusion.
6. Report the result, evidence, limitations, environment, isolation method,
   artifact location or identifier, and retention state. Refresh affected
   Current Truth, then return the Context Capsule to Router for the next route.
7. Apply the artifact-retention rules below. Cleanup is never inferred from
   silence.

The implemented workflow must use an embedded Mermaid flowchart showing entry,
probe framing, Docker and fallback decisions, execution and stops, conclusion,
retention, Current Truth refresh, and Router return. Keep the prose concise and
include paired bad/good examples for selecting one assumption, bounding the
probe, classifying inconclusive evidence, and reporting the result.

## Result and Context Capsule

The report is conversational by default and contains:

- the assumption, probe, declared bounds, and actual environment;
- the isolation method and repository-integrity evidence;
- material commands or observations and their provenance;
- `supported`, `refuted`, or `inconclusive`, with the reasoning that connects
  the evidence to that classification;
- limitations, unknowns, and the specific next question or action for Router;
- every retained Docker or filesystem artifact, its exact path or identifier,
  relevant contents or scope, and current retention state.

The result is evidence for routing, not proof that production behavior is
complete. Refresh the affected Current Truth claims before Router relies on
the result. The Context Capsule may link to a retained POC, but neither the
capsule nor the POC becomes a canonical contract, implementation, or second
source of repository truth.

## Artifact retention and cleanup

- If the result is `inconclusive`, or an artifact still contains evidence
  needed for follow-up, retain it. Tell the user its exact path or Docker
  identifier, why it is retained, what relevant material it contains, and the
  proposed next step.
- If the result is conclusive and an artifact is no longer needed, ask the
  user whether to delete it. Keep it until the user explicitly chooses
  deletion.
- Apply the same disclosure and consent rule to containers, images, volumes,
  bind mounts, temporary directories, and generated output. No reply means no
  deletion.
- After an authorized cleanup, report what was removed and any related
  artifact that remains.

## Implementation guidance

Add one concise framework-origin Constitution workflow rule under
`.harness/workflow/` using H1-T1's pinned ruleset process and initial H1
construction authorization. Determine the next ruleset version from the
current repository state at implementation time; update framework-rule
provenance, digest, and derived index consistently, then validate. Do not edit
an older pinned snapshot in place or activate a project-origin rule.

Keep the workflow tool-neutral beyond the approved Docker-first isolation and
`mktemp -d` fallback contract. Do not add a persistent spike registry, shared
node schema, production POC directory, mandatory multi-agent orchestration, or
product CLI behavior. Do not implement Explore, Execute, Proof of Work, or
Learning Promotion in this task.

## Test and verification plan

- Walk through a supported probe, a refuted probe, and a probe that reaches a
  declared limit and remains inconclusive. Check evidence-to-conclusion logic
  and the Router handoff.
- Verify a Docker probe uses a read-only repository mount or copied inputs,
  keeps mutations outside the repository, and leaves status/diff unchanged.
- Simulate unavailable Docker. Verify the workflow stops, explains the lost
  isolation, and does nothing until explicit confirmation. After confirmation,
  verify a fresh resolved `mktemp -d` path is under `/tmp` and outside the
  repository.
- Verify an unconfirmed fallback, scope expansion, external side effect, or
  safety-boundary crossing does not execute.
- Verify multiple viable options return to Router for Explore rather than
  launching solution-specific Spike agents.
- Verify an inconclusive or still-needed POC is retained with complete
  disclosure. Verify a disposable conclusive artifact remains until the user
  explicitly authorizes deletion, including equivalent Docker artifacts.
- Verify the workflow reports environment, isolation, evidence, result,
  limitations, retained artifacts, and cleanup state; refreshes affected
  Current Truth; and does not claim that POC code is production-ready.
- Validate the Constitution and inspect the effective rule. Run repository
  format, lint, and test tasks required by the repository before handoff.

## Definition of done

An agent can frame and execute one bounded feasibility probe in Docker without
changing the repository, stop for explicit confirmation before an isolated
`/tmp` fallback, classify the evidence without overstating it, preserve or
clean artifacts only under the agreed lifecycle, and return a complete Context
Capsule through Current Truth to Router. The workflow is discoverable through
effective Constitution lookup and passes validation. Record acceptance impact
at implementation completion; do not create, modify, or execute product
acceptance scenarios in this task.

## References and planning review

- [H1-T0 validated lifecycle](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [H1-T13 Risk Router](H1-T13-risk-router.md)
- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)

The user approved autonomous in-scope probes, Docker-first isolation, an
explicitly confirmed `/tmp` fallback, a single probe direction, strict
working-tree isolation, and user-controlled cleanup. Independent decision
reviews found these boundaries clear and testable. After drafting, H1-T0,
H1-T1, H1-T11, H1-T12, H1-T13, H1-T16, and parent H1 were re-read. Spike
preserves the node-first flow, pinned Constitution process, sourced Current
Truth, confirmed Intent, Router ownership, and Explore's option-comparison
boundary. It adds no competing source of truth, route selector, production
mutation path, or universal Context Capsule schema. Dependency cross-check:
`NO CONFLICT`. All direct implementation dependencies are `done`, so H1-T17
implementation is `ready`. Final independent direct-document review found and
then verified corrections to stale status projections. Final review: `READY`.

## Implementation result

The [Spike workflow](../../.harness/workflow/rule-spike-r1.md) tests one
consequential feasibility assumption with one bounded probe in a disposable
Docker workspace, using a read-only repository mount or copied inputs. When
Docker is unavailable, it stops and waits for explicit confirmation before a
resolved `mktemp -d` fallback under `/tmp` outside the repository. It returns
`supported`, `refuted`, or `inconclusive` evidence through refreshed Current
Truth to Router, without comparing alternatives or producing production code.

Manual workflow walkthrough on 2026-09-27: supported and refuted probes require
their declared evidence thresholds, and a probe that reaches a declared limit
without either threshold is reported `inconclusive`. Missing facts return to
Router for Research, and several viable directions return for Explore rather
than spawning solution-specific probes. Unconfirmed fallback, scope expansion,
unavailable authority, external side effects, and other safety-boundary
crossings stop without executing. Inconclusive or still-needed artifacts are
retained with full disclosure; conclusive disposable containers, images,
volumes, directories, and output remain until the user explicitly authorizes
deletion. The capsule reports environment, isolation, integrity evidence,
provenance, conclusion, limitations, artifacts, and the next Router question,
and it states that POC results are not production completion.

The Constitution moved to pinned ruleset 1.7.0 with matching framework-rule
provenance, a recomputed framework digest, and rebuilt index. `validate`,
`inspect-effective`, and `test-constitution` passed, as did repository
format, lint, and tests. Acceptance impact: `none`, because this generic
harness workflow does not change `tbtm` product behavior or a product
acceptance scenario.
