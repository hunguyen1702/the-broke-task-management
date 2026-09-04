---
id: AT-E5-S2-001
story: E5-S2
status: approved
---

# Claim the next available task

## Setup

Use a temporary repository with one agent and several available tasks with
different priorities, plus a claimed and a blocked task.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Claim next task | `"$TB_BIN" task claim-next --agent "$AGENT_ID" --json` | Exit `0`; output shows the highest-ranked available task claimed by the agent |
| Apply a filter | Run `task claim-next` with a matching type or tag filter | Exit `0`; the selected task matches the filter |
| No available task | Run `task claim-next --agent "$AGENT_ID" --json` when all matching tasks are unavailable | Exit `0`; output data is `null` |
| Unknown agent | Run `task claim-next` with a nonexistent agent ID | Exit `3`; error code is `AGENT_NOT_FOUND` |

## References

- [E5-S2 contract](../../epics/E5-S2-claim-the-next-available-task-atomically.md)
- [Common test setup](../test-setup.md)
