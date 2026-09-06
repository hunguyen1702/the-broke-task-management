# Acceptance run summary

- Date: 2026-09-07
- Revision: `04734ff`
- Platform: Darwin 25.6.0 arm64
- Scope: all 23 approved scenarios in the acceptance regression catalog
- Result: 88 passed, 0 failed, 0 blocked
- Cleanup: all temporary repositories and linked worktrees were removed

## Outcome

Every approved case matched its expected process exit code and user-visible
output. The run covered repository initialization and resolution, agent identity,
task content and lifecycle, statuses, hierarchy and dependencies, availability,
claim lifecycle, relationship maps, and approved comment add/list behavior.

The draft AT-E6-S2-001 scenario and the queued E7-S5 acceptance addition were not
executed because only approved scenarios are eligible for execution.

## Verification baseline

The CLI was built before execution with `rtk cargo build -p tbtm`. Every scenario
used fresh isolated temporary repositories and the binary at revision `04734ff`.
