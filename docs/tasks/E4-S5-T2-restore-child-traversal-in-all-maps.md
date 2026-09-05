---
id: E4-S5-T2
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E4-S5-T1
implements:
  - E4-S5
acceptance_failure:
  scenario: AT-E4-S5-001
  run: 2026-09-04-f02774b
  classification: environment_setup_problem
---

# E4-S5-T2: Restore child traversal in combined maps

## Parent story

[E4-S5: View relationship maps](../epics/E4-S5-view-relationship-maps.md)

## Failure evidence

The 2026-09-04 acceptance run created a root task with a direct parent, child,
upstream dependency, and downstream dependent. `tbtm task map <id>
--direction all --json` returned the root, parent, upstream, and downstream
nodes, but omitted the direct child and its hierarchy edge.

The failed run used a `task` root and a `task` child. E4-S1 permits task-like
types to have only epic or story parents, so that fixture could not create the
claimed child edge. Focused regression coverage confirms that `all` combines
recursive `parent`, `child`, `upstream`, and `downstream` maps when the fixture
uses a valid epic-to-story-to-task hierarchy.

See the [acceptance run summary](../testing/runs/2026-09-04-f02774b/summary.md#at-e4-s5-001--view-all-relationships).

## Objective

Restore reliable acceptance coverage for child traversal in combined maps and
lock the existing complete, deterministic behavior with focused regression
tests.

## Deliverables

- Identify why the failed run omitted the claimed child node and hierarchy edge.
- Correct the acceptance fixture to satisfy the authoritative hierarchy type
  matrix.
- Preserve node deduplication, `reachedBy`, edge orientation, ordering,
  snapshot consistency, cycle safety, and read-only behavior.
- Add focused core and CLI regression coverage for a root having all four
  relationship directions simultaneously.

- Revalidate and rerun AT-E4-S5-001 after implementation verification.

## Verification

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

The valid acceptance fixture must return the root, parent, child, upstream, and
downstream nodes plus all corresponding edges for `--direction all`.

## Acceptance impact

`revalidate`: AT-E4-S5-001 must be checked against the unchanged E4-S5 contract
and rerun after the fix.

## Completion

- The failed acceptance fixture used an invalid task-to-task hierarchy; E4-S1
  permits task-like children to have only epic or story parents.
- Focused core coverage proves combined selection includes recursive child
  traversal.
- Focused CLI coverage uses a valid epic-to-story-to-task hierarchy and proves
  `all` returns parent, child, upstream, and downstream nodes and edges.
- `rtk mise run format`, `rtk mise run lint`, and `rtk mise run test` passed on
  2026-09-05.
