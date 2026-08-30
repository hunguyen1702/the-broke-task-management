---
id: E4-S5
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E4-S1
  - E4-S2
---

# E4-S5: View relationship maps

## Outcome

Users and agents can inspect one task in recursive hierarchy and dependency context through deterministic human and JSON maps without conflating containment with execution order.

## User story

As a user or agent, I want directional relationship maps so that I can understand a task in context.

## Command

```text
tbtm task map <task-id> [--direction <upstream|downstream|parent|child|all>] [--json]
```

`--direction` defaults to `all`. The command is read-only, accepts no actor, and every direction is recursive.

## Product decisions

### Direction semantics

- The requested task is always the root and is included even when it has no relationships.
- `upstream` follows dependency edges from the root to every recursive prerequisite.
- `downstream` follows dependency edges from the root to every recursive dependent.
- `parent` follows the single-parent hierarchy from the root through every ancestor; the direct parent has depth 1.
- `child` follows hierarchy edges through direct children and every recursive descendant.
- `all` combines all four traversals while retaining relationship type and direction. Hierarchy never implies dependency and dependency never implies hierarchy.
- General maps traverse through active, archived, incomplete, and completed tasks. Unlike E4-S4 blocker explanations, completion does not stop traversal because the purpose is relationship context rather than unresolved-blocker discovery.

### Graph model and ordering

- JSON returns one canonical node per task ID and every distinct relationship edge reached by the selected direction. Shared or diamond paths do not duplicate nodes or discard valid edges.
- Each node contains the E2-S2 related-task identity, type, title, and status summary plus `archived`, nullable `claim`, `blocked`, `ready`, and `reachedBy`. `reachedBy` contains one `{direction, depth}` item for each selected direction that reaches the node, using that direction's shortest positive depth; it sorts in `parent`, `child`, `upstream`, `downstream` order. The root has an empty `reachedBy` collection.
- `ready` applies the exact E4-S3 availability predicate. `blocked` means the node has at least one direct upstream that is not effectively completed; it is not a generic synonym for unavailable.
- Claim uses the shared `{agent: {id, displayName}, claimedAt}` shape. Completion comes from `status.completed`; archive remains explicit even though it also provides effective completion.
- Nodes sort by their minimum `reachedBy` depth ascending and task ID ascending, with the root first. Edges sort by relationship type (`hierarchy`, then `dependency`), direction (`parent`, `child`, `upstream`, `downstream`), source task ID, then target task ID; only type/direction combinations applicable to the selected traversal occur.
- An edge contains `type` (`hierarchy` or `dependency`), `direction`, `fromTaskId`, and `toTaskId`. Direction is relative to the selected traversal. Authoritative endpoints are always parent to child for hierarchy and upstream to downstream for dependency.
- Traversal relies on the graph invariants from E4-S1 and E4-S2 and also tracks visited task/edge identities so corrupted cyclic data cannot loop indefinitely.

### JSON output

JSON uses the shared success envelope with:

```json
{
  "rootTaskId": "TBTM-task-a1b2c3d4",
  "direction": "all",
  "nodes": [],
  "edges": []
}
```

The root is the first node under the depth/ID ordering. An unrelated root returns one node and no edges. Fields use camelCase and collections remain deterministic.

### Human output

- Output begins with `Relationship map for <task-id> (<direction>)`, followed by a `Target:` line.
- `all` renders non-empty sections in fixed order: `Parent hierarchy`, `Child hierarchy`, `Upstream dependencies`, `Downstream dependencies`. A single direction renders only its corresponding section.
- Within each section, a deterministic breadth-first spanning tree chooses the first incoming traversal edge at the node's shortest section depth, breaking ties by traversal-predecessor task ID then reached-node task ID. The traversal predecessor is the downstream node for `upstream`, upstream node for `downstream`, child for `parent`, and parent for `child`. All full-node and reference occurrences beneath one traversal predecessor sort together by reached-node task ID. Each remaining reached edge renders once beneath its traversal predecessor as `↩ <task-id> (already shown)`; the renderer does not enumerate every possible graph path.
- A full node line is `<id> [<type>, <status-code>] <badges> <title>`. Applicable badges are independent and appear in fixed order: `[archived]`, `[completed]`, `[claimed: <display-name> (<agent-id>)]`, `[blocked]`, `[ready]`.
- `archived` is not accompanied by `completed` solely because archive supplies effective completion; `[completed]` represents status completion. Valid simultaneous badges, such as claimed plus blocked or completed plus claimed, remain visible.
- Empty `all` output ends with `No relationships found.` A selected empty direction uses `No <direction> relationships found.` Empty sections in `all` are omitted.
- Rendering does not adapt or truncate based on terminal width, keeping plain-text output stable and snapshot-testable.

