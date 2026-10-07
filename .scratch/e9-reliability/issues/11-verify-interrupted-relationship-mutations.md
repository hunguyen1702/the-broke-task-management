# 11: Verify interrupted relationship mutations

Status: resolved

**What to build:** Prove real CLI termination and post-write failures preserve original dependency and hierarchy state, using the validated interruption boundary from ticket 10. This is a slice of approved E9-S3-T1.

**Blocked by:** 10 — Verify interrupted claim acquisition. Its validated mutation-specific interruption gate must be available before this slice starts.

- [ ] Extend the validated real CLI harness to dependency addition and parent replacement. Each case independently establishes requested-mutation write evidence before commit; do not assume the claim gate alone proves these operations.
- [ ] Preserve original dependency rows or parent edge, affected task actor/timestamp metadata, complete claims, and unrelated state after writer termination and writable reopening of the existing database.
- [ ] Require successful integrity checks, empty foreign-key checks, and healthy read-only repository status for each recovered fixture.
- [ ] Focused public core-operation rollback tests cover dependency add/remove and parent set/replace/remove at meaningful post-write failure boundaries, including result hydration where feasible. Compare original state rather than merely returned errors; reuse validation and no-op proofs.
- [ ] Keep mutation-specific deadlines, premature-exit detection, diagnostics, and kill/reap-before-reader-release cleanup. Use no production hooks or sleep-based timing assumptions.
- [ ] Unreliable gating reopens the decision with retained evidence and blocks acceptance; production defects require separate approved remediation and passing reruns.
- [ ] Record relationship acceptance evidence in the E9-S3 matrix, execute authoritative focused checks with nonzero cases, and pass format, lint, and workspace gates.
- [ ] Follow the authoritative E9-S3 claim rules. Leave aggregate story/task completion pending all recovery slices and final verification at the resulting repository state.

## Comments

- 2026-10-07: User confirmed the dependency on ticket 10; no dependency on graph verification or backup/failure slices is introduced.

## Answer

`crates/tbtm-cli/tests/recovery_interruption.rs` now proves real CLI dependency addition and parent replacement began their requested writes before termination using mutation-specific rollback-journal markers. The existing database then preserves original relationships, task metadata, claims, and unrelated state, and passes integrity, foreign-key, and health checks. The public core `recovery_rollback` tests cover dependency add/remove and parent set/replace/remove post-write failures. Exact focused results: 3 interruption and 2 rollback tests passed; format, lint, and workspace tests passed on Darwin. E9-S3 aggregate completion still requires final integration checks and metadata synchronization.
