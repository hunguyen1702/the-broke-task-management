---
id: E7-S4-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E4-S3-T1
  - E4-S4-T1
  - E5-S1-T1
  - E5-S2-T1
  - E5-S3-T1
  - E5-S4-T1
  - E5-S5-T1
  - E7-S1-T1
implements:
  - E7-S4
---

# E7-S4-T1: Harden the availability and claim CLI surface

## Epic

[E7-S4: Expose availability and claim operations](../epics/E7-S4-expose-availability-and-claim-operations.md)

## Objective

Audit, complete, and regression-test the established availability, blocker, claim, claim-next, owner-unclaim, and force-unclaim CLI surface so safe work acquisition is discoverable and consistently uses the shared E7-S1 transport without changing domain behavior.

## Readiness

All direct implementation dependencies are complete. E4-S3 and E4-S4 provide authoritative availability selection and explanations; E5-S1 through E5-S5 provide claim lifecycle and independence; E7-S1 provides the shared output boundary. The current CLI already exposes the expected commands, so implementation begins as an evidence-driven audit and changes production code only for demonstrated wiring, help, or consistency gaps.

## Deliverables

- The exact E7-S4 command inventory protected by a focused command/help regression matrix.
- Help checks for read-only query intent, required agent UUIDs, reusable filters, owner versus force-unclaim paths, and explicit confirmation flags.
- A compact integration matrix covering representative human/JSON, query/mutation, success/empty/conflict/forbidden, confirmation, and main/linked-worktree paths.
- Fixes for missing wiring, inaccurate help, unsupported option exposure, leaf-local transport bypasses, or command-path ambiguity found by the audit.
- Preservation of all owning-story domain, transaction, rollback, and concurrency suites.

No migration, new syntax, payload/error redesign, domain model, acceptance-scenario change, or broad core refactor is expected.

## Proposed structure

Prefer extending the current CLI definition and integration suites:

```text
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/command_surface.rs   # optional shared E7 matrix
crates/tbtm-cli/tests/command_output.rs    # reuse E7-S1 coverage where useful
crates/tbtm-cli/tests/task_available.rs
crates/tbtm-cli/tests/task_blockers.rs
crates/tbtm-cli/tests/task_claim.rs
crates/tbtm-cli/tests/task_unclaim.rs
```

Exact test placement follows the implemented repository. Keep domain operations and typed errors in `tbtm-core`; do not add CLI-owned availability or claim rules to simplify testing.

## Technical choices

- Treat the Clap command tree as the public exposure boundary and existing E4/E5 core APIs as authoritative behavior.
- Maintain these leaves: `task available`, `task blockers`, `task claim`, `task claim-next`, and `task unclaim`, with the latter protecting both owner and force invocation forms.
- Test command exposure through generated help and real process invocations rather than private enum layout or brittle full-help snapshots.
- Assert presence and nesting of E7-S4 commands without asserting unrelated task commands are absent.
- Reuse E7-S1 global output selection and renderer; correct any audited leaf-specific mode path rather than adding a compatibility layer.
- Verify option support by capability: available and claim-next share repeatable status/type/tag filters; claim/claim-next/owner-unclaim use explicit agent identity; blocker reads do not; force-unclaim accepts no agent identity and binds `--yes` to `--force`.
- Reuse canonical repository fixtures and one representative linked-worktree acquisition flow. Leave exhaustive graph predicates, SQLite races, rollback injection, and observed-claim replacement coverage in owning suites.
- Prefer semantic help assertions so harmless Clap formatting changes do not cause noise.

## Implementation flow

### 1. Inventory current exposure

- Compare live task and leaf help with the epic inventory and trace each handler to its authoritative core operation.
- Compare task IDs, required UUIDs, repeatable filters, mutually exclusive `--agent`/`--force`, and `--yes` requirements with owning contracts.
- Check empty results and stable conflict/ownership failures through representative process invocations.
- Record demonstrated gaps in focused tests before changing production code.

### 2. Correct wiring and help gaps

- Wire any missing listed leaf to its existing core operation.
- Correct inaccurate `about`, argument help, value names, or nesting that prevents users and agents from forming a valid invocation or understanding informative versus atomic behavior.
- Keep every operation below the established `task` group and preserve current verb names.
- Do not add aliases, wrappers, duplicate handlers, or CLI-side domain validation that competes with core.

### 3. Verify actor, force, and lifecycle boundaries

- Assert no actor option on `task available` or `task blockers`.
- Assert required `--agent <uuid>` on `task claim` and `task claim-next`; owner-unclaim requires the same unless the explicit force path is selected.
- Assert `--agent` and `--force` are mutually exclusive, `--yes` without `--force` is rejected, and force execution represents only the logical user.
- Verify claim/unclaim operations do not expose or perform implicit status transitions, transfers, expiry, or identity defaults.

### 4. Verify query, atomicity, and result distinctions