Normative shape:

```text
Relationship map for TBTM-task-a1b2c3d4 (downstream)
Target: TBTM-task-a1b2c3d4 [task, in_progress] [claimed: worker-ab12cd34 (00000000-0000-0000-0000-000000000001)] Implement parser

Downstream dependencies:
├─ TBTM-task-b2c3d4e5 [task, to_do] [blocked] Integrate parser
│  └─ TBTM-task-d4e5f6a7 [testing, to_do] [blocked] Test integration
└─ TBTM-task-c3d4e5f6 [task, done] [completed] Document parser
   └─ ↩ TBTM-task-d4e5f6a7 (already shown)
```

Each level uses three characters: `├─ ` or `└─ ` for the current item and `│  ` or three spaces for inherited ancestor columns. The target uses the same full-node grammar without a tree connector. An empty single direction is exactly:

```text
Relationship map for TBTM-task-a1b2c3d4 (upstream)
Target: TBTM-task-a1b2c3d4 [task, in_progress] [ready] Implement parser

No upstream relationships found.
```

### Read behavior and errors

- Each invocation resolves the canonical store, opens the database read-only, applies no pending migration, and reads the root, nodes, edges, statuses, claims, and derived state from one SQLite snapshot.
- Core performs a bounded number of recursive and hydration queries rather than one query per node. The MVP has no depth option, result limit, pagination, or silent truncation and remains practical for personal repositories with thousands of tasks.
- Missing root returns `TASK_NOT_FOUND` with exit 3. Invalid direction or malformed arguments return exit 2. Repository, database, permission, and unexpected failures retain shared codes and exits and never return a partial map.

## Functional acceptance criteria

1. The command supports recursive `upstream`, `downstream`, `parent`, `child`, and combined `all` maps, with `all` as the default.
2. The root and every reachable archived, completed, active, claimed, ready, or dependency-blocked task remain visible with accurate snapshot state.
3. JSON exposes canonical deduplicated nodes, per-direction shortest-depth `reachedBy` metadata, and every reached edge with explicit relationship type, traversal direction, authoritative endpoints, and deterministic ordering.
4. Human output uses the documented fixed sections, tree paths, shared-node references, badge meanings and order, and exact empty-result messages.
5. Shared, diamond, and multi-relationship graphs retain every reached edge without ambiguous duplicate nodes, infinite loops, hierarchy/dependency conflation, or an unbounded enumeration of graph paths.
6. `ready` reuses the authoritative availability predicate, while `blocked` reports unresolved direct dependencies independently of archive, completion, and claim state.
7. Missing roots, invalid directions, human/JSON success, and operational failures use stable output and exit behavior without modifying repository state.

## Non-functional acceptance criteria

1. Every map is internally consistent within one read-only SQLite snapshot and applies no migration or persistent write.
2. Core owns typed traversal, node hydration, state derivation, deduplication, and ordering; CLI owns parsing and deterministic rendering.
3. Traversal uses bounded query count and cycle-safe visited tracking, with no N+1 query growth, silent truncation, or depth cap for repositories containing thousands of tasks.
4. Commands are local, network-free, linked-worktree consistent, and responsive for personal repository-sized graphs.
5. JSON is camelCase, timestamps are RFC 3339 UTC, and human output is stable regardless of terminal width.

## Verification

- Cover empty, single-edge, deep, branched, diamond, shared-node, and nodes connected through both relationship types in every direction and default `all`.
- Verify shortest-depth node classification, all-edge retention, section/collection ordering, reference markers, and exact empty human/JSON output.
- Exercise archived, completed-status, claimed, blocked, ready, and meaningful simultaneous badge combinations; cross-check E4-S3 availability and E4-S4 direct-blocker semantics.
- Prove general map traversal continues through completed and archived nodes, while hierarchy and dependency directions remain distinct.
- Inject or simulate corrupt cycles and prove traversal terminates without weakening the normal graph invariants.
- Verify one-snapshot consistency during concurrent writes, bounded query growth with thousands of tasks, pending-migration behavior, no writes, and main/linked-worktree equivalence.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Relationship mutation, blocker-only explanations, claiming, or availability guarantees.
- Filtering multiple roots, custom depth/size limits, pagination, shortest-path-only output, or persisted graph caches.
- Interactive graph visualization; E8-S7 owns the Visual Studio Code experience and may consume this stable core model.
- Cross-repository relationships or graph export formats beyond shared JSON output.

## Implementation task

See [E4-S5-T1: Implement relationship maps](../tasks/E4-S5-T1-implement-relationship-maps.md).
