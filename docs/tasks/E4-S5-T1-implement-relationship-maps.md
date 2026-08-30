---
id: E4-S5-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E4-S1-T1
  - E4-S2-T1
  - E4-S3-T1
  - E5-S1-T1
implements:
  - E4-S5
---

# E4-S5-T1: Implement relationship maps

## Epic

[E4-S5: View relationship maps](../epics/E4-S5-view-relationship-maps.md)

## Objective

Implement a reusable, cycle-safe relationship-map query and deterministic CLI rendering for recursive hierarchy, dependency, and combined context around one task.

## Readiness

Planning is complete. E4-S1-T1 and E4-S2-T1 provide the authoritative hierarchy and dependency graphs; E4-S3-T1 provides the exact availability predicate; E5-S1-T1 provides shared claim hydration. All dependencies are implemented, so this task is ready.

## Deliverables

- Typed direction, node, edge, and map-result models in core.
- Caller-owned read queries for all four recursive directions and combined maps.
- Node state hydration using shared status, claim, availability, and dependency-satisfaction primitives.
- `tbtm task map` parsing plus stable human and JSON output.
- Core, CLI, traversal, snapshot, performance, linked-worktree, and regression tests.

## Proposed structure and technical choices

- Extend the core task relationship modules, extracting a focused map module if it keeps hierarchy/dependency SQL and typed assembly cohesive.
- Reuse E2-S2 related-task/status models, E4-S1 hierarchy orientation, E4-S2 dependency orientation and effective completion, E4-S3 availability, and E5 claim summaries.
- Represent nodes once by stable task ID and edges once by `(type, direction, fromTaskId, toTaskId)`. Track shortest depth per selected direction in stable `reachedBy` items and retain all valid reached edges.
- Use recursive CTEs or equivalent bounded-query traversal for each selected graph direction, union their results for `all`, then hydrate tasks/statuses/claims and derive state in bounded batches.
- Keep an explicit visited set or cycle-safe SQL path guard even though valid databases are acyclic. Do not introduce persisted map, depth, blocked, or ready state.
- Open and own the standalone read snapshot in the operation layer while keeping selectors callable with a borrowed connection or transaction.

## Implementation flow

### 1. Define the map contract

Add direction values `upstream`, `downstream`, `parent`, `child`, and `all`, defaulting CLI parsing to `all`. Define `RelationshipMap`, `RelationshipNode`, and `RelationshipEdge` with camelCase serialization and stable enum values.

Nodes contain the shared related-task fields, `archived`, nullable shared claim, `blocked`, `ready`, and sorted `reachedBy: [{direction, depth}]`. The root has empty `reachedBy`; every other node records the shortest depth for each selected direction that reaches it. Edges distinguish hierarchy/dependency and selected traversal direction. Their authoritative endpoint orientation is parent to child for hierarchy and upstream to downstream for dependency.

### 2. Implement directional traversal

- Upstream dependency traversal follows downstream-to-upstream edges.
- Downstream dependency traversal follows upstream-to-downstream edges.
- Parent traversal follows child-to-parent edges through every ancestor.
- Child traversal follows parent-to-child edges through every descendant.

Seed depth 0 with the root, assign reached nodes their shortest depth independently per direction, deduplicate nodes, and retain all distinct reached edges. Traverse through archive and completion states without E4-S4's unresolved-only stop rule. For `all`, combine results without converting hierarchy edges into dependencies or vice versa.

### 3. Hydrate state in one snapshot

Load all node task/status rows and claims in bounded batches. Calculate `ready` with the exact E4-S3 predicate. Calculate `blocked` from existence of at least one direct upstream not effectively completed, even when another state already makes the task unavailable.

Sort nodes by minimum reached depth then task ID, with the root first. Sort `reachedBy` in parent/child/upstream/downstream order. Sort edges by type (`hierarchy`, `dependency`), direction (`parent`, `child`, `upstream`, `downstream`), source ID, then target ID. Return `TASK_NOT_FOUND` before traversal when the root is absent.

### 4. Add the read-only operation and CLI

Expose:

```text
tbtm task map <task-id> [--direction <upstream|downstream|parent|child|all>] [--json]
```

Resolve the canonical repository, open the database with the established read-only/no-create path, start one read snapshot, and apply no migrations. Map malformed directions to shared argument errors and a missing root to exit 3.

JSON returns `{rootTaskId, direction, nodes, edges}` under the shared success envelope. It never returns partial traversal data on failure.

### 5. Render deterministic human maps

