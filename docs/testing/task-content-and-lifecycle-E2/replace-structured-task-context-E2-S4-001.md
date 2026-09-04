---
id: AT-E2-S4-001
story: E2-S4
status: approved
---

# Replace structured task context

## Setup

Use a temporary repository with a task containing tags, URLs, and code
references.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Replace context | Update the task with new tags, URLs, and code references using `--json` | Exit `0`; output contains the new collections and not the old ones |
| Clear context | Update the task to clear its structured context | Exit `0`; output contains empty context collections |
| Duplicate value | Update with the same tag twice | Exit `2`; error code is `DUPLICATE_TASK_CONTEXT`; existing context remains unchanged |
| Invalid URL | Update with an invalid URL | Exit `2`; error code is `INVALID_EXTERNAL_URL` |

## References

- [E2-S4 contract](../../epics/E2-S4-manage-structured-task-context.md)
- [Common test setup](../test-setup.md)
