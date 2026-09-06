---
id: E7-S3-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1-T1
  - E3-S2-T1
  - E3-S3-T1
  - E3-S4-T1
  - E4-S1-T1
  - E4-S2-T1
  - E4-S3-T1
  - E4-S4-T1
  - E4-S5-T1
  - E4-S5-T2
  - E7-S1-T1
implements:
  - E7-S3
---

# E7-S3-T1: Harden the planning relationship and status CLI surface

## Epic

[E7-S3: Expose planning relationships and status operations](../epics/E7-S3-expose-planning-relationships-and-status-operations.md)

## Objective

Audit, complete, and regression-test the established status, hierarchy,
dependency, and relationship-map CLI surface so every owned core capability is
discoverable and uses the shared E7-S1 transport without changing domain
behavior.

## Readiness

All direct implementation dependencies are complete. E3 provides authoritative
status operations; E4 provides hierarchy, dependency, readiness, blocking, and
map behavior; E4-S5-T2 preserves child traversal in combined maps; E7-S1
provides the shared output boundary. The current CLI already exposes the owned
commands, so implementation begins as an evidence-driven audit and adds
production changes only for demonstrated wiring, help, or consistency gaps.

## Deliverables

- The E7-S3 command inventory protected by a focused help and process-level
  regression matrix.
- Root/group/leaf discoverability checks for status management, parent and
  dependency mutations, hierarchy queries, and five-direction maps.
- Actor-boundary checks for logical-user status operations, attributed
  relationship mutations, and actor-free relationship reads.
- Confirmation checks for `status set-completed`, including `--yes`, JSON,
  non-terminal, no-op, and cancellation boundaries already supported by its
  owner.
- A compact canonical-store journey across main and linked worktrees.
- Minimal fixes for demonstrated missing wiring, inaccurate help,
  leaf-specific transport bypasses, or actor/confirmation drift.

No migration, new command syntax, payload redesign, domain operation,
acceptance-scenario change, or broad production refactor is expected.

## Proposed structure

Prefer extending the established CLI and command-surface integration suite:

```text
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/command_surface.rs
crates/tbtm-cli/tests/command_output.rs
```

A focused `planning_command_surface.rs` test may be introduced if it keeps the
E7-S3 matrix clearer than extending the E7-S2 suite. Keep core operations and
typed errors in `tbtm-core`; do not create CLI-owned substitutes for testing.

## Technical choices

- Treat the Clap command tree as the public exposure boundary and the existing
  E3/E4 core APIs as authoritative.
- Maintain exactly the inventory in the epic. Do not include `task available`
  or `task blockers`; E7-S4 owns their command-surface completeness.
- Test generated help and real process invocations rather than private command
  enum layout or brittle full-help snapshots.
- Assert semantic command/option presence without requiring unrelated E7
  families to be absent from root/task help.
- Reuse E7-S1's global JSON selection and renderer; correct any audited
  leaf-local transport path rather than adding a compatibility layer.
- Verify authority by capability: no status command accepts `--agent`; parent
  and dependency mutations accept it; hierarchy and map queries do not.
- Test `status set-completed` as the sole owned confirmation workflow. Preserve
  its authoritative impact calculation and prompt implementation rather than
  recreating either inside the surface suite.
- Use one representative linked-worktree flow; detailed discovery,
  transaction, race, cycle, traversal, and snapshot behavior remains covered
  by owning stories.

## Implementation flow

### 1. Inventory current exposure

- Compare live root/group/leaf help with the epic inventory.
- Trace every leaf handler to its existing core operation and identify missing,
  duplicated, disconnected, or CLI-reimplemented domain behavior.
- Compare positional roles, required arguments, placement flags, recursive and
  direction options, actor flags, completion values, and confirmation options
  with owning contracts.
- Capture demonstrated gaps with focused tests before changing production code.

### 2. Correct command wiring and help gaps

- Wire a missing owned leaf to its authoritative core operation if the audit
  finds one.
- Correct inaccurate `about`, argument help, value names, or nesting that stops
  a user or agent forming a documented invocation.
- Preserve the top-level `status` group and the existing `task parent`, `task
  dependency`, `task hierarchy`, and `task map` paths.
- Add no aliases, wrapper workflows, duplicate relationship lists, or
  replacement handlers.

### 3. Verify actor and authority boundaries

- Assert every status command rejects `--agent` and retains logical-user
  authority.
- Assert parent/dependency set/add/remove mutations accept optional registered
  agent UUIDs and preserve logical-user default behavior.
- Assert hierarchy and map reads reject actor selection and perform no task
  attribution.
- Reuse owning-story malformed/unknown-agent errors and metadata assertions;
  add no E7-S3-specific actor state or error.

### 4. Verify confirmations and shared transport

- Exercise `status set-completed` with non-empty impact using `--yes`, missing
  authorization in JSON/non-terminal mode, and existing human cancellation
  coverage where practical.
- Exercise its same-value no-op without confirmation and prove other owned
  commands never gain prompt/`--yes` behavior.
