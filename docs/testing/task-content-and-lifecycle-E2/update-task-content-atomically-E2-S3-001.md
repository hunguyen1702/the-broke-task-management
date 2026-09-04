---
id: AT-E2-S3-001
story: E2-S3
status: approved
---

# Update task fields

## Setup

Use a temporary repository with one task and one registered agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Update common fields | Update the task title, description, priority, estimate, and status with `--json` | Exit `0`; output contains all updated values |
| Clear optional fields | Update the task to clear its description and estimate | Exit `0`; output shows those fields cleared |
| Invalid value | Update the task with priority `1000001` | Exit `2`; error code is `INVALID_TASK_PRIORITY` and the task keeps its previous values |
| Unknown task | Update a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E2-S3 contract](../../epics/E2-S3-update-task-content.md)
- [Common test setup](../test-setup.md)
