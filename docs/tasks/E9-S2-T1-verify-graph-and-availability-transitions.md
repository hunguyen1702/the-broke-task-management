---
id: E9-S2-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E2-S3-T1
  - E2-S5-T1
  - E2-S6-T1
  - E3-S3-T1
  - E4-S1-T1
  - E4-S2-T1
  - E4-S3-T1
  - E4-S4-T1
  - E4-S5-T1
  - E4-S5-T2
  - E5-S1-T1
  - E5-S5-T1
---

# E9-S2-T1: Verify graph and availability transitions

Approved on 2026-10-07 after user confirmation of the shared understanding.

## Objective

Close the agreed E9-S2 proof gaps with one process-level lifecycle journey and a focused ordering test, reusing the existing suites.

## Proof ownership and acceptance matrix

Paths below are relative to `crates/tbtm-cli/tests/`.

| Proof | Existing evidence | Addition |
|---|---|---|
| Hierarchy cycles and preserved state | `task_hierarchy.rs::invalid_types_self_and_cycles_have_stable_errors` | Reuse |
| Dependency cycles and preserved state | `task_dependency.rs::invalid_edges_return_stable_errors_without_metadata_writes`, `competing_opposite_edges_cannot_form_a_cycle` | Reuse |
| Availability predicate | `task_available.rs::available_applies_every_predicate_and_effective_upstream_completion` | Cross-operation journey |
| Multi-level stop/traversal distinction | `task_blockers.rs::unresolved_graph_is_deduplicated_sorted_and_stops_at_completed_nodes`, `direct_membership_wins_over_recursive_and_archived_edges_stop_traversal`; `task_map.rs::traversal_survives_completed_archived_nodes_and_corrupt_cycles` | Compare available IDs, blocker context, and map node state in the journey |
| Archive/unarchive and exact claims | `task_archive.rs::force_never_releases_a_replacement_claim_and_archive_unblocks_downstream`; `task_unarchive.rs::unarchive_requires_confirmation_and_returns_ordered_direct_impact`; `claim_status_independence.rs` | Exact post-transition available IDs and full downstream claim equality |
| Repository status-completion changes | `status.rs::set_completed_reports_exact_impact_requires_confirmation_and_changes_semantics` | Both directions, multiple tasks sharing a status, and claimed downstream preservation |
| Ordering | `task_available.rs::available_filters_orders_and_does_not_duplicate_rows` | Deliberate equal-priority/equal-time ID tie |
| Canonical-store consistency | Existing linked-worktree journeys in `availability_claim_command_surface.rs`, `planning_command_surface.rs`, `task_unarchive.rs`, `task_blockers.rs`, and `task_map.rs` | Reuse; no new worktree matrix |

## Implementation evidence (Darwin, 2026-10-07)

| Stories | Passing test ownership |
|---|---|
| 1–2: hierarchy and dependency cycles without partial writes | `task_hierarchy::invalid_types_self_and_cycles_have_stable_errors`; `task_dependency::invalid_edges_return_stable_errors_without_metadata_writes`, `competing_opposite_edges_cannot_form_a_cycle` |
| 3–6: availability, chain claims, status-update semantics, completed direct upstream | `graph_availability_transitions::chain_claims_follow_direct_effective_completion_and_hierarchy_is_independent`; `task_available::available_applies_every_predicate_and_effective_upstream_completion` |
| 7–10: archived upstream, blocker stop, map traversal, containment independence | `graph_availability_transitions::chain_claims_follow_direct_effective_completion_and_hierarchy_is_independent`, `archive_and_shared_status_transitions_preserve_existing_claims`; `task_blockers::unresolved_graph_is_deduplicated_sorted_and_stops_at_completed_nodes`, `direct_membership_wins_over_recursive_and_archived_edges_stop_traversal`; `task_map::traversal_survives_completed_archived_nodes_and_corrupt_cycles` |
| 11–15: archive/unarchive, shared status completion in both directions, archive completion, exact claim preservation and release | `graph_availability_transitions::archive_and_shared_status_transitions_preserve_existing_claims`; `task_archive`, `task_unarchive`, `status`, `claim_status_independence` suites |
| 16: priority, creation time, task ID order | `task_available::available_filters_orders_and_does_not_duplicate_rows`, `available_breaks_equal_priority_and_creation_time_ties_by_task_id` |

