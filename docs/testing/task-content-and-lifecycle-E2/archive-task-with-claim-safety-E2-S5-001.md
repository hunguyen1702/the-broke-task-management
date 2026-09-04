---
id: AT-E2-S5-001
story: E2-S5
status: approved
---

# Archive a task

## Setup

Use a temporary repository with one unclaimed task and one task claimed by a
registered agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Archive unclaimed task | `"$TB_BIN" task archive "$TASK_ID" --reason retired --json` | Exit `0`; output shows the task archived with reason `retired` |
| Archive claimed task without force | Archive the claimed task with `--json` | Exit `4`; error code is `TASK_CLAIMED`; the task remains active and claimed |
| Force archive claimed task | Archive the claimed task with `--force --yes --json` | Exit `0`; output shows the task archived and its claim released |
| Archive unknown task | Archive a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E2-S5 contract](../../epics/E2-S5-archive-a-task.md)
- [Common test setup](../test-setup.md)
