---
id: E7-S5
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E6-S1
  - E6-S2
  - E6-S3
  - E7-S1
---

# E7-S5: Expose comment operations

## Outcome

Users and coding agents can discover and invoke the complete immutable-comment workflow from the terminal with the same actor, lifecycle, output, and ownership behavior defined by the comment domain contracts.

## User story

As a user or agent, I want comment commands so that task context can be managed from the terminal.

## Product decisions

### Complete command surface

E7-S5 consolidates and protects the command surface introduced by E6:

```text
tbtm task comment add <task-id> --content <markdown> [--agent <uuid>]
tbtm task comment list <task-id>
tbtm task comment delete <task-id> <comment-id> [--agent <uuid>]
```

- Comments remain nested below `task` because every comment belongs to one task.
- Existing names, nesting, positional arguments, required options, and actor options remain authoritative.
- E7-S5 adds no root-level `comment` alias, shorthand, aggregate correction command, interactive editor, stdin input, or file-based content input.
- If implementation audit finds a listed operation missing or disconnected, it is wired to the authoritative E6 core operation rather than reimplementing domain logic in the CLI.

### Actor and ownership boundaries

- `comment add` accepts `--agent <uuid>` for an exact registered agent; omission selects the logical `user` actor.
- `comment list` is actor-independent and exposes no `--agent` option.
- `comment delete` accepts `--agent <uuid>` under E6-S2 ownership rules; omission selects logical-user authority.
- The logical user may delete any comment. A selected agent may delete only a comment authored by that exact agent UUID.
- Actor selection remains a local trust convention rather than an authentication boundary. E7-S5 adds no credentials, implicit current-agent state, or broader authorization policy.

### Immutable history and correction

- The CLI exposes exactly add, list, and ownership-aware hard delete. It exposes no edit, update, amend, replace, or move operation for existing comments.
- Correction remains two explicit commands: delete the incorrect comment, then add corrected content as a new comment with a new UUID and independently captured timestamp.
- Delete and add are separate committed operations. Failure of the later add does not restore the deleted comment, and E7-S5 adds no atomic replacement or compensation behavior.
- Group help briefly identifies comments as immutable and directs correction to delete then add. Leaf help stays focused on valid invocation, required values, and actor selection rather than duplicating all domain rules.

### Task lifecycle and output preservation

- Active and archived tasks use the same add, list, and permitted-delete command paths. No archive-specific alias, flag, confirmation, or payload is introduced.
- E6-S1 remains authoritative for comment identity, Markdown preservation, actor representation, chronological ordering, empty-list success, task-state preservation, and focused comment JSON shape.
- E6-S2 remains authoritative for task-scoped lookup, hard deletion, ownership failures, concurrency behavior, and deleted-comment result.
- E6-S3 remains authoritative for immutable fields, absence of edit operations, and the non-atomic correction boundary.
- Every command inherits E7-S1's global and idempotent `--json`, one-envelope response boundary, stream discipline, parse-error behavior, help/version exception, exit taxonomy, and non-interactive guarantees.
- E7-S5 introduces no new success payload, human projection, error code/detail, exit mapping, ordering rule, confirmation, or domain state transition.

### Integration boundary

- E7-S5 is a command-surface completeness and regression boundary. It begins with an audit of current wiring, help, and transport behavior.
- Production changes are limited to demonstrated missing wiring, inaccurate help, actor-option drift, transport bypass, or accidental edit-like exposure.
- A help-and-test-only implementation is valid when the audit finds no production behavior gap.

## Functional acceptance criteria

1. Root, `task`, and `task comment` help make the nested comment group and its exact add, list, and delete operations discoverable.
2. Each leaf reaches its authoritative E6 core operation and preserves required arguments, actor behavior, payloads, human output, errors, exits, ordering, and mutation boundaries.
3. Add and delete accept the exact optional registered-agent UUID selector and otherwise use logical-user behavior; list exposes no actor selector.
4. The command tree exposes no comment edit/update/amend/replace/move operation and no top-level comment alias or alternate content-input mechanism.
5. Group help states that comments are immutable and that correction is delete followed by add as two independent operations.
6. User-authored and registered-agent-authored comments can be added, listed, and deleted when permitted, while foreign-agent deletion retains E6-S2's stable rejection.
7. Active and archived tasks follow the same command flow and preserve all task metadata, archive state/reason, claim, hierarchy, and dependencies.
8. Empty comment history remains a successful deterministic result, and non-empty history remains ordered by `createdAt` then comment `id`, ascending.
9. Every comment command supports E7-S1 JSON placement/repetition, envelope, streams, parse failures, and exit behavior without leaf-specific transport handling.
10. Main and linked worktrees invoke the same comment surface against one canonical repository store.

## Non-functional acceptance criteria

1. CLI command definitions, help, argument parsing, and rendering remain in `tbtm-cli`; comment validation, identity, actor resolution, ownership, persistence, transactions, and ordering remain in `tbtm-core`.
2. A compact command/help regression matrix detects missing leaves, nesting/name drift, actor-option drift, alternate content inputs, edit-like operations, and E7-S1 transport bypasses.
3. Representative integration coverage reuses the detailed E6 tests instead of duplicating their full migration, failure, transaction, race, and portability matrices.
4. Comment operations remain synchronous, local, network-free, and safe across linked worktrees and concurrent local processes under the E6 contracts.
5. The story adds no database migration, persistent state, domain operation, authentication mechanism, or concurrency protocol.

## Verification

- Inspect root, task-group, comment-group, and all three leaf help surfaces; verify exact nesting, arguments, actor placement, immutable correction guidance, and absence of unsupported aliases or edit-like operations.
- Exercise a logical-user add/list/delete journey and a registered-agent journey, including a foreign-agent deletion rejection.
- Repeat one representative journey on an archived task and verify the same syntax, output, ownership, and unchanged task state.
- Exercise empty list and deterministic multi-comment list behavior without redefining the detailed E6 ordering suite.
- Run representative commands in human and global/repeated JSON modes and verify E7-S1 envelope, streams, parse behavior, and exits.
- Run one representative linked-worktree flow against the canonical main-worktree store.
- Preserve all focused E6 and E7-S1 tests and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- A comment edit, update, amend, replace, move, restore, batch, or atomic correction command.
- A top-level comment alias, interactive editor, stdin/file input, pagination, filtering, reverse ordering, reactions, attachments, or mentions.
- Changes to comment UUIDs, timestamps, Markdown preservation, actor representation, ownership, ordering, persistence, or full-task projections.
- Authentication, credentials, implicit current-agent selection, remote synchronization, or Visual Studio Code integration.
- Redesigning E7-S1 output envelopes, streams, parse behavior, help/version handling, or exit categories.
- Creating, editing, reviewing, approving, or executing acceptance scenarios during planning or implementation.

## Implementation task

See [E7-S5-T1: Harden the comment CLI surface](../tasks/E7-S5-T1-harden-comment-cli-surface.md).
