---
id: E4-S3-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E3-S1-T1
  - E4-S2-T1
  - E5-S1-T1
implements:
  - E4-S3
---

# E4-S3-T1: Implement available-task querying

## Epic

[E4-S3: Query available tasks](../epics/E4-S3-query-available-tasks.md)

## Objective

Implement a read-only available-task command backed by one reusable core selection query that applies the complete live availability predicate, agreed filters, deterministic ordering, and existing compact list output.

## Readiness

Planning is complete. E3-S1-T1 supplies authoritative completion semantics, E4-S2-T1 supplies dependency edges and effective-completion predicates, and E5-S1-T1 supplies authoritative active claims. All implementation dependencies are complete, so this task is ready.

## Deliverables

- Core available-query input and caller-owned connection/transaction selection API.
- SQLite query implementing the exact predicate, filters, and deterministic ordering without persisted derived state.
- Bounded tag and claim/list projection hydration within one snapshot.
- `tbtm task available` parsing plus compact human and stable JSON rendering.
- Core and CLI tests for predicate branches, filters, ordering, snapshot behavior, linked worktrees, outputs, errors, and exits.

## Proposed structure

- Keep public orchestration, inputs, and result types in `crates/tbtm-core/src/task.rs`, reusing `TaskListItem`, status summaries, claim shapes, archive resolution, and database helpers already used by task list and claim.
- Extract or introduce a private/shared SQL selection layer that accepts `&Connection` or a transaction-compatible borrowed connection and returns compact task rows for a supplied availability/filter input.
- Keep repository resolution and read-snapshot ownership at the public standalone operation boundary. The reusable selector must not open its own connection or transaction.
- Add the `Available` subcommand and args in `crates/tbtm-cli/src/main.rs`, reusing list filter parsing, JSON envelopes, task table rendering, and shared error mapping where practical.
- Add focused core tests beside task behavior and process-level command/output tests under `crates/tbtm-cli/tests/`.

## Technical choices

- Define an available-query input containing repeatable status codes, task types, and tags. Normalize duplicate values only if doing so preserves observable validation and output behavior.
- Validate every distinct status code inside the same read snapshot as selection. If any is unknown, return `STATUS_NOT_FOUND` before returning rows. Parse and reject invalid task types at the CLI boundary with `INVALID_TASK_TYPE`.
- Build the candidate set with all four conjuncts: `archived = false`, joined status `completed = false`, no active claim, and no direct upstream whose task is neither archived nor in a completed status.
- Prefer `NOT EXISTS` predicates for active claims and unresolved upstreams, or an equivalent SQL plan that cannot duplicate task rows. Zero upstreams must pass naturally.
- Apply type/status/tag filters only as additional restrictions. Repeat values use OR within a group; non-empty groups combine with AND. Exact tag membership is case-sensitive.
- Apply `priority DESC, created_at ASC, id ASC` in SQL after predicate and filters. Do not expose custom sorting or pagination.
- Reuse `TaskListItem` exactly; do not add `available`, blocker, or dependency fields. Hydrate tags and any shared projection data with a bounded number of queries.
- Do not add a migration or availability table/column. Read-only opening follows E2-S2 behavior: no pending migration application and stable incomplete-database failure when required schema is absent.

## Implementation flow

### 1. Define the reusable query contract

Add an input type for status/type/tag filters and a selector callable with a borrowed SQLite connection or transaction. Keep repository discovery, connection mode, and transaction creation outside the selector so E5-S2 can invoke the exact same selection behavior after beginning an immediate transaction.

The selector returns ordered `TaskListItem` values. It performs no writes, emits no CLI output, and makes no ownership guarantee.

### 2. Implement the complete SQL predicate

Select candidates only when:

```text
task.archived = false
AND task.status.completed = false
AND no active claim exists for task
AND no direct upstream exists that is not effectively completed
```

Treat an upstream as effectively completed when it is archived or its joined status is completed. Use current normalized task/status/dependency/claim tables and parameterized SQL. Ensure joins or tag filters cannot create duplicate result rows.

Validate supplied status codes using the same connection before candidate selection. Perform filtering and the fixed three-part ordering in SQLite where practical, then hydrate compact collections without N+1 queries.

### 3. Own one standalone read snapshot

The public `available_tasks` operation resolves the canonical store and opens it with the established read-only/no-create path. Begin one read transaction, validate filters, select candidates, hydrate projection fields, and complete the read using that snapshot.

Do not apply pending migrations, update task metadata, create SQLite sidecars beyond established read-only guarantees, or acquire a write lock. Concurrent commits may make the returned information stale after the snapshot, which is expected.

