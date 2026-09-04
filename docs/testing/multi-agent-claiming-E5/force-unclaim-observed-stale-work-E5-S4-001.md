---
id: AT-E5-S4-001
story: E5-S4
status: approved
---

# Force-unclaim stale work

## Setup

Use a temporary repository with a task claimed by a registered agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Confirm force unclaim | `"$TB_BIN" task unclaim "$TASK_ID" --force --yes --json` | Exit `0`; output identifies the released claim and shows the task unclaimed |
| Confirmation required | Run force unclaim with `--json` but without `--yes` | Exit `2`; error code is `CONFIRMATION_REQUIRED`; the claim remains |
| Conflicting options | Run unclaim with both `--agent` and `--force` | Exit `2`; error code is `CONFLICTING_ARGUMENTS` |
| Task has no claim | Force-unclaim an unclaimed task | Exit `3`; error code is `CLAIM_NOT_FOUND` |

## References

- [E5-S4 contract](../../epics/E5-S4-force-unclaim-stale-work.md)
- [Common test setup](../test-setup.md)
