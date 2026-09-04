---
id: AT-E2-S6-001
story: E2-S6
status: approved
---

# Unarchive a task

## Setup

Use a temporary repository with an archived task and another active task that
depends on it.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Confirmation required | `"$TB_BIN" task unarchive "$TASK_ID" --json` | Exit `2`; error code is `CONFIRMATION_REQUIRED` and output reports the affected dependent task |
| Unarchive | `"$TB_BIN" task unarchive "$TASK_ID" --yes --json` | Exit `0`; output shows the task active and reports its direct impact |
| Unarchive active task | Run unarchive on an active task | Exit `0`; output returns the unchanged active task |
| Unknown task | Run unarchive on a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E2-S6 contract](../../epics/E2-S6-unarchive-a-task-safely.md)
- [Common test setup](../test-setup.md)
