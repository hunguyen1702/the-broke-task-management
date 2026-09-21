---
id: H1-T12
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - H1-T1
  - H1-T11
---

# H1-T12: Establish Intent workflow

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
defines the lifecycle. Intent turns a new user work request into a concise,
user-confirmed interpretation that gives Current Truth a sufficiently specific
information need and scope. It does not select a route, gather Current Truth,
approve a commitment, or execute work.

## Entry and trigger

Keep `AGENTS.md` a short entry telling agents that the repository uses the
harness and to consult effective Constitution rules before handling work;
reconsult when relevant scope or rules change. The entry does not encode
Intent's trigger or workflow. An effective Constitution entry rule owns the
trigger, and a separate Intent workflow rule owns interpretation and handoff.
Do not use a hook for this initial node.

Run Intent when a new work intent begins and no confirmed interpretation for
it exists: normally the first substantive request in a session, or a new goal
or material goal/scope change in an ongoing session. An ordinary answer,
correction, status request, or continuation updates the current conversation
without restarting Intent. A Current Truth `insufficient_query` returns to
the same Intent for a targeted clarification. If the resulting interpretation
changes materially, present it for confirmation again. Reconsult effective
rules when the target paths or material rules change; do not reload the full
rule set for every message.

## Interpretation and clarity

Use the user's request and available conversation context. Infer goal and
scope when they are sufficiently clear. A request is clear when the agent can
state the desired outcome, identify scope specific enough for Current Truth to
choose relevant sources, preserve stated constraints and output preference,
and sees no remaining plausible interpretation that would materially change
the result. Intent need not know the implementation approach or all repository
facts. Harmless missing details may be stated as assumptions or left to later
nodes; do not make users supply routine implementation choices.

If meaning remains materially ambiguous, ask a focused question describing
the divergent interpretations or missing decision. Apply each answer to the
same Intent and ask again only while a material ambiguity remains. A clear
request that conflicts with an applicable Constitution rule is handled by the
Constitution/Current Truth stop boundary, not silently reinterpreted by Intent.
Scope expansion or a change to an approved contract requires the user's
decision under the authority rule even if the words are otherwise clear.

The workflow must express this decision path as a Mermaid flowchart, with
short instructions beside it. The flowchart covers entry, clarity, focused
clarification, interpretation display, correction, explicit confirmation,
`ready` handoff, and a return from Current Truth `insufficient_query`.
Keep each decision label specific enough to act on; do not turn the diagram
into a catalog of every possible user message.

Clarity examples to include in the workflow:

- **Bad:** Treat “add a dashboard” as `ready` when the target surface could
  reasonably be a web app or an editor view.
- **Good:** Ask which surface is intended, then assess clarity again. For
  “update the named document's summary,” infer routine editing details and
  proceed to the confirmation display without asking how to edit Markdown.

## User-visible confirmation

For every new interpretable work intent, show a brief interpretation before
Current Truth, Router, or execution:

```text
Context: <relevant background and scope from the request/conversation>
Task: <desired outcome or action>
Format: <requested output form, or a concise default if unspecified>
```

Ask the user to confirm that interpretation. Do not treat silence, elapsed
time, a tool result, or the agent's own confidence as confirmation. A user
correction updates the interpretation and is shown again; a confirmation
reply confirms the existing intent and does not trigger a new one. The
confirmation is a comprehension gate only; it does not authorize a later
irreversible action or replace the Commitment Gate. Established work
continues without repeating this display unless the goal/scope materially
changes.

The display must be brief, concrete, and limited to relevant context. The
workflow includes paired bad/good examples for the confirmation text:

```text
Bad:
Context: The repository has many components and there may be several ways
to improve it, including possible future integrations and architecture work.
Task: Analyze all possibilities and propose a comprehensive solution.
Format: A detailed report covering every option.

Good (for “shorten the setup guide”):
Context: The repository's setup guide.
Task: Shorten the guide while keeping the steps needed to get started.
Format: Proposed default — edit the guide and give a brief change summary.
```

Do not invent scope, constraints, or a requested format while rewriting. If
the user specified no format, state a concise default as a proposal rather
than presenting it as the user's requirement.

