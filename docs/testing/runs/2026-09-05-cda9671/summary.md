# Targeted acceptance run summary

- Date: 2026-09-05
- Revision: `cda9671` with pre-existing uncommitted E6-S1 and E4-S5-T2 changes
- Platform: Darwin arm64
- Scope: AT-E1-S1-001
- Result: 5 passed, 0 failed, 0 blocked
- Cleanup: both temporary Git repositories were removed

## Outcome

Repeat initialization returned exit `2` with `ALREADY_INITIALIZED`, matching
the corrected scenario and authoritative E1-S1 contract. Initialization,
invalid-prefix rejection, uninstall preview, and confirmed uninstall also
matched their expected exit codes, output, and repository-state effects.
