---
id: E7-S5-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E6-S1-T1
  - E6-S2-T1
  - E6-S3-T1
  - E7-S1-T1
implements:
  - E7-S5
---

# E7-S5-T1: Harden the comment CLI surface

## Epic

[E7-S5: Expose comment operations](../epics/E7-S5-expose-comment-operations.md)

## Objective

Audit, complete, and regression-test the established comment CLI surface so add, deterministic list, and ownership-aware delete are discoverable and consistently invoke the immutable E6 domain behavior through the shared E7-S1 transport.

## Readiness

All direct implementation dependencies are complete. E6-S1-T1 supplies comment creation/listing, E6-S2-T1 supplies ownership-aware deletion, E6-S3-T1 hardens immutability and correction boundaries, and E7-S1-T1 supplies the shared CLI output boundary. The current CLI already exposes the expected leaves, so implementation starts with an evidence-driven audit and adds production changes only for demonstrated exposure, help, actor, or transport gaps.

## Deliverables

- The exact `task comment add/list/delete` inventory protected by a focused command-surface regression matrix.
- Root, task, comment-group, and leaf help checks for discoverability, stable nesting, required input, actor placement, and immutable correction guidance.
- Guards against a top-level comment alias, alternate content input, or edit/update/amend/replace/move operation.
- A compact integration matrix covering logical-user/agent ownership, active/archived tasks, human/JSON modes, empty/list ordering, and a linked-worktree journey.
- Minimal fixes for missing wiring, inaccurate help, actor-option drift, or E7-S1 transport bypasses found by the audit.
- Preservation of the authoritative E6 domain behavior and focused regression suites.

No migration, new domain operation, success/error payload, authentication behavior, acceptance-scenario change, or broad production refactor is expected.

## Proposed structure

Prefer extending the existing CLI definition and focused integration suites:

```text
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/task_comment.rs
crates/tbtm-cli/tests/command_surface.rs   # optional shared E7 matrix
```

Exact test placement follows the implemented repository. Keep domain operations and typed errors in `tbtm-core`; do not create CLI-owned comment rules or test-only production APIs.

## Technical choices

- Treat the Clap tree as the public exposure boundary and E6 core functions as the authoritative implementations.
- Maintain exactly the nested add/list/delete inventory. Test generated help and real process invocations rather than snapshotting private Rust enum layout.
- Preserve `--content` as the only add-content source and its E6 byte-preservation behavior. Do not add stdin, editor, or file adapters.
- Verify actor support by capability: add/delete accept optional exact UUID `--agent`; list does not. Omission continues to select logical user only where the owning operation defines an actor.
- Add concise comment-group help that identifies immutability and the two-command delete-then-add correction. Do not duplicate ownership, persistence, or failure specifications in CLI prose.
- Reuse E7-S1's inherited global output mode and response renderer. Correct any audited leaf-specific output behavior rather than adding a compatibility path.
- Test absence of unsupported public commands semantically through help and parse rejection. Keep E6-S3's focused core/SQL architecture guard authoritative for absence of in-place mutation.
- Reuse canonical repository fixtures and one linked-worktree flow. Detailed concurrency and transaction safety remain covered by E6.
- Prefer semantic assertions over full help snapshots to avoid noise from non-contractual Clap formatting.

## Implementation flow

### 1. Inventory current exposure

- Compare root, task, comment-group, and leaf help with the epic inventory.
- Trace add/list/delete handlers to `add_comment`, `list_comments`, and `delete_comment` and identify missing, duplicated, or CLI-reimplemented domain logic.
- Compare positional arguments, required content, actor flags, help descriptions, and inherited JSON behavior with E6/E7-S1.
- Record gaps in focused tests before changing production code.

### 2. Correct wiring and help gaps

- Wire any missing listed leaf directly to its authoritative core operation.
- Correct help or nesting that prevents a caller from forming a valid invocation.
- Make comment-group help state that correction is delete then add because existing comments are immutable and the two operations are independent.
- Do not add aliases, wrappers, alternate content inputs, or duplicate handlers.

### 3. Verify actor and ownership boundaries

- Exercise add without `--agent` and with an exact registered UUID; verify unchanged E6-S1 attribution.
- Assert list rejects an unsupported actor option before domain execution.
- Exercise delete as logical user, owning agent, and foreign agent; preserve E6-S2 task-scoped lookup, hard-delete result, and stable ownership rejection.
- Add no E7-S5-specific identity or permission error.

### 4. Verify lifecycle and immutable correction

- Run the same add/list/permitted-delete surface for active and archived tasks.
- Exercise correction as two caller-controlled invocations and preserve the new comment identity plus independent failure boundary.
- Assert no command exposes in-place editing and no comment operation changes parent task metadata, archive state/reason, claim, hierarchy, or dependencies.
- Rely on E6 focused suites for exhaustive immutable fields, failed-add-after-delete, races, and persistence checks.

### 5. Verify shared transport

