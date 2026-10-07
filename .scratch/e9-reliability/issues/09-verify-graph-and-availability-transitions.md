# 09: Verify graph and availability transitions

Status: resolved

**What to build:** Prove that agents receive exactly the eligible, deterministically ordered work through graph and lifecycle changes, while existing claims retain their contractual meaning. This ticket implements the approved E9-S2-T1 scope under E9-S2; it does not claim implementation by publication.

**Blocked by:** None (can start immediately). Existing authoritative implementation prerequisites are done; completion-precondition decision 08 is not a blocker.

- [ ] One real CLI journey in a temporary repository proves A-to-B-to-C claim eligibility with B depending on A and C depending on B, including successful eligible claims and blocked outcomes.
- [ ] A separate phase proves completed or archived B satisfies C's direct dependency while A remains unresolved; blockers stop at effective completion, maps continue through it, and hierarchy alone does not block availability.
- [ ] Archive/unarchive and both directions of repository-wide status completion produce exact ordered available IDs immediately after each commit. Cover multiple active tasks sharing a status and an archived task whose effective completion remains independent of that status.
- [ ] Blocking changes preserve complete downstream claim summaries, including owner and original timestamp. Archive releases its target claim; unarchive does not restore it.
- [ ] A focused ordering proof covers priority descending, creation time ascending, and task ID ascending with a deliberate equal-priority/equal-time tie. Direct SQLite fixture writes are confined to deterministic ordering setup; lifecycle operations use the CLI.
- [ ] Reuse existing cycle/no-partial-write, availability, blocker, map, lifecycle, claim/status, concurrency, and linked-worktree proofs. Do not add a new concurrency/worktree matrix or duplicate production predicates in expected values.
- [ ] Tests are bounded, deterministic, offline, and isolated; fixtures remain alive and failures report operation, expected/actual IDs, exit status, stdout, and stderr.
- [ ] Record the complete E9-S2 acceptance-to-test matrix with reused ownership and final test names; execute the authoritative focused checks and required format, lint, and workspace gates.
- [ ] Retain reproducible production-defect evidence and require separate approved remediation plus passing reruns. Do not change production behavior, commands, schema, dependencies, or hooks in this ticket.
- [ ] Claim and complete implementation through the authoritative E9-S2 task/story/dashboard rules. Synchronize completion metadata only after all acceptance evidence passes.

## Comments

- 2026-10-07: User confirmed the five-ticket breakdown and blocking edges. Parent spec remains authoritative and unchanged.

## Answer

Implemented the CLI graph journeys and deterministic task-ID tie proof in `crates/tbtm-cli/tests/graph_availability_transitions.rs` and `task_available.rs`. The complete acceptance matrix is in `docs/tasks/E9-S2-T1-verify-graph-and-availability-transitions.md`. Focused suites passed (7 new/availability and 54 reused cases); format, lint, and workspace tests passed on Darwin. E9-S2 and E9-S2-T1 are done.
