---
id: E7-S2-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E1-S1-T1
  - E1-S1-T2
  - E1-S2-T1
  - E1-S3-T1
  - E1-S4-T1
  - E1-S5-T1
  - E2-S1-T1
  - E2-S2-T1
  - E2-S3-T1
  - E2-S4-T1
  - E2-S5-T1
  - E2-S6-T1
  - E7-S1-T1
implements:
  - E7-S2
---

# E7-S2-T1: Harden the repository, agent, and task CLI surface

## Epic

[E7-S2: Expose repository, agent, and task operations](../epics/E7-S2-expose-repository-agent-and-task-operations.md)

## Objective

Audit, complete, and regression-test the established repository, agent, and task-content lifecycle CLI surface so every owned core capability is discoverable and uses the shared E7-S1 transport without changing domain behavior.

## Readiness

All direct implementation dependencies are complete. E1 and E2 provide the authoritative operations and E7-S1 provides the shared output boundary. The current CLI already exposes the expected commands, so implementation should begin as an evidence-driven audit and add production changes only for demonstrated wiring, help, or consistency gaps.

## Deliverables

- The command inventory in the E7-S2 epic, protected by a focused regression matrix covering repository lifecycle/inspection, agent registration/listing, and task create/view/list/update/archive/unarchive.
- Root, group, and representative leaf help checks for discoverability, stable nesting, actor placement, query scopes, and lifecycle confirmation options.
- A compact integration matrix covering representative human/JSON, query/mutation, confirmation, logical-user/agent, and main/linked-worktree paths.
- Fixes for missing wiring, inaccurate help, leaf-local transport bypasses, or actor-option drift found by the audit.
- Preservation of all owning-story domain behavior and focused regression suites.

No migration, new command syntax, payload redesign, domain model, acceptance-scenario change, or broad production refactor is expected.

## Proposed structure

Prefer extending the existing CLI definition and integration suites:

```text
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/command_surface.rs   # optional focused matrix
crates/tbtm-cli/tests/command_output.rs    # reuse E7-S1 coverage where suitable
```

Exact test placement follows the implemented repository. Keep domain operations and typed errors in `tbtm-core`; do not create CLI-owned substitutes merely to simplify tests.

## Technical choices

- Treat the Clap command tree as the public exposure boundary and the E1/E2 core APIs as authoritative implementations.
- Maintain this exact owned inventory: root `init`/`uninstall`; `repo status`; `agent register`/`list`; and `task create`/`view`/`list`/`update`/`archive`/`unarchive`.
- Test the command tree through generated help and real process invocations instead of snapshotting private enum layouts.
- Assert presence and nesting of owned commands without asserting that unrelated E7-S3–E7-S5 commands are absent from the overall CLI.
- Reuse the global E7-S1 output selection and renderer. Remove or correct any audited leaf-specific JSON flag/render path rather than adding another compatibility layer.
- Verify actor support by capability: task create/update/archive accept `--agent`; task unarchive and non-attributed/query operations do not.
- Reuse canonical repository fixtures and one representative linked-worktree flow. Detailed resolver and concurrency behavior remains covered by E1-S5 and owning operations.
- Prefer semantic assertions over full help snapshots where version-dependent Clap formatting would create noise.

## Implementation flow

### 1. Inventory current exposure

- Compare the live root/group/leaf command tree with the epic inventory.
- Trace every leaf handler to its existing core operation and identify missing, duplicated, or CLI-reimplemented domain behavior.
- Compare argument names, required values, repeatable options, filters, actor flags, force/yes/dry-run options, and archive scopes with the owning contracts.
- Record gaps in focused tests before changing production code.

### 2. Correct command wiring and help gaps

- Wire a missing listed leaf to its authoritative core operation if the audit finds one.
- Correct inaccurate or incomplete `about`, argument help, value names, or nesting that prevents a user from forming the documented invocation.
- Preserve root-level placement of `init`/`uninstall` and existing `repo`, `agent`, and `task` groups.
- Do not add aliases, wrapper workflows, or duplicate handlers.

### 3. Verify actor and authority boundaries

- Assert `--agent <uuid>` on task create, update, and archive and verify omitted actor remains logical user.
- Assert repository lifecycle/query, agent register/list, task view/list, and task unarchive do not accept an unsupported agent selector.
- Verify unarchive remains user-only and archive retains its existing force/confirmation authority rules.
- Reuse owning-story errors for malformed/unknown agent values; add no E7-S2-specific actor code.