- Exercise representative leaves with global and repeated `--json` placement.
- Assert one envelope, stream discipline, help/version behavior, parse errors, and existing exits without redefining payload fields.
- Verify all three comment operations are non-interactive and do not introduce prompts or confirmation flags.
- Preserve human add/list/delete and empty-state projections from E6.

### 6. Verify canonical-store integration

- From a linked worktree, add a comment through one actor, list it from the main worktree, and perform one permitted delete from either worktree.
- Confirm both worktrees observe the same comment state and no linked-worktree-local store is created.
- Keep exhaustive shared-store concurrency behavior in the owning E6 suites.

### 7. Consolidate regressions

- Add a table-driven command/help matrix that fails clearly for missing leaves, wrong nesting, actor drift, alternate content sources, edit-like exposure, or transport bypass.
- Keep representative end-to-end journeys compact and reuse E6/E7-S1 fixtures and assertions where practical.
- If the audit finds no production gap beyond help, a help-and-test-only change satisfies the task.

## Output and error contract

E7-S5 introduces no new output or error contract:

- add returns the E6-S1 comment shape and human projection;
- list returns E6-S1 comments ordered by `createdAt` then `id`, with the existing successful empty array/message;
- delete returns the E6-S2 deleted-comment JSON result and concise human confirmation;
- correction emits two independent owning-command results and has no replacement ID, revision, or edit marker;
- errors retain E6 task/content/agent/comment/ownership codes, details, validation precedence, and exits;
- every leaf uses E7-S1's global JSON envelope, stdout/stderr rules, parse behavior, help/version exception, and exit taxonomy.

## Test plan

### Command and help matrix

- Assert root/task discovery of `task comment` and comment-group discovery of exactly add, list, and delete.
- Check leaf help for task/comment positional IDs, required `--content`, actor availability on add/delete, no actor on list, and inherited global JSON visibility.
- Check group help for concise immutable delete-then-add correction guidance.
- Reject root-level aliases, edit-like leaves, alternate content flags, and unsupported actor placement through normal parse behavior without repository mutation.

### Representative behavior matrix

- Logical user: add preserved Markdown, list it, and delete it.
- Registered agents: add/list as one agent, delete as owner, and reject deletion as a foreign agent.
- Lifecycle: repeat representative add/list/delete and delete-then-add correction on an archived task without changing task state.
- Query: list zero, one, and several comments and preserve deterministic E6 ordering.
- Transport: exercise representative human output plus global/repeated JSON success, empty, validation, not-found, and forbidden results under E7-S1.
- Worktrees: one cohesive add/list/delete journey shared between main and linked worktrees.

### Regression boundary

- Keep all E6-S1/S2/S3 and E7-S1 focused tests passing without changing comment payloads, actor identity, ownership, ordering, task invariants, transaction behavior, or persistence.
- Do not duplicate detailed migration, UUID collision, read-only snapshot, fault-injection, race, SQL architecture, or portability matrices owned by dependencies.
- Verify parser rejection paths do not mutate repository or comment state.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test the three-leaf inventory, one logical-user journey, one agent ownership rejection, one archived correction, global/repeated JSON, and one linked-worktree flow in isolated temporary repositories.

## Acceptance scenario impact

Classify as `none`, `revalidate`, `add`, or `supersede` only after implementation from the actual public behavior changed. A non-`none` result is a handoff to the separate acceptance-scenario workflow; do not edit or execute scenario files here.

## Definition of done

- Every E7-S5 functional and non-functional acceptance criterion passes.
- The exact nested add/list/delete inventory is discoverable and connected to authoritative E6 core operations.
- Help accurately exposes required input, actor placement, immutability, and non-atomic correction without duplicating domain contracts.
- No alias, alternate content source, edit-like operation, implicit actor state, confirmation, migration, persistent field, domain operation, or E7-S5-specific payload/error is introduced.
- Logical-user, agent ownership, active/archived, empty/order, human/JSON, and main/linked-worktree representative flows pass.
- Existing E6 domain behavior and E7-S1 transport remain stable.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E7-S5 epic](../epics/E7-S5-expose-comment-operations.md)
- [PRD comment model](../PRD.md#710-comment)
- [PRD comment requirements](../PRD.md#fr-9-comments)
- [PRD CLI requirements](../PRD.md#fr-11-cli-experience)
- [PRD E7-S5 story](../PRD.md#story-e7-s5-expose-comment-operations)
- [E6-S1 add/list contract](../epics/E6-S1-add-and-view-comments.md)
- [E6-S1 implementation task](E6-S1-T1-implement-task-comments.md)
- [E6-S2 delete contract](../epics/E6-S2-delete-comments-under-ownership-rules.md)
- [E6-S2 implementation task](E6-S2-T1-implement-ownership-aware-comment-deletion.md)
- [E6-S3 immutability contract](../epics/E6-S3-keep-comments-immutable.md)
- [E6-S3 implementation task](E6-S3-T1-harden-comment-immutability.md)
- [E7-S1 output contract](../epics/E7-S1-provide-consistent-command-output.md)
- [E7-S1 implementation task](E7-S1-T1-standardize-cli-output-boundary.md)
