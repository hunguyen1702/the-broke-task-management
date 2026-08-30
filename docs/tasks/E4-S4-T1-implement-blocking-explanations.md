---
id: E4-S4-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E4-S2-T1
  - E4-S3-T1
implements:
  - E4-S4
---

# E4-S4-T1: Implement blocking explanations

## Epic

[E4-S4: Explain blocking](../epics/E4-S4-explain-blocking.md)

## Objective

Implement a deterministic read-only command and reusable core query that explain every current availability reason and unresolved dependency chain for one task.

## Readiness

Planning is complete. E4-S2-T1 supplies the authoritative acyclic dependency graph and effective-completion rules, while E4-S3-T1 supplies the exact availability predicate and caller-owned query boundary. Both dependencies are implemented, so this task is ready.

## Deliverables

- Typed availability-reason, dependency-explanation item, and task-explanation models.
- Caller-owned core queries for top-level reasons and direct/recursive unresolved dependency context.
- `tbtm task blockers` parsing and deterministic human/JSON rendering.
- Core, CLI, graph, snapshot, performance, linked-worktree, output, and regression tests.

## Proposed structure

- Extend the existing task/availability modules in `crates/tbtm-core`, extracting a focused blocker module only if it improves separation of recursive graph SQL and typed results.
- Reuse E2-S2 status and related-task summaries, E5's shared claim summary, E4-S2 effective completion, and E4-S3 availability logic rather than introducing parallel representations.
- Add the `Blockers` task subcommand in `crates/tbtm-cli`, following existing repository resolution, envelope, error, and exit patterns.
- Keep repository opening, snapshot ownership, parsing, and rendering outside the reusable core selector.

## Technical choices

- Define availability reasons as stable serialized values `archived`, `completed`, `claimed`, and `dependencies_blocked`; construct them in this fixed order without short-circuiting.
- Define explanation data as:

```text
TaskBlockingExplanation {
    task_id,
    available,
    reasons,
    claim,
    unresolved_dependencies: { direct, recursive }
}
```

- Each dependency item reuses the existing related-task fields and adds sorted `blockedByTaskIds` for immediate unresolved upstreams included in the explanation graph.
- Select direct unresolved upstreams using the E4-S2 effective-completion rule. Recursively traverse only unresolved upstream edges and stop before archived or completed-status tasks.
- Classify depth-one nodes as `direct`; classify all unique nodes at depth greater than one as `recursive`, except any node also reachable at depth one remains only in `direct`.
- Deduplicate by stable task ID and sort both groups and every `blockedByTaskIds` collection by task ID.
- Derive `available` from the authoritative E4-S3 predicate and assert it is equivalent to `reasons.is_empty()` within the same snapshot.
- Use a recursive CTE or equivalent bounded traversal plus bounded hydration queries; do not issue one query per dependency node or persist derived state.

## Implementation flow

### 1. Define shared core contracts

Add typed reason values and explanation structures with camelCase serialization. Reuse the nested shared claim shape:

```json
{
  "agent": {"id": "...", "displayName": "..."},
  "claimedAt": "..."
}
```

Keep the core API callable with a borrowed SQLite connection or transaction so standalone reads and E2-S6 transactional impact calculation share authoritative primitives.

### 2. Calculate top-level reasons

Within the caller's snapshot, load the target and return `TASK_NOT_FOUND` when absent. Evaluate archive state, status completion, current claim, and existence of unresolved direct upstreams without precedence. Emit all applicable reasons in fixed order and set `available` from the same complete predicate used by E4-S3.

An available task returns empty reasons, `claim: null`, and empty dependency groups. A claimed task exposes the existing claim summary even when archive, completion, or dependency reasons also apply.

### 3. Traverse unresolved dependencies

Seed traversal with every direct upstream whose task is neither archived nor in a completed status. Continue only through immediate upstreams meeting the same unresolved condition; do not traverse through an effectively completed node.

Build one unique explanation graph. Put every depth-one node in `direct`, put unique deeper nodes not also direct in `recursive`, and compute each returned node's `blockedByTaskIds` from its immediate unresolved upstream edges whose endpoints are returned in either group. Sort all output by task ID.

Do not query hierarchy tables or infer dependency from task type or parentage. Rely on E4-S2's cycle prevention, while making traversal robust against duplicate paths in diamond graphs.

### 4. Own a read-only snapshot

The public operation resolves the canonical store and opens the database through the established read-only/no-create path. Begin one read transaction, load all explanation components, and finish without applying pending migrations or writing SQLite/repository artifacts.

Return the established incomplete-database error when the required schema is unavailable. Concurrent commits may make a completed result stale, but no response may mix states from different snapshots.

### 5. Add CLI output

Expose:

```text
tbtm task blockers <task-id> [--json]
```

The command accepts no `--agent`, filters, mutation flags, or confirmation. JSON returns the complete explanation under the shared success envelope. An available human result is exactly `<task-id> is available.`

Unavailable human output is deterministic:

