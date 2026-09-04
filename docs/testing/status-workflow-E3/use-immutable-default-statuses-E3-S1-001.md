---
id: AT-E3-S1-001
story: E3-S1
status: approved
---

# Use default task statuses

## Setup

Use an initialized temporary repository.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Default status | Create a task without specifying status | Exit `0`; output shows status `to_do` |
| Select a status | Create a task with status `in_progress` | Exit `0`; output shows status `in_progress` |
| Complete a task | Update the task to status `done` | Exit `0`; output shows status `done` and completed behavior |
| Unknown status | Create or update a task with an unknown status | Exit `3`; error code is `STATUS_NOT_FOUND` |

## References

- [E3-S1 contract](../../epics/E3-S1-use-default-statuses.md)
- [Common test setup](../test-setup.md)
