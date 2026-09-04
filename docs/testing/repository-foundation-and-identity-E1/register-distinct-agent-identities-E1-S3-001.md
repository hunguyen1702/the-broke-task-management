---
id: AT-E1-S3-001
story: E1-S3
status: approved
---

# Register agent identities

## Setup

Use an initialized temporary repository.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Register an agent | `"$TB_BIN" agent register agent-a --json` | Exit `0`; output contains a generated agent ID and display name `agent-a` |
| Register another agent | `"$TB_BIN" agent register agent-b --json` | Exit `0`; the second agent has a different ID |
| Missing display name | `"$TB_BIN" agent register '' --json` | Exit `2`; error code is `INVALID_AGENT_NAME` |

## References

- [E1-S3 contract](../../epics/E1-S3-register-an-agent.md)
- [Common test setup](../test-setup.md)
