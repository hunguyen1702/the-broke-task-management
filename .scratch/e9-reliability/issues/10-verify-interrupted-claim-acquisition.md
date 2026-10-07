# 10: Verify interrupted claim acquisition

Status: resolved

**What to build:** Establish trustworthy real CLI evidence that terminating claim acquisition after its mutation begins and before commit preserves original repository state. Include focused claim rollback evidence and recovered health checks. This is a slice of approved E9-S3-T1, not a new production feature.

**Blocked by:** None (can start immediately). Existing authoritative implementation prerequisites are done.

- [ ] Validate a hook-free harness on a fully migrated temporary fixture: establish a reader transaction that actually holds a SHARED lock, launch a real CLI claim writer, and prove journal evidence attributable to the requested mutation before commit. Journal existence or size alone is insufficient.
- [ ] Use a bounded deadline shorter than the writer busy timeout, detect premature exit, and capture command diagnostics. Always kill and reap the writer before releasing the reader, including failure cleanup; do not rely on sleeps for correctness.
- [ ] Reopen the existing database writable without creating a replacement where SQLite recovery is needed. Compare original complete claims, relevant task rows, actor/timestamp metadata, and unrelated state.
- [ ] Require SQLite integrity success, empty foreign-key checks, and healthy read-only repository status. Do not require database byte equality or claim power-loss durability or guaranteed hot-journal replay.
- [ ] Focused public core-operation tests inject meaningful post-write claim failures using test-owned SQLite fixtures and prove original rows/metadata are preserved. Cover result hydration where feasible without production hooks; an error alone is insufficient proof.
- [ ] If mutation-specific gating is unreliable, retain feasibility evidence and reopen the design decision. Leave this ticket and E9-S3 incomplete; do not substitute rollback-only evidence or add production hooks.
- [ ] Record claim acceptance evidence and reusable harness boundaries in the E9-S3 matrix. Tests are deterministic, bounded, offline, and isolated; focused tests execute nonzero cases and format, lint, and workspace gates pass.
- [ ] Preserve reproducible production-defect evidence for separate approved remediation. Follow the authoritative E9-S3 implementation claim rules; this slice cannot mark the entire story/task complete.

## Comments

- 2026-10-07: User confirmed the five-ticket breakdown. Ticket 11 depends on this ticket's validated interruption gate.
- 2026-10-07: The pre-held SHARED reader prevented the CLI's no-op migration transaction from committing before claim acquisition. Feasibility evidence is in `docs/testing/E9-S3-claim-interruption-feasibility.md`. User approved a separate remediation in ticket 14; retry this ticket after that change. The ticket remains claimed and incomplete.

## Answer

After approved remediation 14, `crates/tbtm-cli/tests/recovery_interruption.rs::killed_claim_writer_preserves_original_claims_and_repository_health` passed against the real CLI. A pre-held SHARED reader blocks commit; a claim-table marker in the rollback journal proves the requested write began. The child is killed and reaped before lock release, then the existing database passes original-state, integrity, foreign-key, and health checks. `crates/tbtm-core` also passes a post-write claim rollback test under `recovery_rollback`. Both exact focused commands, format, lint, and workspace tests passed on Darwin. Ticket 11 may reuse the validated gate.