- Run representative leaves with global and repeated `--json` placement.
- Assert shared envelope/stream/exit behavior without redefining domain payload
  fields or error details.

### 5. Verify relationship queries and map directions

- Build a small mixed hierarchy/dependency fixture through public mutations.
- Inspect direct relationships through `task view`, direct/recursive hierarchy
  through `task hierarchy`, and recursive maps through `parent`, `child`,
  `upstream`, `downstream`, default `all`, and explicit `all`.
- Use semantic assertions for root, direction, nodes, and edge kinds; leave
  exhaustive ordering, badge, diamond, cycle, and renderer cases in E4-S5.
- Protect E4-S5-T2 by including at least one child edge in a representative
  combined map.

### 6. Verify canonical-store integration and consolidate regressions

- Run one cohesive journey across a main worktree and linked worktree: create a
  custom status, build parent/dependency relationships, inspect hierarchy/map,
  and observe mutations from both roots.
- Confirm all leaves resolve the main worktree's canonical database and create
  no linked-worktree `.tbtm` store.
- Keep the matrix easy to extend, with diagnostics identifying the missing
  command, option, authority boundary, transport behavior, or worktree result.

## Output and error contract

E7-S3 introduces no payload or error shape:

- status commands retain E3 status, collection, deletion, completion-impact,
  no-op, and cancellation results;
- relationship mutations retain E4 edge/parent results and task metadata
  behavior;
- hierarchy, task detail, and maps retain their deterministic read models and
  human renderers;
- all commands use E7-S1's global `--json`, one `ok/data/error` envelope,
  stdout/stderr discipline, parse errors, and exits `0` through `5`;
- audited gaps are fixed in their owning CLI adapter and receive no E7-S3
  payload field, error code, or compatibility alias.

## Test plan

### Command and help matrix

- Assert discovery of top-level `status`, all six status leaves, `task parent`
  set/remove, `task dependency` add/remove, `task hierarchy`, and `task map`.
- Check representative help for status placement, completed boolean/`--yes`,
  relationship endpoint roles, optional actor selectors, hierarchy recursion,
  map default direction, and all five allowed direction values.
- Verify undeclared aliases and unsupported actor/confirmation options on
  leaves that do not own them fail through normal parse behavior without domain
  mutation.

### Representative behavior matrix

- Status: ordered list, custom create/rename/move/delete, same-value no-op, and
  one confirmed completion change with impact.
- Relationships: set/replace/remove parent, add/remove dependency, direct detail
  hydration, direct/recursive hierarchy, and mixed relationship maps.
- Actor: logical-user default, registered-agent relationship mutations, and
  rejection on status/hierarchy/map commands.
- Transport: representative success, no-op, validation, not-found, conflict,
  forbidden, and confirmation-required paths in human/JSON modes by reusing
  E7-S1 assertions.
- Worktrees: one cohesive main/linked journey against the canonical store.

### Regression boundary

- Keep focused E3/E4/E7-S1 tests passing without changing payloads, stable
  errors, ordering, mutation metadata, transactions, concurrency, traversal,
  snapshots, or persistence.
- Do not duplicate detailed status migration/order races, graph cycles,
  availability/blocker cases, map diamonds/badges, or large-graph performance
  tests already owned by dependencies.
- If the audit finds no production gap, a test/help-only implementation is a
  valid result.
- Do not create, edit, review, approve, or execute acceptance scenarios during
  this implementation task. Acceptance impact is classified only after actual
  implementation changes are known.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test the owned inventory and one status/relationship journey across
the main worktree and a linked worktree.

## Definition of done

- Every E7-S3 functional and non-functional acceptance criterion passes.
- All owned status and planning-relationship operations are discoverable and
  connected to authoritative E3/E4 behavior.
- Help accurately exposes syntax, endpoint roles, actor support, confirmation,
  hierarchy recursion, and every map direction.
- Representative commands inherit E7-S1 transport without leaf-specific JSON
  or prompt behavior.
- Main and linked worktrees use the same canonical store.
- `task available`, `task blockers`, and claim workflows remain for E7-S4.
- No alias, aggregate workflow, implicit actor state, migration, persistent
  field, domain operation, or E7-S3-specific payload/error is introduced.
- Existing E3/E4 behavior and focused tests remain stable; formatting, Clippy
  with warnings denied, and all workspace tests pass.

## References

- [E7-S3 epic](../epics/E7-S3-expose-planning-relationships-and-status-operations.md)
- [PRD E7-S3 story](../PRD.md#story-e7-s3-expose-planning-relationships-and-status-operations)
- [E7-S1 output contract](../epics/E7-S1-provide-consistent-command-output.md)
- [E3 status workflow contracts](../PRD.md#epic-e3-status-workflow)
- [E4 hierarchy, dependency, and map contracts](../PRD.md#epic-e4-hierarchy-dependencies-and-availability)
- Direct dependency epics and implementation tasks listed in frontmatter.
