---
id: AT-E4-S3-001
story: E4-S3
status: approved
---

# Find available tasks

## Setup

Use basic tasks including one claimed, one completed, and one blocked by an
unfinished dependency.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| List available tasks | `"$TB_BIN" task available --json` | Exit `0`; active unclaimed and unblocked tasks are returned; claimed, completed, and blocked tasks are omitted |
| Filter available tasks | Run `task available` with a type or tag filter | Exit `0`; every returned task is available and matches the filter |
| No matches | Run `task available` with a filter that matches nothing | Exit `0`; output contains an empty task list |

## References

- [E4-S3 contract](../../epics/E4-S3-query-available-tasks.md)
- [Common test setup](../test-setup.md)
