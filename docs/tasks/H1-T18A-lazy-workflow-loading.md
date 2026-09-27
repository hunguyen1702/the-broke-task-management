---
id: H1-T18A
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T1
  - H1-T11
  - H1-T12
  - H1-T13
  - H1-T14
  - H1-T15
  - H1-T16
  - H1-T17
---

# H1-T18A: Establish lazy workflow loading

## Parent and outcome

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md)
currently validates, indexes, and path-filters Constitution rules correctly,
but every installed framework rule uses repository-wide scope. The repository
entry protocol then requires agents to read every effective rule returned by
`inspect-effective`, so Direct, Research, Explore, and Spike enter context even
when the Router selects none of them.

This task separates Constitution integrity from model-context loading. All
canonical rules remain validated, indexed, effective, and pinned. A small core
context is loaded for every request, while a route-specific workflow is loaded
only after Risk Router selects it. The mechanism also supports a routed
workflow selecting a nested on-demand workflow later, which H1-T18 Plan needs
for Epic, Story, and Implementation Task Planning.

## Current behavior and problem boundary

The existing validator intentionally scans every direct Markdown child of the
six Constitution categories, validates its structure and relationships,
computes the framework digest, resolves effectiveness, and checks the derived
index. That full scan is an integrity operation and must remain exhaustive.

`inspect-effective [paths...]` currently returns every effective rule whose
repository scope matches a supplied path. Because all framework workflows use
`scope: {repository: true}`, every workflow is returned for every path. The
entry instructions then require reading each returned file, making workflow
context eager rather than adaptive.

Do not solve this by weakening validation, placing unpinned guides outside the
Constitution, abusing repository paths as route selectors, or assuming that
separate files are lazy by themselves.

## Loading model

Add the optional frontmatter field `contextLoading` to the Constitution schema.
It is required for every workflow-category rule, forbidden on every
non-workflow rule, and accepts exactly:

- `always`: the rule belongs to the bootstrap context when it is effective and
  applies to the target repository paths;
- `on_demand`: the rule remains canonical and effective but is returned for
  model context only when explicitly selected by workflow ID.

The field is independent of `scope`. Scope continues to answer which
repository paths a rule governs; loading classification answers when its body
should enter model context. Effectiveness, extension, supersession, origin,
approval, and ruleset pinning retain their existing meanings.

Non-workflow rules have derived loading classification `always`; do not
introduce route semantics for governance, authority, safety, repository, or
verification categories. The derived index records `contextLoading` for every
rule, using the explicit workflow value or derived `always` value.

The initial classification is:

- `always`: Workflow Entry, Intent, Current Truth, and Risk Router;
- `on_demand`: Direct, Research, Explore, and Spike; and
- `always`: every effective non-workflow rule.

Migration updates all existing workflow frontmatter to the explicit value
above. The new framework snapshot also updates installation provenance on all
framework rules as required by H1-T1; it does not add `contextLoading` to
non-workflow frontmatter.

Later workflow nodes may be `on_demand`. Nested Plan subprocess workflows are
also expected to be `on_demand`, but H1-T18 owns their contracts and files.

## Validator and lookup interface

Preserve the existing commands and add two read-only lookup modes:

```text
validate
rebuild-index
inspect-effective [relative paths...]
inspect-context [relative paths...]
inspect-workflow <workflow-id> [relative paths...]
```

Their responsibilities are:

- `validate` exhaustively validates all canonical rules, loading metadata,
  relationships, digest, and index. An `on_demand` rule receives no integrity
  exemption.
- `rebuild-index` retains its atomic derived-index behavior and includes the
  validated loading classification in each rule entry.
- `inspect-effective` remains the complete diagnostic view of all effective,
  path-applicable rules regardless of loading classification. This preserves
  its current semantic meaning and compatibility for auditing.
- `inspect-context` returns only effective, path-applicable `always` rules.
  Repository startup uses this command, then reads every returned canonical
  file.
- `inspect-workflow` selects one effective `on_demand` workflow by stable
  logical rule ID, checks its applicability to the supplied target paths, and
  returns only the canonical workflow rule chain needed for that selection.
  Core rules are not repeated because they are already present.

Path matching retains the existing `inspect-effective` semantics. With one or
more paths, a rule is applicable when it matches at least one supplied path;
scope mismatch means that neither the selected workflow nor any effective
replacement matches any supplied path. With no paths, scope does not filter
the selected workflow, matching the existing no-path diagnostic behavior.