Focused results at final integration state: `graph_availability_transitions` and `task_available`: 7 passed; ten reused CLI suites: 54 passed. `rtk mise run format`, `rtk mise run lint`, and `rtk mise run test` passed on Darwin.

## Implementation approach

1. Add `graph_availability_transitions.rs` using `CARGO_BIN_EXE_tbtm`, existing command/JSON fixture patterns, and a temporary repository. Keep helpers test-only; do not create a production verification API.
2. Prepare A, B, C with B depending on A and C depending on B, plus containment-only work and a second task referencing the tested custom status. Isolate journey branches with supported exact filters or fresh fixture state.
3. Prove normal eligibility: while all are incomplete, only A from the chain is available; after A is effectively completed, B qualifies; after B is effectively completed, C qualifies. Exercise actual claim acquisition for eligible chain tasks and existing unavailable outcomes for blocked tasks, accounting explicitly for claims in expected sets.
4. In a separate phase, keep A unresolved and mark B completed through the CLI. Assert C's edge is satisfied, blocker traversal stops at B, and the map still includes A beyond B. Include containment-only relationships and prove they do not introduce blockers.
5. Exercise archive and unarchive of incomplete upstream work, then status completion false-to-true and true-to-false across multiple active tasks referencing one status. Assert exact expected ordered available IDs after every commit. Include an archived task referencing that status to prove archive continues to satisfy dependencies.
6. Claim a downstream task while eligible, then introduce an unresolved upstream through unarchive and status-completion changes. Compare the entire claim summary before/after, including claimant identity and `claimedAt`. Verify archived targets lose claims and restored targets do not regain them.
7. Extend the existing availability ordering test with a deliberate equal-priority/equal-creation-time tie. Use test-owned SQLite fixture setup for otherwise uncontrollable timestamps; invoke the CLI for the query. Do not depend on clocks, sleeps, randomness, or incidental insertion order for expected results.
8. Record the final test names in the matrix and report failures with operation, expected/actual IDs, exit status, stdout, and stderr. Reuse existing assertions rather than duplicating implementation predicates in expected-value calculations.

Public operations in the lifecycle journey must use the CLI; direct database writes are confined to deterministic ordering fixture setup. Core continues to own rules/storage and CLI continues to own parsing/output.

## Defect boundary

If a test exposes a violation of an existing contract, retain a minimal failing regression or captured evidence and create a separate remediation task tied to that contract. Do not change production code under this verification task. Record the blocker and require passing verification after remediation before marking E9-S2 complete.

## Verification commands

Run focused proofs before the required workspace gates:

```bash
rtk cargo test -p tbtm --test graph_availability_transitions --test task_available
rtk cargo test -p tbtm --test task_hierarchy --test task_dependency --test task_blockers --test task_map --test task_archive --test task_unarchive --test status --test claim_status_independence --test availability_claim_command_surface --test planning_command_surface
rtk mise run format
rtk mise run lint
rtk mise run test
```

The first command becomes available when the planned integration target exists. The existing full workspace gate also exercises reusable core proofs. No stress repetitions or new concurrency harness are required.

## Definition of done

- The parent story's acceptance criteria have explicit passing evidence in the final matrix.
- The focused journey and final ID tie-break proof pass, with reused suites passing at the implementation repository state.
- Any production defect has separate approved remediation and passing rerun evidence; unresolved defects block completion.
- Production code, schema, dependencies, commands, and current contracts remain unchanged by this task.
- Formatting, Clippy with warnings denied, and workspace tests pass.
- Authoritative task/story metadata and `docs/STATUS.md` are updated together after implementation.

## Acceptance impact

`none`: internal automated verification only; no public behavior or acceptance-scenario catalog change.

## Dependencies and approval

All listed implementation prerequisites are currently done. E4-S5-T2 supplies the corrected combined-map traversal; E2-S3-T1 and E5-S1/S5 supply the existing status-update and claim operations used in the journey. E9-S1 is reused background evidence, not a new implementation dependency. Decision ticket 08 is not a dependency.

User confirmed the shared understanding on 2026-10-07. Implementation completed on 2026-10-07 with the evidence above and passing required gates.
