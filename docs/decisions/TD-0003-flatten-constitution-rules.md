---
id: TD-0003
status: accepted
date: 2026-09-21
related_epics: [H1]
related_stories: []
supersedes: [TD-0002]
scenario_impact: none
affected_scenarios: []
---

# Store Constitution rules by category and make workflows canonical

## Context

The original framework/project directory split scattered rules across trees.
Current Truth was implemented as a separate guide, leaving it outside effective
Constitution inspection and embedding this repository's own document names in
a workflow intended for reuse.

## Decision

Store canonical rules together in `.harness/<category>/`. Record `origin` in
rule frontmatter so the validator can preserve framework pinning and project
approval checks without a directory split. Move schema and derived index to
`.harness/`. Pin framework-origin files by SHA-256 over sorted paths relative
to `.harness/`, NUL, file bytes, NUL. Install the generic Current Truth workflow
as an active framework-origin rule in `.harness/workflow/`. Future workflows
use that category and embed concise flowcharts and concrete examples.

## Observable behavior

Effective inspection now returns Current Truth and paths under the flattened
layout. `tbtm` CLI behavior does not change.

## Consequences

The manifest ruleset advances to 1.1.0 and its digest changes. Existing
Constitution paths and index entries migrate together. The validator still
requires user approval metadata for active project-origin rules and matching
installation approval for framework-origin rules. TD-0002 remains a historical
record of the original 1.0.0 layout; this decision replaces its directory and
digest-path choices.

## Acceptance scenario impact

`none`: this changes repository harness behavior, not product scenarios.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T1](../tasks/H1-T1-implement-constitution.md)
- [H1-T11](../tasks/H1-T11-current-truth.md)
