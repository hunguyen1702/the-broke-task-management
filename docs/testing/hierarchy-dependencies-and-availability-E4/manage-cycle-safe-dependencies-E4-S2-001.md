---
id: AT-E4-S2-001
story: E4-S2
status: approved
---

# Manage task dependencies

## Setup

Use a temporary repository containing three active tasks.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Add dependency | `"$TB_BIN" task dependency add "$TASK_B" --depends-on "$TASK_A" --json` | Exit `0`; output shows task B depends on task A |
| Remove dependency | Remove the dependency from B to A with `--json` | Exit `0`; output no longer contains the dependency |
| Duplicate dependency | Add the same dependency twice | The second command returns the documented conflict error and does not duplicate the relation |
| Reject cycle | Make B depend on A, then try to make A depend on B | Nonzero exit; output reports a dependency cycle and the second relation is not added |

## References

- [E4-S2 contract](../../epics/E4-S2-manage-dependencies.md)
- [Common test setup](../test-setup.md)