```text
<task-id> is unavailable.
Reasons: <comma-separated-reasons>
Claim: <display-name> (<agent-id>) since <claimed-at>
Direct unresolved dependencies:
- <id> [<type>, <status-code>] <title>; blocked by: <comma-separated-ids-or-none>
Recursive unresolved dependencies:
- <id> [<type>, <status-code>] <title>; blocked by: <comma-separated-ids-or-none>
```

Reasons retain the fixed JSON order. The claim line appears only when a claim exists. Each dependency heading and its lines appear only when that collection is non-empty. Root output deliberately uses the stable task ID only; the agreed JSON/root model does not duplicate the root title.

Map outcomes as follows:

| Exit | Outcome |
|---:|---|
| 0 | Available or unavailable explanation returned |
| 2 | Argument validation failure |
| 3 | `TASK_NOT_FOUND` |
| 5 | Permission denied |

Database and other operational failures retain exit `1`; the command never produces claim-conflict exit `4` merely because it reports a claim.

### 6. Preserve reuse boundaries

Expose unresolved direct-upstream and explanation queries below the standalone repository-opening layer. E2-S6 must be able to calculate `otherUnresolvedUpstreamTaskIds` inside its write transaction without invoking CLI behavior or duplicating effective-completion SQL. E4-S5 remains free to build richer path and hierarchy maps without changing this compact explanation contract.

## Test plan

### Core predicate and reason tests

- Return the exact available shape for an active, incomplete, unclaimed task with all dependencies resolved.
- Exercise archived, completed, claimed, and dependency-blocked reasons independently and in every meaningful combination; assert complete fixed-order reason sets.
- Verify the shared nested claim shape and preserved `claimedAt` for claimed explanations.
- Cross-check `available`, empty reasons, and E4-S3 selection for the same snapshot states.

### Dependency traversal tests

- Cover zero, one, multiple, deep, branched, diamond, and shared-ancestor dependency graphs.
- Verify direct/recursive membership, direct precedence when a node is reachable at multiple depths, deduplication, task-ID ordering, and immediate `blockedByTaskIds`.
- Put archived and completed-status nodes at different depths and prove traversal stops without exposing ancestors behind satisfied edges.
- Add hierarchy-only relations and prove they neither block nor appear.
- Change dependency, archive, and status state between reads and prove results are derived without cached data.

### Snapshot, performance, and integration tests

- Coordinate concurrent claim, status, archive, and dependency commits with an open explanation read and prove each response is internally snapshot-consistent.
- Inspect query counts or plans with thousands of tasks and deep/branched graphs to prove bounded traversal and hydration without N+1 growth.
- Invoke shared helpers using both a read connection and caller-owned transaction to protect E2-S6 reuse.
- Run from main and linked worktrees and assert identical committed results from the canonical store.
- Verify pending migrations are not applied, no read artifacts are created, and stable incomplete-database errors replace raw missing-table failures.

### CLI and regression tests

- Snapshot exact available human text, every unavailable human section combination, and complete JSON output.
- Verify missing-task JSON/human errors, malformed arguments, unsupported `--agent`, exact exits, and no partial stdout.
- Assert the command never changes tasks, statuses, claims, dependencies, timestamps, actors, or filesystem state.
- Re-run available query, specified claim, dependency, and task-view regression coverage around shared models and predicates.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test a temporary repository containing simultaneous reason states, a diamond-shaped unresolved graph with completed and archived stop nodes, an unrelated hierarchy, and access from main and linked worktrees.

## Definition of done

- Every E4-S4 functional and non-functional acceptance criterion passes.
- Available and unavailable tasks return complete, deterministic, multi-reason explanations with the stable human/JSON contract.
- Direct and recursive dependency context follows only unresolved edges, stops at effective completion, deduplicates shared paths, and never traverses hierarchy.
- Explanation reads are one-snapshot, read-only, migration-free, bounded-query, local, and linked-worktree consistent.
- Core explanation primitives are reusable from caller-owned transactions, including E2-S6, without duplicating availability or dependency semantics.
- Errors and exits are stable, no mutation occurs, and existing task/dependency/availability/claim behavior remains compatible.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [PRD effective completion](../PRD.md#76-effective-completion)
- [PRD dependency model](../PRD.md#78-dependency)
- [PRD availability rules](../PRD.md#8-availability-and-selection-rules)
- [PRD FR-7: Availability query](../PRD.md#fr-7-availability-query)
- [PRD E4-S4 story](../PRD.md#story-e4-s4-explain-blocking)
- [E2-S2 task detail and listing](../epics/E2-S2-view-and-list-tasks.md)
- [E2-S6 safe unarchive](../epics/E2-S6-unarchive-a-task-safely.md)
- [E4-S2 dependency management](../epics/E4-S2-manage-dependencies.md)
- [E4-S3 available-task querying](../epics/E4-S3-query-available-tasks.md)
- [E5-S1 specified-task claiming](../epics/E5-S1-claim-a-specified-task-atomically.md)
