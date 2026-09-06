---
id: E7-S3
kind: epic
planning_status: done
implementation_status: ready
contract_depends_on:
  - E3-S1
  - E3-S2
  - E3-S3
  - E3-S4
  - E4-S1
  - E4-S2
  - E4-S3
  - E4-S4
  - E4-S5
  - E7-S1
---

# E7-S3: Expose planning relationships and status operations

## Outcome

Users and coding agents can discover and invoke the complete status and
planning-relationship command surface from the terminal while retaining the
authoritative status, hierarchy, dependency, map, actor, and output contracts.

## User story

As a user or agent, I want hierarchy, dependency, map, and status commands so
that planning can be managed without the UI.

## Product decisions

### Owned command surface

E7-S3 consolidates and protects this existing command inventory:

```text
tbtm status list
tbtm status create
tbtm status rename
tbtm status move
tbtm status delete
tbtm status set-completed
tbtm task parent set
tbtm task parent remove
tbtm task dependency add
tbtm task dependency remove
tbtm task hierarchy
tbtm task map
```

- Existing names, nesting, required arguments, repeatable options, direction
  values, actor selectors, and confirmation flags remain authoritative.
- Direct parent and dependency state remains queryable through the shared full
  task detail, while `task hierarchy` and `task map` provide the established
  dedicated direct/recursive projections. E7-S3 adds no separate relationship
  list command.
- If implementation audit finds an owned command missing or disconnected, it
  is wired to its existing core operation without duplicating domain logic in
  the CLI.
- `task available` and `task blockers` belong to E7-S4 together with claim and
  work-acquisition operations. E7-S3 still depends on E4-S3 and E4-S4 because
  relationship-map node state reuses their authoritative `ready` and `blocked`
  meanings.
- Repository/task-content lifecycle, claim, and comment completeness remains
  owned by E7-S2, E7-S4, and E7-S5 respectively.

### Actor and authority boundaries

- Status configuration is logical-user-only. No `status` command accepts
  `--agent` or persists actor metadata.
- `task parent set/remove` and `task dependency add/remove` retain the optional
  `--agent <uuid>` selector; omission selects the logical user and successful
  changes update only the child/downstream task's mutation metadata.
- `task hierarchy` and `task map` are read-only and accept no actor selector.
- E7-S3 adds no implicit current agent, authentication mechanism, or broader
  authority under the banner of command consistency.

### Confirmation and non-interactive behavior

- `status set-completed` retains its impact-aware confirmation contract. A
  non-empty effective change requires accepted TTY confirmation or `--yes`.
- JSON and non-terminal invocations never prompt; when confirmation is needed,
  omission of `--yes` returns the existing confirmation-required error and
  exit `2`. Human TTY cancellation remains success exit `0` with no mutation.
- Same-value completion changes remain valid no-write no-ops. Status deletion,
  ordering, rename, relationship mutations, hierarchy reads, and maps do not
  gain confirmation flags.

### Help, output, and behavior preservation

- Root, `status`, `task`, `task parent`, and `task dependency` help make every
  owned operation discoverable at its established nesting.
- Representative leaf help exposes enough syntax to form valid commands,
  including placement choices, completion boolean/confirmation, child-oriented
  parent operations, downstream/upstream dependency roles, recursive hierarchy,
  and the five relationship-map directions.
- Every command inherits E7-S1's global idempotent `--json`, one-envelope
  response boundary, stream discipline, parse-error behavior, exit taxonomy,
  and non-interactive rules.
- Successful payloads, human projections, stable errors/details, validation
  precedence, ordering, no-op behavior, transaction boundaries, snapshot rules,
  and canonical-worktree resolution remain owned by E3 and E4.
- This story adds no migration, persisted field, status or graph semantics,
  alias, batch workflow, interactive editor, or network behavior.

## Functional acceptance criteria

1. Root and group help expose every owned status and relationship command with
   the established names and nesting.
2. Each owned command reaches its authoritative E3/E4 core operation and
   preserves arguments, payloads, human output, errors, exits, confirmations,
   no-ops, ordering, transaction behavior, and read-snapshot behavior.
