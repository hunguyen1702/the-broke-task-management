---
id: E6-S3-T1
kind: implementation_task
planning_status: done
implementation_status: ready
scenario_impact: add
scenario_impact_targets:
  - AT-E6-S3-001
depends_on:
  - E6-S1-T1
  - E6-S2-T1
implements:
  - E6-S3
---

# E6-S3-T1: Harden comment immutability

## Epic

[E6-S3: Keep comments immutable](../epics/E6-S3-keep-comments-immutable.md)

## Objective

Consolidate and regression-test the invariant that supported TBTM operations may insert, read, or delete a complete comment but never update one in place, while documenting delete-then-add as the explicit non-atomic correction workflow.

## Readiness

Planning is complete. E6-S1-T1 already supplies authoritative comment creation, listing, persistence, actor representation, and output. E6-S2-T1 must first implement ownership-aware deletion so the complete correction workflow and its failure boundaries can be tested. This task remains blocked until E6-S2-T1 is done.

## Deliverables

- A focused core and CLI regression matrix for immutable comment fields and delete-then-add correction.
- Public-help, command-surface, and focused core/SQL architectural guards proving that no supported comment edit/update operation exists.
- Active-task, archived-task, actor, failure, concurrency, and linked-worktree coverage using the authoritative E6-S1/E6-S2 operations.
- Minimal production fixes or shared-helper refactors only if tests expose an existing supported in-place update path.
- Updated E6-S3 planning and implementation status documentation after verification.

No migration, new command, output field, error code, edit history, or atomic replacement operation is expected.

## Proposed structure

Prefer extending the focused comment tests beside the authoritative implementation:

```text
crates/tbtm-core/src/comment.rs
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/tests/task_comment.rs
```

A dedicated test module may be added if it keeps the invariant matrix readable. Preserve the current core/CLI boundary and do not expose test-only mutation APIs in production code.

## Technical choices

- Treat `task_comments` as insert-or-delete through supported production operations. Do not add SQL `UPDATE` statements for comment rows.
- Define immutable state as the full persisted/public identity: `id`, `task_id`, `content`, author actor fields, and `created_at`.
- Reuse E6-S1 `add_comment`/`list_comments`, E6-S2's exact ownership-aware delete operation, canonical repository resolution, and existing transaction behavior.
- Test correction as two caller-controlled operations. A successful delete commits before add begins; an add failure does not compensate by restoring the old row.
- Do not add a SQLite trigger. Raw database access is outside the supported API, and existing tests may use controlled SQL fixture setup without representing product behavior.
- Add a focused architecture test that allowlists the public functions in the core comment module (`add`, `list`, and E6-S2 `delete`) and rejects production SQL containing `UPDATE task_comments`. This deliberately fails review when another public comment operation or in-place update path is introduced.
- Keep the same immutable core boundary for every future consumer. E8-S6 may expose add/list/delete in the Visual Studio Code UI but must not expose comment editing.
- Production refactoring is justified only to remove a demonstrated supported update path or make mutation ownership clearer. Preserve all public inputs, outputs, errors, exits, and transaction boundaries.

## Implementation flow

### 1. Establish the immutable-state assertion

Add a reusable test snapshot covering the comment row and serialized `TaskComment` fields:

```text
id, taskId, content, author, createdAt
```

Where direct persistence checks are useful, also capture the tagged author columns and exact task foreign key. Use the snapshot to detect changes caused by supported operations, not to promise protection from external SQL writes.

### 2. Lock the supported mutation surface

- Assert the Clap command tree exposes `comment add`, `comment list`, and E6-S2 `comment delete`, with no edit/update/amend/replace subcommand.
- Keep the core comment module free of a public update request or update function.
- Implement the focused architectural guard as a narrow allowlist over public function declarations in the comment module plus a check for `UPDATE task_comments` in production Rust/SQL sources. Scope it to comment mutation code rather than snapshotting unrelated source text.

### 3. Exercise invariants around existing operations

- Add several comments and prove later additions do not modify prior rows.
- List comments repeatedly and prove reads preserve every field and maintain E6-S1 chronological ordering.
- Exercise one representative unrelated parent-task mutation and archive/unarchive lifecycle case; assert comments remain unchanged without duplicating every owning story's matrix.
- Delete one comment and prove surviving comments remain unchanged and no retained row is rewritten into a tombstone.
- Run representative operations as logical user, owner agent, and foreign agent without changing E6-S1/E6-S2 authority semantics.

