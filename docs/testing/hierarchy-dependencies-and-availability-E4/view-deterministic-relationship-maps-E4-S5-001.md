---
id: AT-E4-S5-001
story: E4-S5
status: approved
---

# View task relationship maps

## Setup

Use related tasks containing a small hierarchy and dependency chain.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| View all relationships | `"$TB_BIN" task map "$TASK_ID" --direction all --json` | Exit `0`; output contains the task and its related parent, child, upstream, and downstream tasks |
| View one direction | Run `task map` for only upstream relationships | Exit `0`; output contains upstream dependencies and omits unrelated directions |
| Unrelated task | Run `task map` for a task with no relationships | Exit `0`; output contains the root task and no relation edges |
| Unknown task | Run `task map` for a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E4-S5 contract](../../epics/E4-S5-view-relationship-maps.md)
- [Common test setup](../test-setup.md)
