---
id: AT-E5-S1-001
story: E5-S1
status: approved
---

# Claim a specified task

## Setup

Use a temporary repository with two agents, an available task, and a blocked
task.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Claim available task | `"$TB_BIN" task claim "$TASK_ID" --agent "$AGENT_A" --json` | Exit `0`; output shows the task claimed by agent A |
| Claim already claimed task | Have agent B claim the same task | Exit `4`; error code is `CLAIM_CONFLICT` and agent A's claim remains |
| Claim blocked task | Try to claim the task with an unfinished dependency | Exit `2`; error code is `TASK_NOT_AVAILABLE` |
| Unknown agent | Claim an available task with a nonexistent agent ID | Exit `3`; error code is `AGENT_NOT_FOUND` |

## References

- [E5-S1 contract](../../epics/E5-S1-claim-a-specified-task-atomically.md)
- [Common test setup](../test-setup.md)
