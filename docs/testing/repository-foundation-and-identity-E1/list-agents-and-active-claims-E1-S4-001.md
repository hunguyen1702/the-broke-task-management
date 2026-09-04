---
id: AT-E1-S4-001
story: E1-S4
status: approved
---

# List agents and active claims

## Setup

Use a temporary repository with two agents and one task claimed by `agent-a`.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| List agents | `"$TB_BIN" agent list --json` | Exit `0`; both agents are listed and `agent-a` contains the claimed task |
| No agents | Run `"$TB_BIN" agent list --json` in a newly initialized repository | Exit `0`; output contains an empty agent list |

## References

- [E1-S4 contract](../../epics/E1-S4-list-agents-and-claims.md)
- [Common test setup](../test-setup.md)
