---
id: E6-S3
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E6-S1
  - E6-S2
---

# E6-S3: Keep comments immutable

## Outcome

Readers can trust that a comment returned by TBTM retains its original content, author, task association, stable ID, and creation time until an authorized actor deletes the complete comment.

## User story

As a reader, I want existing comments not to change so that previously read context is not silently rewritten.

## Product decisions

### Public mutation boundary

- TBTM exposes no CLI or core operation that edits an existing comment's content or metadata.
- E6-S1 `comment add` is the only way to create comment content. It always creates a new UUID and creation timestamp.
- E6-S2 `comment delete` removes the complete existing row under its ownership rules. It does not retain or mutate a tombstone.
- Immutability covers content, task association, author, comment ID, and creation timestamp. There is no supported metadata-only edit.
- Every public consumer must preserve this boundary. A future Visual Studio Code comment UI, including E8-S6, may compose add/list/delete behavior but must not expose an edit operation.
- Direct manipulation of the repository SQLite database is outside the supported product API and trust boundary.

### Correction workflow

- A correction uses two explicit existing commands: delete the incorrect comment, then add the corrected text as a new comment.
- The replacement has a new stable ID, captures its own creation timestamp, and uses the actor selected for the add operation. Only the UUID is guaranteed to differ; timestamp values need not be unequal. It is not presented as the original comment.
- Delete and add remain independent operations. TBTM provides no atomic replace command, automatic retry across both operations, or rollback that restores a deleted comment when the subsequent add fails.
- E6-S2 authorization continues to govern deletion. A reader or agent that cannot delete the original may add a follow-up clarification but cannot rewrite or remove the original through E6-S3.
- The workflow applies equally to active and archived tasks and preserves all E6-S1/E6-S2 task-state invariants.

### Persistence and compatibility

- No schema migration, SQLite update-prevention trigger, edit-history table, revision number, or `updatedAt`/`updatedBy` comment fields are introduced.
- The authoritative `task_comments` row remains insert-or-delete through supported operations. Existing E6-S1 ordering and JSON/human projections remain unchanged.
- A focused architectural guard allowlists the public core comment operations and rejects production `UPDATE task_comments` SQL; CLI command-surface tests reject edit-like subcommands. Future consumers, including the UI, must use that guarded core surface.
- Minimal production refactoring is allowed only if implementation-time inspection or tests expose an existing supported update path; public add, list, and delete behavior must remain stable.

## Functional acceptance criteria

1. No TBTM CLI command or supported core operation edits an existing comment's content, author, task association, ID, or creation timestamp.
2. Adding corrected text creates a distinct comment with a new UUID and a freshly captured valid creation timestamp and leaves the original unchanged while it exists; timestamp inequality is not required.
3. Correction by delete then add follows E6-S2 deletion authorization and E6-S1 creation/actor rules.
4. If deletion fails, the original comment remains unchanged and no replacement is created by that command; if a later add fails after a successful deletion, TBTM does not restore the deleted comment automatically.
5. The correction workflow works on active and archived tasks without changing task content, status, archive reason/state, claim, hierarchy, dependencies, `updatedAt`, or `updatedBy`.
6. Existing add, list, and delete human/JSON output, errors, exits, ordering, and task-scoped behavior remain unchanged.

## Non-functional acceptance criteria

1. Comment immutability is expressed as a shared domain invariant and regression-tested across core and CLI boundaries.
2. No migration or persistent revision/audit state is added solely for this invariant.
3. Existing E6-S1 add/list and E6-S2 transactional delete behavior remains safe across concurrent local processes and linked worktrees.
4. Tests distinguish unsupported in-place editing from the supported delete-and-add sequence without depending on private database manipulation as a product workflow.
5. Core continues to own comment domain mutation behavior; CLI continues to own parsing, rendering, and exit mapping.

## Verification

- Run focused architectural guards over the public core comment operations and production comment SQL, and inspect the CLI tree to prove that only add, list, and ownership-aware delete are available.
- Add a comment, exercise unrelated task and comment operations, and verify the complete persisted and serialized comment remains byte-for-byte/logically unchanged.
- Delete an authorized comment and add corrected content; verify the new ID, timestamp, actor attribution, ordering, and absence of any link that presents it as an edit.
- Exercise forbidden/missing deletion and failed add paths independently and verify their documented non-atomic outcomes.
- Repeat representative correction flows for active and archived tasks and from a linked worktree.
- Snapshot help plus human/JSON behavior and run formatting, lint, and workspace tests.

## Out of scope

- A comment edit, update, amend, replace, or move command.
- Atomic delete-and-recreate correction or automatic restoration after a failed add.
- Revision histories, audit tombstones, soft deletion, recovery, or replacement links.
- Authentication beyond the E6-S1/E6-S2 actor-selection trust convention.
- Preventing a repository owner or external SQLite client from modifying database files directly.
- Changes to comment output shape, full-task projections, ordering, or pagination.
- Implementing the Visual Studio Code UI; E8-S6 must consume this immutable core contract when implemented.

## Implementation task

See [E6-S3-T1: Harden comment immutability](../tasks/E6-S3-T1-harden-comment-immutability.md).