### 4. Verify correction and failure boundaries

For an authorized correction:

1. Capture the original comment.
2. Delete it through E6-S2 and verify the exact row is absent.
3. Add corrected content through E6-S1.
4. Verify a new UUID, a freshly captured valid RFC 3339 timestamp, current add actor attribution, and ordinary list ordering. Do not require the timestamp value to differ from the original.

For failure boundaries:

- A forbidden, missing, or otherwise failed delete leaves the original unchanged; no combined operation creates a replacement.
- After a successful delete, independently fail the add using invalid content, unknown actor, or another stable validation path. Verify the deletion remains committed and no automatic restoration occurs.
- Do not introduce a correction-specific output or error. Each step retains its owning command's result.

### 5. Cover lifecycle and concurrency

Repeat representative immutable-state and correction cases on active and archived tasks, asserting parent-task state and mutation metadata remain unchanged by comment operations.

Use one representative linked-worktree interleaving: retain an existing comment while another process adds a different comment, then list both and verify the retained row is identical. Reuse E6-S2's own concurrent-delete coverage rather than duplicating its race matrix. Do not invent atomicity across a caller's separate delete and add commands.

## Output and error contract

E6-S3 introduces no output or error contract:

- add and list retain the E6-S1 comment shape, human rendering, ordering, errors, and exits;
- delete retains the E6-S2 deleted-comment result, ownership errors, validation precedence, and exits;
- correction emits two independent command results and has no replacement ID, revision, or edit marker;
- help exposes no supported edit command.

Tests must verify outputs describe the same immutable comment state observed through the supported read API and, where appropriate, SQLite inspection.

## Test plan

### Core and persistence tests

- Capture a user- and agent-authored comment and assert all immutable fields survive later additions and repeated listing.
- Assert representative unrelated task lifecycle mutations never update comment rows.
- Delete one comment and assert every survivor is unchanged and no tombstone/revision row appears.
- Correct an owned/user-authorized comment via delete then add; assert old absence plus new ID, timestamp, content, and actor.
- Reject a foreign-agent delete and assert exact original retention.
- Fail add after committed delete and assert no replacement or automatic restoration.

### CLI, concurrency, and regression tests

- Snapshot `task comment --help` and reject edit/update-like subcommands through normal Clap behavior.
- Preserve exact E6-S1/E6-S2 human and JSON outputs and error exits during correction steps.
- Cover active and archived correction with unchanged parent task state.
- Run the named linked-worktree add/list interleaving and assert the pre-existing surviving row is identical; retain E6-S2's existing delete-race regression.
- Retain E6-S1 ordering, actor attribution, read-only listing, and E6-S2 ownership/transaction regressions.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test an authorized user correction and a forbidden agent correction on both active and archived tasks in isolated temporary repositories, including one linked-worktree read.

## Acceptance scenario impact

`add`: E6-S3 establishes the combined public correction journey and its partial-failure boundary even though it introduces no command. After implementation, add `AT-E6-S3-001` under the E6 comment catalog to cover authorized delete-then-add identity replacement, forbidden deletion preserving the original, and failed add after committed deletion without duplicating E6-S1/E6-S2 command-level cases.

## Definition of done

- Every E6-S3 functional and non-functional acceptance criterion passes.
- No supported CLI or core operation edits persisted comment content or metadata in place.
- Delete-then-add correction produces a new comment identity and follows existing E6-S1/E6-S2 actor, lifecycle, transaction, output, and failure behavior.
- Failed deletion preserves the original; failed add after committed deletion does not restore it.
- No migration, trigger, edit metadata, history, replacement link, new command, output field, or error code is introduced.
- Core, CLI, active/archived, actor, failure, concurrency, linked-worktree, and help-surface regressions pass.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E6-S3 epic](../epics/E6-S3-keep-comments-immutable.md)
- [PRD comment model](../PRD.md#710-comment)
- [PRD comments requirements](../PRD.md#fr-9-comments)
- [PRD E6-S3 story](../PRD.md#story-e6-s3-keep-comments-immutable)
- [E6-S1 add/view contract](../epics/E6-S1-add-and-view-comments.md)
- [E6-S1 implementation task](E6-S1-T1-implement-task-comments.md)
- [E6-S2 ownership-aware deletion contract](../epics/E6-S2-delete-comments-under-ownership-rules.md)
- [E6-S2 implementation task](E6-S2-T1-implement-ownership-aware-comment-deletion.md)
