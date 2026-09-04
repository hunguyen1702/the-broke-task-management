---
id: AT-E4-S4-001
story: E4-S4
status: approved
---

# Explain why a task is blocked

## Setup

Use related tasks where task C depends on unfinished task B, and task B depends
on unfinished task A.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Explain dependency blockers | `"$TB_BIN" task blockers "$TASK_C" --json` | Exit `0`; output identifies task B as direct and task A as a transitive unresolved dependency |
| Available task | Run blockers for a task with no blocking condition | Exit `0`; output says the task is available and lists no blockers |
| Unknown task | Run blockers for a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E4-S4 contract](../../epics/E4-S4-explain-blocking.md)
- [Common test setup](../test-setup.md)
