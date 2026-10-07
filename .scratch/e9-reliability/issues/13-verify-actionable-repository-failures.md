# 13: Verify actionable repository failures

Status: resolved

**What to build:** Prove existing repository/configuration/database/permission failures provide actionable recovery direction and stable machine-readable results while preserving repository data. This is a slice of approved E9-S3-T1.

**Blocked by:** None (can start immediately). Existing authoritative implementation prerequisites are done; other recovery slices do not gate failure inspection.

- [ ] Reuse existing repository-status and core health proofs, extending only missing failure classification, guidance, stream, and unchanged-state coverage.
- [ ] Cover unavailable repository (exit 1), missing initialization (3), malformed/unsafe configuration and configuration/database identity mismatch (2), missing/non-SQLite/corrupt database (1), and genuine permission denial (5).
- [ ] Assert authoritative stable codes, including `PERMISSION_DENIED` for access denial, affected check/path where available, and useful recovery direction.
- [ ] JSON errors appear on stdout with empty stderr; human errors and guidance appear on stderr. Assert unchanged repository data and no creation, migration, or repair.
- [ ] Permission fixtures establish actual denied access; root-bypassed fixtures cannot pass. Record executed platform and unexecuted/unsupported checks as unavailable, distinct from passing, failing, or blocked evidence.
- [ ] Keep tests bounded, deterministic, offline, and temporary-repository based, with sufficient command diagnostics to reproduce failures.
- [ ] Record failure acceptance evidence in the E9-S3 matrix, execute authoritative focused checks, and pass format, lint, and workspace gates.
- [ ] Preserve minimal reproducible production-contract violations and require separate approved remediation plus passing reruns; do not change production behavior under this verification slice.
- [ ] Follow E9-S3 claim rules. Tickets 10–13 collectively supply recovery acceptance; only after all their evidence passes, interruption gating is reliable, and defects are resolved may the authoritative E9-S3 task/story/dashboard be completed together with final gates at the resulting repository state. This aggregation rule does not add start blockers to this ticket.

## Comments

- 2026-10-07: User confirmed the independent failure-verification slice and aggregate E9-S3 completion boundary. Parent spec remains unchanged.

## Answer

Extended `crates/tbtm-cli/tests/repo_status.rs` to prove existing repository, configuration, identity, database, and permission failures retain stable codes and exits, direct recovery guidance, correct JSON/human streams, and unchanged workspace data. The Darwin permission fixture confirms actual denied access. Focused `repo_status` suite: 9 passed; format, lint, and workspace tests passed. Windows ACL evidence is unavailable on this host. Aggregate E9-S3 remains incomplete pending tickets 10 and 11.
