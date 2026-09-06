---
id: TD-0001
status: accepted
date: 2026-09-06
related_epics:
  - E7-S1
related_stories:
  - E7-S1
supersedes: []
scenario_impact: add
affected_scenarios:
  - E7-S1
---

# Centralize CLI output-mode and parse-error handling

## Context

Each leaf command previously owned a separate `--json` flag, and Clap terminated on parse failures before the selected output mode could be honored. Partial uninstall also emitted a success document before reporting its failure.

## Decision

Define one inherited, idempotent global `--json` option. Parse argv fallibly, use an exact `--json` token to select machine-readable parse failures, and retain complete partial-uninstall results in a typed error rendered by the common envelope boundary.

## Observable behavior

Public output behavior now follows the approved [E7-S1 contract](../epics/E7-S1-provide-consistent-command-output.md): JSON placement is global, recognized parse errors use stdout-only envelopes, help/version remain ordinary Clap output, and partial uninstall emits one error document.

## Consequences

Commands no longer duplicate output-mode fields or traversal logic. Existing successful domain payloads and persistence remain unchanged. CLI callers may place or repeat `--json` throughout the command path and can parse failures from the same response stream as domain errors.

## Acceptance scenario impact

`add` for E7-S1. A separate acceptance-scenario workflow should add coverage for the new global output boundary; this implementation does not edit the scenario catalog.

## References

- [E7-S1 epic](../epics/E7-S1-provide-consistent-command-output.md)
- [E7-S1-T1 task](../tasks/E7-S1-T1-standardize-cli-output-boundary.md)
- [PRD CLI requirements](../PRD.md#fr-11-cli-experience)