## Outcomes and handoff

- `needs_clarification`: the current interpretation, the material ambiguity,
  and one focused question. Remain in Intent; do not hand off downstream.
- `ready`: a confirmed Context–Task–Format interpretation, the information
  need and target scope required by Current Truth, stated constraints, and
  any nonblocking assumptions. Pass these to Current Truth, then to Risk
  Router through the existing lifecycle. No durable prompt-rewrite artifact
  or shared schema is required.

An interpretable but unconfirmed request is pending the user-visible
confirmation, not `ready`. Cancellation or replacement of active work is a
conversation control event, not a third Intent outcome.
The workflow pairs each outcome with a bad/good example: a vague “please
clarify” versus a question naming the material ambiguity; and an
unconfirmed or embellished rewrite versus a concise confirmed handoff with
a specific information need and scope.

## Implementation guidance

Add short framework-origin Constitution rules for workflow entry and Intent
under `.harness/workflow/`, with responsibilities separated as above. Keep
the rules generic and use the required embedded Mermaid flowchart, short
actionable explanation, and paired bad/good examples for clarity decisions
and both outputs;
avoid repository-specific task paths or a mandatory script. Update only the
short `AGENTS.md` lookup pointer needed to discover effective rules.

During initial H1 construction, the user's bounded authorization in
[H1-T1](H1-T1-implement-constitution.md) permits the necessary framework
workflow rules without a separate approval round per node. Do not edit the
pinned 1.1.0 snapshot under its old version: choose the next ruleset version,
recompute the framework digest, update matching rule provenance, rebuild the
index, and validate. Do not activate project-origin drafts, change unrelated
rules, or implement H1-T24's general rule-management workflow.

## Verification and completion

Walk through: a clear first request; a materially ambiguous request and
clarification loop; correction and redisplay before confirmation; explicit
confirmation followed by Current Truth handoff; an ordinary continuation;
a new goal in the same session; a material scope change; and a Current Truth
`insufficient_query` return. Verify no downstream analysis or action precedes
confirmation, that no per-message full rule reload is required, and that
the flowchart matches the prose and Context–Task–Format stays concise. Check
that bad/good examples teach material ambiguity and both outcome shapes
without introducing extra requirements. Verify rule lookup, version/digest/index
consistency, and a successful Constitution validation. Run format, lint, and
test commands if source or tooling changes; for documentation-only workflow
work, check links, examples, validation, and `git diff --check`.

H1-T12 is complete when agents can discover the entry rule, invoke Intent at
the defined boundary, reach `needs_clarification` or user-confirmed `ready`,
and hand a specific need and scope to Current Truth without taking Router or
Commitment Gate authority. Record acceptance impact at implementation
completion; do not edit or execute product acceptance scenarios here.

## References and planning review

- [H1-T0 validated flow](H1-T0-validate-working-flow.md)
- [H1-T1 Constitution contract](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth contract](H1-T11-current-truth.md)
- [Current Truth workflow](../../.harness/workflow/rule-current-truth-r1.md)
- [Constitution lookup](../../.harness/README.md)

Planning decisions: Intent runs per new work intent rather than per message;
the effective entry rule triggers the workflow; a concise
Context–Task–Format interpretation requires explicit confirmation; only
`ready` and `needs_clarification` are Intent outcomes. Independent decision
reviews found the trigger testable and identified the H1 short-path and H1-T1
pinning conflicts; the parent and H1-T1 addenda resolve those conflicts.
The user also required a flowchart-led workflow with short, focused
explanations and paired bad/good examples for evaluation logic and outputs.
Final direct-document review: READY. H1-T0, H1-T1, and H1-T11 were re-read
after drafting. H1-T0 permits an owning node to define its approval boundary;
H1 now makes this one Intent confirmation explicit while preserving short
paths after it. H1-T1 owns Constitution lookup and pin integrity; its initial
construction addendum permits necessary H1 workflow-rule additions without
silently editing version 1.1.0. H1-T11 consumes a specific information need
and scope and returns `insufficient_query` to Intent. This task creates no
competing truth store, route selector, or product contract. NO CONFLICT.