### 4. Add CLI parsing and rendering

Expose:

```text
tbtm task available [--status <code>]... [--type <type>]... [--tag <tag>]... [--json]
```

Reuse E2-S2 semantics: repeatable OR values within each group, AND across groups, machine status codes, fixed task types, and exact case-sensitive tags. Do not add archive flags because archived tasks can never be available.

Non-empty human output uses the compact task-list table. Empty human output is exactly `No available tasks found.` JSON uses the shared success envelope with `data: []` when empty.

Map validation and lookup failures as follows:

| Exit | Errors |
|---:|---|
| 0 | Successful non-empty or empty query |
| 1 | Database or other operational failure |
| 2 | `INVALID_TASK_TYPE` or argument validation |
| 3 | `STATUS_NOT_FOUND` |
| 5 | Permission denied |

Exit `4` remains reserved for claim conflicts and is not produced by this read-only command.

### 5. Preserve E5-S2 reuse boundary

Keep the caller-owned selector independent of read-only repository opening. Document through tests or API shape that a future immediate transaction can use the same filter validation, candidate predicate, and ordering before inserting a claim. Do not implement next-task claiming in this task.

## Test plan

### Core behavior

- Return a zero-dependency active, incomplete, unclaimed task and exclude it independently for archive, completed status, or active claim.
- Cover one and multiple upstreams: unresolved, completed-status, archived, mixed resolved, and mixed with one unresolved; prove every-upstream semantics and archived effective completion.
- Combine conditions, including claimed tasks that are also otherwise eligible or dependency-blocked, to prove the query only excludes rows and does not need error precedence.
- Change dependency edges, upstream status/archive state, and claims between calls and prove results track current persisted state without cached availability.

### Filters and ordering

- Exercise each status/type/tag group, repeated OR values, cross-group AND combinations, duplicates, exact tag case, and filters that match only unavailable tasks.
- Verify an unknown status returns `STATUS_NOT_FOUND`, an invalid type returns `INVALID_TASK_TYPE`, and a known completed-status filter succeeds with an empty result.
- Verify priority, creation-time, and ID ordering independently and with ties after predicate and filters.
- Assert no duplicate task rows when multiple tags or relationships match.

### Snapshot, performance, and integration

- Coordinate a reader with concurrent claim, dependency, status, or archive commits and prove every returned aggregate reflects one internally consistent snapshot; do not assert freshness after completion.
- Inspect query execution/counts with many tasks, tags, claims, and dependencies to prove bounded query growth and practical index use for thousands of repository tasks.
- Invoke the reusable selector through both a connection-owned read snapshot and a caller-owned transaction test harness to protect the E5-S2 integration boundary.
- Run from the main worktree and a linked worktree and assert identical committed results from the canonical store.
- Verify a pending required migration returns the established read-only incomplete-database error without applying migrations or writing repository state.

### CLI and regression coverage

- Snapshot non-empty compact human and JSON output, including `archived: false`, `claim: null`, and `status.completed: false`.
- Snapshot exact empty human output, JSON `data: []`, validation errors, error details, and exit codes.
- Verify available query does not change task rows, claim rows, dependencies, timestamps, actors, or repository filesystem artifacts.
- Re-run task list, task view, dependency, and specified-claim regression suites to protect the shared projection and predicate inputs.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

## Definition of done

- The standalone command implements the complete live predicate and fixed ordering for every returned row.
- Filters retain E2-S2 semantics and cannot weaken availability.
- Empty, success, and failure outputs use the documented projections, messages, errors, and exits.
- Reads are one-snapshot, read-only, migration-free, bounded-query, local, and linked-worktree consistent.
- The core selector can be called from a caller-owned transaction and is ready for E5-S2 reuse without duplicating selection rules.
- No derived availability state or unrelated source behavior is introduced.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [PRD availability and selection rules](../PRD.md#8-availability-and-selection-rules)
- [PRD FR-7: Availability query](../PRD.md#fr-7-availability-query)
- [PRD personal-project non-functional requirements](../PRD.md#12-personal-project-non-functional-requirements)
- [E2-S2: View and list tasks](../epics/E2-S2-view-and-list-tasks.md)
- [E3-S1: Use default statuses](../epics/E3-S1-use-default-statuses.md)
- [E4-S2: Manage dependencies](../epics/E4-S2-manage-dependencies.md)
- [E5-S1: Claim a specified task atomically](../epics/E5-S1-claim-a-specified-task-atomically.md)
