---
id: AT-E2-S1-001
story: E2-S1
status: approved
---

# Create a task

## Setup

Use an initialized temporary repository with one registered agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Create a basic task | `"$TB_BIN" task create --title 'Task A' --type task --json` | Exit `0`; output contains a task ID, title `Task A`, type `task`, and status `to_do` |
| Create with common details | Create a task with description, priority, estimate, tag, URL, and registered agent | Exit `0`; output contains the supplied values |
| Empty title | `"$TB_BIN" task create --title '' --type task --json` | Exit `2`; error code is `INVALID_TASK_TITLE` |
| Unknown agent | Create a task with a nonexistent agent ID | Exit `3`; error code is `AGENT_NOT_FOUND` |

## References

- [E2-S1 contract](../../epics/E2-S1-create-a-task.md)
- [Common test setup](../test-setup.md)