All successful inspection output keeps the existing stable YAML shape of rule
ID, revision, and path. Unknown IDs, non-workflow IDs, requests for `always`
workflows through the on-demand interface, invalid paths, or scope mismatch
must fail explicitly rather than silently loading an unintended rule. These
are invocation or selection errors on an otherwise valid Constitution: exit
`2`, one stable diagnostic on stderr, and no partial stdout. Invalid loading
metadata or incompatible relationships are Constitution errors with exit `1`.

## Relationships and replacement behavior

On-demand lookup must preserve existing Constitution relationships using this
deterministic algorithm:

1. Resolve the requested logical ID to its single active workflow revision.
   Missing IDs, IDs without an active revision, and non-workflow IDs are
   selection errors.
2. Require that starting revision to be `on_demand`. Follow the unique active
   supersession chain until its effective leaf; cycles and multiple effective
   replacements remain Constitution failures.
3. Apply the path rule above to the effective leaf. No matching supplied path
   is a scope-mismatch selection error.
4. Include every effective, path-applicable workflow rule whose transitive
   `extends` closure reaches the requested revision or a revision on its chosen
   replacement chain. Do not include unrelated extenders or siblings.
5. Return the effective leaf and included extenders in the same stable
   ID/revision ordering used by existing inspection output.

Loading classification is invariant across every `extends` or `supersedes`
relationship, including cross-category references. Compare each rule's
explicit workflow value or derived non-workflow `always` value with its target;
they must match. The validator rejects mismatches with exit `1`. Thus an
`always` non-workflow rule cannot extend or replace an `on_demand` workflow, a
project extension cannot make route-specific behavior enter bootstrap context,
and a replacement cannot make a stable workflow ID switch lookup modes.

The command selects a workflow family by its stable logical rule ID; it does
not accept an arbitrary file path as a workflow selector. This prevents a
caller from bypassing effectiveness, scope, approval, or relationship
resolution by reading a known file directly.

## Repository entry and routing protocol

Update the repository lookup guidance and affected workflow rules to use this
sequence:

1. Validate the complete Constitution.
2. Run `inspect-context` with every currently relevant repository-relative
   path and read the returned core rules.
3. Intent, Current Truth, and Risk Router operate from the core context.
4. After Router selects Direct, Research, Explore, Spike, or a later routed
   workflow, run `inspect-workflow` for that workflow ID and the same relevant
   paths, then read only the returned rule files.
5. If the workflow selects a nested subprocess, repeat `inspect-workflow` for
   that subprocess ID; do not inspect sibling subprocesses.
6. Reinspect core or selected workflow context when target paths, effective
   rules, or the selected route materially change.

Loading is incremental within a conversation. A rule already read cannot be
removed from the model's existing context; rerouting loads the newly selected
workflow without claiming that earlier context was erased. The improvement
prevents unrelated workflows from entering context up front.

## Schema, index, and compatibility boundaries

Extend the Constitution schema and rule parser with the `contextLoading` field
and conditional rules above. Keep the current path scope grammar and do not
add workflow names to `scope`. The derived index always serializes
`contextLoading`, including derived `always` for non-workflow rules, so
inspection remains deterministic and auditable.

Install the change as a new pinned framework snapshot. Update every
framework-rule installation provenance value, the manifest ruleset/framework
version, framework digest, derived index, and documentation consistently.
Validation must reject missing or invalid workflow loading metadata, invalid
category/loading combinations, digest drift in either core or on-demand rules,
and stale indexes.

Existing callers of `inspect-effective` retain the full diagnostic result.
Repository entry guidance moves to `inspect-context`; do not silently redefine
`inspect-effective` to mean bootstrap context.

## Implementation guidance

Keep the implementation in the existing local Constitution validator and
canonical rule files. Update the concise entry documentation in `AGENTS.md`
and `.harness/README.md`, plus any core workflow text that currently instructs
agents to use the full effective-rule list as context.

Do not implement Plan or its three subprocesses, Commitment Gate, dynamic
context eviction, a daemon, a persistent workflow registry, semantic routing
inside the validator, or product behavior. The validator exposes deterministic
lookup; Risk Router continues to own semantic route selection.

## Test and verification plan

- Verify `validate` still parses and checks every core and on-demand rule and
  detects drift in an on-demand rule that has never been selected.
- Verify `inspect-effective` continues to return every effective applicable
  rule, including Direct, Research, Explore, and Spike.
