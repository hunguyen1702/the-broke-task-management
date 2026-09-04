---
id: AT-E1-S2-001
story: E1-S2
status: approved
---

# Resolve and inspect repository state

## Setup

Use an initialized temporary Git repository and create a nested directory.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Inspect from root | `"$TB_BIN" repo status --json` | Exit `0`; output contains repository ID, prefix, root, and database path |
| Inspect from nested directory | Run `"$TB_BIN" repo status --json` from the nested directory | Exit `0`; output resolves the same repository ID and root |
| Outside a repository | Run `"$TB_BIN" repo status --json` from a fresh directory | Nonzero exit; output reports that no initialized repository was found |

## References

- [E1-S2 contract](../../epics/E1-S2-resolve-repository-configuration.md)
- [Common test setup](../test-setup.md)