Render the exact header and full-node target grammar documented by the epic, then the applicable non-empty sections in fixed `Parent hierarchy`, `Child hierarchy`, `Upstream dependencies`, `Downstream dependencies` order. Per section, form a deterministic breadth-first spanning tree from shortest-depth edges. The traversal predecessor is downstream for upstream traversal, upstream for downstream traversal, child for parent traversal, and parent for child traversal. Break competing-predecessor ties by predecessor ID then reached-node ID; beneath each predecessor, sort full-node and reference occurrences together by reached-node ID. Render every non-tree reached edge once beneath its traversal predecessor as `↩ <task-id> (already shown)`; do not enumerate every possible path.

Full node lines use `[type, status-code]` followed by applicable badges in this order: archived, completed, claimed, blocked, ready. Archive alone does not add the completed badge. Use the epic's exact UTF-8 connectors and three-character indentation units. Do not adapt wrapping or columns to terminal width.

Use `No relationships found.` for empty `all`; use `No <direction> relationships found.` for an empty single direction. Snapshot every exact phrase, heading, badge, and reference marker.

### 6. Preserve module boundaries and compatibility

Keep SQL, traversal semantics, derived state, and typed ordering in core. Keep Clap parsing and presentation in CLI. Do not change `task hierarchy`, `task blockers`, `task view`, compact lists, dependency mutations, or claim behavior; add regression coverage proving their existing contracts remain stable.

## Output and exit contract

| Exit | Outcome |
|---:|---|
| 0 | Map returned, including a root with no relationships |
| 2 | Invalid direction or malformed arguments |
| 3 | `TASK_NOT_FOUND` |
| 5 | Permission denied |

Repository, database, incomplete-schema, and unexpected failures retain shared codes and exits. The command never mutates data and never returns claim-conflict exit 4 merely because a node is claimed.

## Test plan

### Core traversal and model tests

- Exercise all directions and default `all` with zero, one, deep, branched, diamond, shared-node, and mixed hierarchy/dependency graphs.
- Assert root inclusion, per-direction shortest depths, canonical node deduplication, all-edge retention, graph/type/direction separation, and exact node/`reachedBy`/edge sorting.
- Include archived and completed nodes at multiple depths and prove traversal continues through them.
- Feed corrupt cyclic fixtures to traversal boundaries and prove termination without hiding normal integrity failures.

### State tests

- Verify ready, blocked, claimed, completed, and archived independently and in meaningful combinations.
- Cross-check ready against E4-S3 and blocked against unresolved direct dependency state; prove blocked is not generic unavailability.
- Verify claim ID, display name, and claimed timestamp hydration and snapshot consistency.

### CLI, snapshot, and performance tests

- Snapshot every direction and `all` in human and JSON modes, including exact target grammar, headings, breadth-first spanning-tree choice, sibling order, UTF-8 indentation, non-tree edge reference placement, badge order, empty messages, and non-adaptive terminal behavior.
- Verify missing task, invalid direction, pending/incomplete schema, permission, and operational errors with exact exits and no partial stdout.
- Coordinate relationship/status/archive/claim writes with an open read and prove each result uses one internally consistent snapshot.
- Inspect query count/plans on graphs with thousands of tasks and prove bounded query growth without per-node loading, truncation, or depth caps.
- Run from main and linked worktrees and confirm identical committed maps and no database or repository writes.
- Re-run hierarchy, dependency, blocker, available, task-view, and claim regressions.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test a temporary repository containing mixed hierarchy/dependency diamonds, shared nodes, all badge states, archived/completed intermediate nodes, and access from main and linked worktrees.

## Definition of done

- Every E4-S5 functional and non-functional acceptance criterion passes.
- All recursive directions and combined maps return complete deterministic nodes, edges, per-direction depths, state, and human trees without ambiguous duplication or path enumeration.
- Read behavior is one-snapshot, read-only, migration-free, bounded-query, cycle-safe, local, and linked-worktree consistent.
- Human and JSON output, empty results, errors, and exits match the approved contract exactly.
- Existing hierarchy, dependency, availability, blocker, task-detail, and claim behavior remains compatible.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [PRD hierarchy model](../PRD.md#77-hierarchy)
- [PRD dependency model](../PRD.md#78-dependency)
- [PRD availability rules](../PRD.md#8-availability-and-selection-rules)
- [PRD FR-10: Relationship map](../PRD.md#fr-10-relationship-map)
- [PRD E4-S5 story](../PRD.md#story-e4-s5-view-relationship-maps)
- [E2-S2 task detail and listing](../epics/E2-S2-view-and-list-tasks.md)
- [E4-S1 hierarchy management](../epics/E4-S1-manage-task-hierarchy.md)
- [E4-S2 dependency management](../epics/E4-S2-manage-dependencies.md)
- [E4-S3 available-task querying](../epics/E4-S3-query-available-tasks.md)
- [E4-S4 blocking explanations](../epics/E4-S4-explain-blocking.md)