3. Status commands remain logical-user-only; relationship mutations retain
   explicit optional agent attribution; relationship queries reject actor
   selection.
4. Parent and dependency relationships can be added, removed, and inspected
   through the established detail, hierarchy, and map projections without a
   duplicate relationship-query syntax.
5. `task map` exposes recursive `upstream`, `downstream`, `parent`, `child`, and
   `all` directions, with `all` as the default and unchanged deterministic
   human/JSON results.
6. `status set-completed` preserves explicit confirmation, no-op, impact, JSON,
   non-terminal, and TTY-cancellation behavior without introducing prompts to
   any other owned command.
7. Main and linked worktrees expose the same surface against one canonical
   repository store without creating linked-worktree-local state.
8. `task available`, `task blockers`, claim, and work-acquisition completeness
   remain scoped to E7-S4 and are not duplicated in this story's regression
   inventory.

## Non-functional acceptance criteria

1. CLI wiring and help remain in `tbtm-cli`; status/graph validation,
   persistence, transactions, traversal, derived state, and typed errors remain
   in `tbtm-core`.
2. A compact command-surface regression matrix detects missing commands,
   nesting/name drift, actor/confirmation drift, missing map directions, and
   bypasses of the shared E7-S1 transport.
3. Coverage reuses owning-story tests for detailed behavior and does not copy
   their migration, graph, ordering, concurrency, or output matrices.
4. Audit-driven production changes are minimal and correct only demonstrated
   exposure, wiring, help, or shared-transport gaps.
5. The story adds no database migration, persistent state, external service,
   or concurrency protocol.

## Verification

- Inspect root, `status`, `task`, `task parent`, and `task dependency` help plus
  representative leaf help for syntax, authority, confirmation, and direction
  discoverability.
- Exercise representative status query/mutations, relationship mutations,
  hierarchy reads, and every map direction in human and JSON modes.
- Verify agent placement, logical-user-only status configuration, confirmation
  requirements, no-op behavior, and rejection of unsupported options.
- Run a cohesive status-and-relationship journey from main and linked
  worktrees and confirm one canonical store.
- Preserve focused E3/E4/E7-S1 suites and run formatting, lint, and workspace
  tests through `mise`.

## Out of scope

- Redesigning command names, nesting, payloads, human output, domain errors,
  exit categories, actor rules, confirmation semantics, or map projections.
- `task available`, `task blockers`, claim/unclaim, comment, repository, agent,
  or task-content lifecycle command completeness.
- Convenience aliases, combined planning workflows, batch mutations,
  interactive editors, implicit current-agent state, or shell completion.
- New status states, relationship types, stored derived state, migrations,
  authentication, networking, or Visual Studio Code integration.
- Creating, editing, reviewing, approving, or executing acceptance scenarios
  during planning or implementation.

## Planning review and dependency cross-check

- The agreed scope assigns status and planning-relationship commands to E7-S3;
  E7-S4 owns availability, blocker, and claim command completeness.
- Decision review: `READY`.
- Final direct-document review: `READY`.
- Re-read E3-S1 through E3-S4, E4-S1 through E4-S5, E4-S5-T2, and
  E7-S1 with their implementation tasks after drafting this contract.
- Cross-checked status identity and ordering, logical-user authority,
  relationship endpoint roles and actor attribution, completion confirmation,
  no-op and mutation boundaries, direct/recursive reads, five-direction map
  semantics, ready/blocked derivation, canonical persistence, transactions,
  snapshots, output/error shapes, and shared JSON behavior.
- `contract_depends_on` matches the PRD and status dashboard. Task dependencies
  include every authoritative implementation consumed by the surface audit and
  the E4-S5-T2 child-traversal correction.
- Result: `NO CONFLICT`.

## Implementation task

See [E7-S3-T1: Harden the planning relationship and status CLI surface](../tasks/E7-S3-T1-harden-planning-relationship-and-status-cli-surface.md).
