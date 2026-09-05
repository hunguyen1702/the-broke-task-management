# Targeted acceptance run summary

- Date: 2026-09-05
- Revision: `41da1af`
- Platform: Darwin arm64
- Scope: AT-E4-S5-001
- Result: 4 passed, 0 failed, 0 blocked
- Cleanup: the temporary Git repository was removed

## Outcome

The revalidated fixture used a valid `epic` parent, `story` root, and `task`
child. The combined map returned the root plus its parent, child, upstream
dependency, and downstream dependent with all four corresponding edge
directions. The upstream-only map omitted unrelated directions, an unrelated
task returned only its root node and no edges, and an unknown task returned
exit `3` with `TASK_NOT_FOUND`.

This rerun closes the remaining failure from the 2026-09-04 full execution and
verifies the completed [E4-S5-T2 remediation task](../../../tasks/E4-S5-T2-restore-child-traversal-in-all-maps.md).
