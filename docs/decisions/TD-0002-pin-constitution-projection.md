---
id: TD-0002
status: accepted
date: 2026-09-21
related_epics: [H1]
related_stories: []
supersedes: []
scenario_impact: none
affected_scenarios: []
---

# Pin Constitution inputs and derive one query index

## Context

H1-T1 needs readable rule files, a reproducible local lookup, and detection of
framework edits without adding product runtime behavior.

## Decision

Version the manifest and rule schema independently. Pin the framework rule
tree with SHA-256 over sorted relative path, NUL, raw file bytes, NUL tuples.
Use a deterministic Ruby validator with the system YAML library. Treat rule
files as canonical and replace the sole index atomically after validating all
canonical inputs. Keep project rules in a separate tree with exact revision
references. No product crate or SQLite schema consumes these files.

## Observable behavior

The repository harness gains `validate`, `rebuild-index`, and
`inspect-effective`. `tbtm` product behavior and CLI output do not change.

## Consequences

Framework drift and stale projections fail validation. A framework upgrade
requires a separately governed change to its pinned digest and ruleset.
Approval metadata is structurally checked but cannot prove user intent.

## Acceptance scenario impact

`none`: harness-only behavior is covered by isolated validator tests.

## References

- [H1](../epics/H1-build-adaptive-repository-harness.md)
- [H1-T1](../tasks/H1-T1-implement-constitution.md)
