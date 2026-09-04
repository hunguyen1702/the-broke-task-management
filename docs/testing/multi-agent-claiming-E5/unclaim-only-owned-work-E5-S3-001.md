---
id: AT-E5-S3-001
story: E5-S3
status: approved
---

# Unclaim owned work

## Setup

Use a temporary repository with two agents and a task claimed by agent A.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Owner unclaims task | `"$TB_BIN" task unclaim "$TASK_ID" --agent "$AGENT_A" --json` | Exit `0`; output shows the released claim and the task is unclaimed |
| Different agent | Unclaim the task as agent B | Exit `5`; error code is `CLAIM_NOT_OWNED` and agent A's claim remains |
| Task has no claim | Unclaim an unclaimed task | Exit `3`; error code is `CLAIM_NOT_FOUND` |
| Unknown task | Unclaim a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E5-S3 contract](../../epics/E5-S3-unclaim-owned-work.md)
- [Common test setup](../test-setup.md)
