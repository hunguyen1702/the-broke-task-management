---
id: E7-S2
kind: epic
planning_status: done
implementation_status: ready
contract_depends_on:
  - E1-S1
  - E1-S2
  - E1-S3
  - E1-S4
  - E1-S5
  - E2-S1
  - E2-S2
  - E2-S3
  - E2-S4
  - E2-S5
  - E2-S6
  - E7-S1
---

# E7-S2: Expose repository, agent, and task operations

## Outcome

Users and coding agents can discover and invoke the complete repository, agent, and task-content lifecycle from one coherent CLI surface without learning private core APIs or encountering command-specific transport conventions.

## User story

As a user or agent, I want core entity operations in the CLI so that the complete workflow is terminal-accessible.

## Product decisions

### Complete command surface

E7-S2 consolidates and protects the command surface already introduced by E1 and E2:

```text
tbtm init
tbtm uninstall
tbtm repo status
tbtm agent register
tbtm agent list
tbtm task create
tbtm task view
tbtm task list
tbtm task update
tbtm task archive
tbtm task unarchive
```

- Root-level `init` and `uninstall` remain repository lifecycle operations; they are not moved below `repo`.
- Existing names, nesting, required arguments, filters, update options, lifecycle options, and confirmation flags remain authoritative. E7-S2 adds no aliases, aggregate workflow command, interactive wizard, or alternative syntax.
- If implementation audit finds a listed operation missing or disconnected, it is wired to the existing authoritative core operation. It must not reimplement domain logic in the CLI.
- Status, relationship, availability, claim, and comment command completeness belongs to E7-S3 through E7-S5 even though those commands may already exist.

### Actor placement

- Actor identity remains explicit on scoped mutations that the owning contracts permit a registered agent to perform: agent-capable task create, update, and archive operations accept `--agent <uuid>` and otherwise use the logical `user`.
- Repository lifecycle operations, repository inspection, agent registration/listing, and task view/list do not invent an actor argument when no actor attribution is persisted.
- `task unarchive` remains logical-user-only and does not gain `--agent`; E7-S2 does not broaden authority under the banner of consistency.
- Agent registration returns an identity for later explicit use but does not create global or worktree-local current-agent state.

### Help and discoverability

- Root and group help make every owned operation discoverable under the established hierarchy.
- Leaf help accurately describes required inputs, repeatable/replacement options, archive/unarchive confirmation flags, actor selection, and active/archived query scope as defined by the owning story.
- Help does not restate every domain rule or become a second behavior contract. It gives enough information to form a valid invocation and points through stable terminology already used by command results and errors.

### Output and behavior preservation

- Every operation consumes E7-S1's inherited, idempotent global `--json`, one-envelope response boundary, stream discipline, parse-error behavior, exit taxonomy, and non-interactive confirmation rules.
- Successful data payloads, human projections, stable error codes/details, ordering, validation precedence, idempotency, no-op behavior, confirmations, and transaction boundaries remain owned by E1/E2.
- All commands resolve the canonical E1-S5 repository store. E7-S2 creates no per-worktree state and no alternative resolver.
- This story is an integration and completeness boundary. No migration, persistent field, domain model, new lifecycle transition, or network behavior is introduced.

## Functional acceptance criteria

1. Root and group help expose every repository, agent, and task operation listed in this contract with the established names and nesting.
2. Each listed command reaches its authoritative E1/E2 core behavior and preserves its argument, payload, human output, error, exit, confirmation, idempotency, and transaction contract.
3. Agent-capable task mutations accept the explicit registered-agent UUID selector, omitted selectors use the logical user, and operations without agent authority or attribution expose no misleading actor option.
4. Every listed operation supports E7-S1 JSON placement and envelope behavior without a leaf-specific output-mode implementation.
5. Repository and task lifecycle confirmations remain non-blocking and safe for JSON and non-terminal invocation; read operations remain non-interactive.
6. Main and linked worktrees invoke the same command surface against one canonical repository store and return behavior consistent with the current worktree context.
7. No alias or made-up entity operation is introduced, and command families owned by E7-S3 through E7-S5 are not expanded by this story.

## Non-functional acceptance criteria

1. CLI wiring and help remain in `tbtm-cli`; domain validation, persistence, transactions, and typed errors remain in `tbtm-core`.
2. A compact command-surface regression matrix detects missing commands, accidental nesting/name changes, actor-option drift, and bypasses of the shared E7-S1 transport.
3. Coverage reuses owning story tests for detailed behavior and avoids duplicating their full edge-case matrices.
4. Audit-driven production changes are minimal and must correct a demonstrated exposure or consistency gap.
5. The story adds no database migration, persistent state, external service, or concurrency protocol.

## Verification

- Inspect root, `repo`, `agent`, and `task` help plus representative leaf help in human mode.
- Exercise one representative repository lifecycle operation, repository query, agent mutation/query, task mutation/query, and task lifecycle operation in human and JSON modes.
- Verify explicit actor placement and rejection on operations where agent identity is unsupported.
- Run representative commands from the main worktree and a linked worktree and confirm one canonical store is used.
- Preserve focused owning-story tests and run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Redesigning command names, nesting, payloads, domain errors, exit categories, or confirmation semantics.
- Adding convenience aliases, batch operations, interactive editors/wizards, implicit current-agent state, or shell completion generation.
- Status, hierarchy, dependency, availability, blocking, relationship-map, claim, unclaim, or comment exposure; E7-S3 through E7-S5 own those completeness contracts.
- New persistence, migrations, domain operations, authentication, networking, or Visual Studio Code integration.
- Creating, editing, reviewing, approving, or executing acceptance scenarios during planning or implementation.

## Implementation task

See [E7-S2-T1: Harden the repository, agent, and task CLI surface](../tasks/E7-S2-T1-harden-repository-agent-and-task-cli-surface.md).
