---
id: AT-E2-S2-001
story: E2-S2
status: approved
---

# View and filter tasks

## Setup

Use a temporary repository with active tasks of different types, statuses,
priorities, and tags, plus one archived task.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| View a task | `"$TB_BIN" task view "$TASK_ID" --json` | Exit `0`; output contains the selected task and its details |
| List active tasks | `"$TB_BIN" task list --json` | Exit `0`; active tasks are listed and the archived task is omitted |
| Filter tasks | Run `task list` with a type, status, or tag filter | Exit `0`; every returned task matches the selected filter |
| Include archived tasks | `"$TB_BIN" task list --all --json` | Exit `0`; active and archived tasks are listed |
| Unknown task | `"$TB_BIN" task view acc-task-00000000 --json` | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E2-S2 contract](../../epics/E2-S2-view-and-list-tasks.md)
- [Common test setup](../test-setup.md)
