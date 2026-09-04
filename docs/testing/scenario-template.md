---
id: AT-E0-S0-001
story: E0-S0
status: draft
---

# User-visible behavior

## Setup

Use an appropriate temporary fixture from [test setup](test-setup.md).

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Happy path | `"$TB_BIN" command --json` | Exit `0`; output contains the expected result |
| Invalid input | `"$TB_BIN" command invalid --json` | Nonzero exit; output contains the documented error code |

## References

- Link the approved story contract.