- Verify `inspect-context` returns the non-workflow rules plus Workflow Entry,
  Intent, Current Truth, and Risk Router, but excludes all routed workflows.
- Select each routed workflow independently; verify `inspect-workflow` returns
  that workflow and no sibling workflow.
- Add isolated temporary rules representing a nested workflow and verify it is
  loaded only by an explicit second lookup.
- Verify path include/exclude behavior remains applicable to both bootstrap
  and on-demand inspection without treating paths as route keys.
- Verify an unknown ID, non-workflow ID, `always` workflow ID, invalid target
  path, and scope mismatch fail with deterministic diagnostics and do not emit
  a partial rule list.
- Exercise extension and supersession of an on-demand workflow; verify lookup
  returns the effective replacement and applicable extensions while preserving
  approval and conflict checks.
- Verify direct and transitive extenders use stable ID/revision output order;
  verify any extension or replacement with a different explicit or derived
  loading class fails Constitution validation, including non-workflow
  `always` rules that extend or supersede an `on_demand` workflow.
- Verify index rebuild is stable and atomic and records loading metadata; a
  stale pre-change index must fail validation.
- Walk through Direct, Research, Explore, and Spike routing from a fresh core
  context and verify only the chosen workflow is subsequently read.
- Walk through a reroute and report the honest incremental-context limitation
  without reloading unrelated siblings.
- Verify updated entry instructions never tell agents to read the complete
  diagnostic list during ordinary startup.
- Run Constitution validation, isolated Constitution tests, repository format,
  lint, and tests before implementation handoff.

## Definition of done

The installed Constitution still validates and pins every canonical rule, but
ordinary repository entry reads only effective core rules. Risk Router can
select one route-specific workflow by stable ID, whose effective relationship
chain and path applicability are checked before its files are returned. Direct,
Research, Explore, and Spike no longer enter context unless selected. Nested
on-demand lookup is available for Plan without implementing Plan itself.
Diagnostic `inspect-effective` behavior remains available and unchanged.

Record acceptance impact at implementation completion; do not create, modify,
or execute product acceptance scenarios in this task.

## References and planning review

- [H1-T1 Constitution](H1-T1-implement-constitution.md)
- [H1-T11 Current Truth](H1-T11-current-truth.md)
- [H1-T12 Intent](H1-T12-intent.md)
- [H1-T13 Risk Router](H1-T13-risk-router.md)
- [H1-T14 Direct](H1-T14-direct-path.md)
- [H1-T15 Research](H1-T15-research.md)
- [H1-T16 Explore](H1-T16-explore.md)
- [H1-T17 Spike](H1-T17-spike.md)
- [Parent H1 epic](../epics/H1-build-adaptive-repository-harness.md)
- [Constitution lookup](../../.harness/README.md)

The user identified eager loading of every repository-wide workflow as a gap
against the adaptive-harness goal and approved a separate prerequisite task
without renumbering the existing H1 roadmap. The task preserves exhaustive
validation while separating bootstrap context from route-specific lookup.

After drafting, H1-T1 and H1-T11–H1-T17 were re-read. The new loading field is
orthogonal to repository scope, retains exact-reference relationship semantics,
keeps Router as the semantic route owner, and changes no workflow's behavior.
Direct, Research, Explore, and Spike become on-demand without changing their
inputs, outputs, authority stops, or return paths. Dependency cross-check:
`NO CONFLICT`.

Independent review found and then verified fixes for the exact loading-field
schema, path matching, stable workflow-family resolution, extension and
replacement compatibility, cross-category loading invariants, and exit-code
behavior. Final review: `READY`. All implementation dependencies are `done`,
so H1-T18A implementation is `ready`.

## Implementation completion

Implemented in the pinned 1.8.0 Constitution snapshot. `contextLoading` is
indexed for every rule; `inspect-context` loads the bootstrap set and
`inspect-workflow` resolves one selected on-demand workflow family. Direct,
Research, Explore, and Spike are on-demand, while Workflow Entry, Intent,
Current Truth, Risk Router, and non-workflow rules remain bootstrap context.
The validator preserves full `inspect-effective` diagnostics and rejects
loading-class mismatches across extensions or replacements.

`validate`, Constitution tests, format, lint, and the workspace test suite
passed. Acceptance impact: `none`, because this changes only the repository
harness and no `tbtm` product behavior or acceptance scenario.