- Exercise available and blocker queries as informative read-only operations without suggesting they reserve work.
- Exercise specified claim as the conflict-capable operation and claim-next as atomic select-and-claim with successful empty output.
- Exercise owner-unclaim and force-unclaim with representative ownership and observed-claim failures.
- Rely on E4/E5 core and concurrency suites for exact predicate, lock acquisition, rollback, and replacement races; the surface matrix proves the correct handlers and results are exposed.

### 5. Verify shared transport and confirmation

- Exercise representative leaves with global/repeated `--json` placement from E7-S1.
- Assert one envelope, stream discipline, and existing exits without redefining payload fields or error details.
- Verify JSON and non-terminal force-unclaim without `--yes` returns the existing confirmation-required result and never prompts; explicit `--yes` executes, while interactive human cancellation remains a no-write success.
- Preserve ordinary human help and E7-S1 help/version exceptions.

### 6. Verify canonical-store integration and consolidate regressions

- Run one linked-worktree journey that queries availability, explains a blocker, claims specified/next work, and releases ownership against the main worktree's canonical database.
- Add a table-driven command/help matrix that neighboring E7 coverage can share without coupling E7-S4 to unrelated command ownership.
- Keep process flows compact and retain focused dependency suites as authorities for payload detail, validation precedence, ordering, transactions, and concurrency.

## Output and error contract

E7-S4 introduces no payload or error shape:

- available and blocker commands retain E4 compact-list/explanation projections and empty behavior;
- specified claim and claim-next retain E5 full-task success projections, with specified conflicts and claim-next `data: null` empty success remaining distinct;
- owner-unclaim retains full-task `claim: null`; force-unclaim retains its released-claim plus authoritative availability result;
- all stable claim, availability, ownership, confirmation, and current-state error codes/details keep their owning exits;
- every command uses E7-S1 global `--json`, one `ok/data/error` envelope, stdout/stderr rules, and help/version behavior;
- audit fixes occur at the owning CLI adapter and receive no E7-S4-specific error code.

## Test plan

### Command and help matrix

- Assert task-group discovery of `available`, `blockers`, `claim`, `claim-next`, and `unclaim`.
- Check leaf help for task ID placement, required agent UUIDs, repeatable status/type/tag filters, owner/force exclusivity, and force confirmation.
- Verify undeclared aliases, unsupported actor placement, missing required agents, and invalid force/yes combinations fail through shared parse behavior without domain mutation.

### Representative behavior matrix

- Query: non-empty and empty available lists; available and multi-reason blocked explanations.
- Acquisition: specified claim success and `CLAIM_CONFLICT`; claim-next success and no-candidate empty success.
- Release: owner-unclaim success and foreign-owner rejection; force-unclaim missing confirmation, confirmed success, and changed observed claim.
- Independence: representative claim/release outcomes preserve status and task mutation metadata.
- Transport: representative success, empty, not-found, conflict, forbidden, validation, and confirmation results in human and JSON modes, reusing E7-S1 assertions.
- Worktrees: one cohesive cross-worktree query/acquire/release journey against canonical shared state.

### Regression boundary

- Keep E4-S3/E4-S4 and E5-S1–E5-S5 focused tests passing without changing predicates, filters, projections, errors, validation order, transactions, or concurrency behavior.
- Do not duplicate detailed graph combinations, timestamp rules, lock races, busy timeout, fault injection, or rollback matrices already owned by dependencies.
- If the audit finds no production gap, a test/help-only implementation is a valid result.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test every owned leaf and one full availability/claim/release lifecycle from both main and linked worktrees.

## Acceptance scenario impact

`none`. The audit found the established command wiring and public help already match
the approved contract, so implementation only adds regression coverage and does not
change user-visible behavior. No files under `docs/testing/` were changed, reviewed,
or executed.

## Definition of done

- Every E7-S4 functional and non-functional acceptance criterion passes.
- All owned operations are discoverable, correctly nested, and connected to authoritative E4/E5 behavior.
- Help accurately distinguishes informative queries, atomic acquisition, required identity, filters, owner release, and explicit force confirmation.
- Representative results inherit E7-S1 transport without leaf-specific JSON or hidden prompt behavior.
- Main and linked worktrees use one canonical repository and preserve atomic claim semantics.
- No alias, top-level claim group, implicit identity, status transition, migration, persistent field, domain operation, or E7-S4-specific payload/error is introduced.
- Existing dependency behavior and focused tests remain stable; formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E7-S4 epic](../epics/E7-S4-expose-availability-and-claim-operations.md)
- [PRD E7-S4 story](../PRD.md#story-e7-s4-expose-availability-and-claim-operations)
- [E7-S1 output contract](../epics/E7-S1-provide-consistent-command-output.md)
- [E4 availability and blocking contracts](../PRD.md#epic-e4-task-hierarchy-and-dependency-graph)
- [E5 claim lifecycle contracts](../PRD.md#epic-e5-multi-agent-claiming)
- Direct dependency epics and implementation tasks listed in frontmatter.