### 4. Verify shared transport and invocation modes

- Exercise representative leaves with global/repeated `--json` placement supplied by E7-S1.
- Assert one success/error envelope, stream discipline, and existing exits without redefining payload fields.
- Exercise destructive/impacting lifecycle commands in JSON and non-terminal mode to prove they never prompt and continue to require their existing explicit confirmation flags.
- Preserve ordinary human help and E7-S1 help/version exceptions.

### 5. Verify canonical-store integration

- Run a representative chain from a linked worktree: inspect repository, register/list an agent, create/view/update/list a task, then archive/unarchive it as permitted.
- Confirm all commands observe the main worktree's canonical store and no linked-worktree `.tbtm` is created.
- Keep exhaustive path, failure, transaction, and race coverage in the owning E1/E2 suites.

### 6. Consolidate regressions

- Add a table-driven command/help matrix that is easy to extend when later E7 stories add their owned families.
- Keep representative process-level end-to-end flows compact; rely on existing focused tests for validation boundaries, payload detail, ordering, idempotency, and concurrency.
- Make the suite fail clearly for a missing command, wrong nesting, actor drift, transport bypass, unexpected prompt, or wrong canonical-store resolution.

## Output and error contract

E7-S2 introduces no new payload or error shape:

- each successful command retains the data and human projection defined by its E1/E2 owner;
- every command uses E7-S1's global `--json`, single `ok/data/error` envelope, stdout/stderr rules, and exits `0`–`5`;
- help and version retain E7-S1's human metadata behavior;
- confirmations, cancellations, no-ops, partial uninstall details, archive claim conflicts, and unarchive impact remain unchanged;
- audit failures found during development are fixed at their owning CLI adapter and do not receive an E7-S2-specific error code.

## Test plan

### Command and help matrix

- Assert root discovery of `init`, `uninstall`, `repo`, `agent`, and `task`.
- Assert `repo status`, `agent register/list`, and the six owned task leaves.
- Check representative leaf help for required task fields, list archive scope, update set/clear options, archive reason/force/yes/agent, unarchive yes without agent, and global JSON visibility.
- Verify representative undeclared aliases and unsupported actor placement fail through normal parse behavior without domain mutation.

### Representative behavior matrix

- Repository: initialize, repeat-init classification, inspect, uninstall dry-run, and one confirmed cleanup path.
- Agent: register and list identities/claims, including empty and populated results.
- Task query/mutation: minimum create, detail, filtered/archived list, combined scalar/structured update, valid no-op, archive, and unarchive.
- Actor: logical-user default plus registered-agent create/update/archive; reject unsupported agent selection for unarchive and read-only operations.
- Transport: representative success, empty/no-op, validation, not-found, conflict, and permission results in human and JSON modes, reusing E7-S1 assertions.
- Worktrees: one cohesive linked-worktree journey against the canonical main-root store.

### Regression boundary

- Keep all E1/E2 focused tests passing without changing expected domain payloads, stable errors, ordering, transaction behavior, or persistence.
- Do not duplicate detailed filesystem, validation, hierarchy, claim-race, downstream-impact, or SQLite rollback matrices already owned by dependencies.
- If the audit finds no production gap, a test/help-only implementation is a valid result.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test the owned command inventory and one full lifecycle from both the main worktree and a linked worktree.

## Definition of done

- Every E7-S2 functional and non-functional acceptance criterion passes.
- All owned repository, agent, and task operations are discoverable and connected to authoritative core behavior.
- Help accurately exposes arguments, actor support, query scope, and lifecycle confirmation options.
- Representative commands inherit E7-S1 transport and never introduce leaf-specific JSON or prompt behavior.
- Main and linked worktrees use the same canonical store.
- No alias, aggregate workflow, implicit actor state, migration, persistent field, domain operation, or E7-S2-specific payload/error is introduced.
- Existing E1/E2 behavior and focused tests remain stable; formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E7-S2 epic](../epics/E7-S2-expose-repository-agent-and-task-operations.md)
- [PRD E7-S2 story](../PRD.md#story-e7-s2-expose-repository-agent-and-task-operations)
- [E7-S1 output contract](../epics/E7-S1-provide-consistent-command-output.md)
- [E1 repository and agent contracts](../PRD.md#epic-e1-repository-foundation-and-identity)
- [E2 task lifecycle contracts](../PRD.md#epic-e2-task-content-and-lifecycle)
- Direct dependency epics and implementation tasks listed in frontmatter.
