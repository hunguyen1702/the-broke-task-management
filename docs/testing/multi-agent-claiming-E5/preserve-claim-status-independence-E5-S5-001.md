---
id: AT-E5-S5-001
story: E5-S5
status: approved
---

# Preserve claims when status changes

## Setup

Use a temporary repository with a task claimed by agent A.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Complete claimed task | Update the claimed task to status `done` | Exit `0`; output shows status `done` and retains agent A's claim |
| Reopen claimed task | Update the completed claimed task to status `in_progress` | Exit `0`; output shows the new status and retains the claim |
| Unclaim completed task | Unclaim the completed task as agent A | Exit `0`; output removes the claim without changing status `done` |
| Archive claimed task | Force archive the claimed task | Exit `0`; output shows the task archived and its claim released |

## References

- [E5-S5 contract](../../epics/E5-S5-preserve-claim-status-independence.md)
- [Common test setup](../test-setup.md)
