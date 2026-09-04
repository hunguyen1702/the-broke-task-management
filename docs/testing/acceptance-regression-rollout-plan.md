# Acceptance regression rollout plan

## Goal

Maintain a small QA-oriented regression catalog for the main ways a person uses
the public `tbtm` CLI.

## Scope

- Cover each implemented story's common successful workflow.
- Cover likely user mistakes and important rejected operations.
- Describe cases as input plus expected user-visible output.
- Use temporary repositories and public CLI commands.
- Keep concurrency, transaction internals, fault injection, database
  corruption, rare collisions, and exhaustive boundaries in automated Rust
  tests rather than acceptance scenarios.

## Completion gates

- Every implemented story has a scenario or an explicit reason for no distinct
  user-visible scenario.
- Happy paths and meaningful bad paths are represented.
- Each case has a clear input, expected exit, and expected output.
- A QA reviewer confirms that common user workflows are covered and do not
  contradict approved behavior.
- The user approves the catalog before execution.

## Lifecycle

Scenarios use `draft`, `in_review`, `approved`, `outdated`, or `superseded`.
When behavior changes, update an unapproved scenario or supersede an approved
one when preserving its previous meaning is useful.
