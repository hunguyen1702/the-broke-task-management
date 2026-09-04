---
id: AT-E1-S1-001
story: E1-S1
status: approved
---

# Initialize and uninstall a repository

## Setup

Build `tbtm` and create a temporary Git repository.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Initialize | `"$TB_BIN" init --prefix acc --json` | Exit `0`; output identifies an initialized repository with prefix `acc` |
| Initialize twice | Run the initialize command again | Exit `4`; error code is `ALREADY_INITIALIZED` |
| Invalid prefix | `"$TB_BIN" init --prefix '' --json` in a fresh repository | Exit `2`; error code is `INVALID_PREFIX`; no repository is initialized |
| Uninstall preview | `"$TB_BIN" uninstall --dry-run --json` | Exit `0`; output lists what would be removed and repository state remains present |
| Uninstall | `"$TB_BIN" uninstall --yes --json` | Exit `0`; output confirms removal and repository state is gone |

## References

- [E1-S1 contract](../../epics/E1-S1-initialize-repository.md)
- [Common test setup](../test-setup.md)
