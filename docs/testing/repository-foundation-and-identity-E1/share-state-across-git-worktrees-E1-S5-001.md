---
id: AT-E1-S5-001
story: E1-S5
status: approved
---

# Share state across Git worktrees

## Setup

Use an initialized temporary Git repository with one linked worktree.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Read shared state | Create a task in the main worktree, then run `"$TB_BIN" task view "$TASK_ID" --json` in the linked worktree | Exit `0`; output shows the same task |
| Write shared state | Register an agent in the linked worktree, then run `"$TB_BIN" agent list --json` in the main worktree | Exit `0`; output includes the new agent |
| Uninitialized worktree | Run `"$TB_BIN" repo status --json` in a linked worktree whose main repository is not initialized | Nonzero exit; output reports no initialized repository |

## References

- [E1-S5 contract](../../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [Common test setup](../test-setup.md)
