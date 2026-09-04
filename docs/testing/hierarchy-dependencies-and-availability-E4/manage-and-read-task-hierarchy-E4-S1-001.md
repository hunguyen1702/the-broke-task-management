---
id: AT-E4-S1-001
story: E4-S1
status: approved
---

# Manage task hierarchy

## Setup

Use a temporary repository containing an epic, a story, and a task.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Set parent | `"$TB_BIN" task parent set "$TASK_ID" --parent "$STORY_ID" --json` | Exit `0`; output shows the task's parent is the story |
| Read children | View the story with `--json` | Exit `0`; output includes the task as a direct child |
| Remove parent | `"$TB_BIN" task parent remove "$TASK_ID" --json` | Exit `0`; output shows no parent |
| Invalid hierarchy | Try to set a task as parent of a story | Nonzero exit; output reports an invalid hierarchy and existing relations remain unchanged |

## References

- [E4-S1 contract](../../epics/E4-S1-manage-task-hierarchy.md)
- [Common test setup](../test-setup.md)
