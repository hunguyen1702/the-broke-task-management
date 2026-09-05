---
id: E4-S5-T2
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E4-S5-T1
implements:
  - E4-S5
acceptance_failure:
  scenario: AT-E4-S5-001
  run: 2026-09-04-f02774b
  classification: implementation_defect
---

# E4-S5-T2: Restore child traversal in combined maps

## Parent story

[E4-S5: View relationship maps](../epics/E4-S5-view-relationship-maps.md)

## Failure evidence

The 2026-09-04 acceptance run created a root task with a direct parent, child,
upstream dependency, and downstream dependent. `tbtm task map <id>
--direction all --json` returned the root, parent, upstream, and downstream
nodes, but omitted the direct child and its hierarchy edge.

This is an implementation defect. E4-S5 defines `all` as the combination of
recursive `parent`, `child`, `upstream`, and `downstream` maps.

See the [acceptance run summary](../testing/runs/2026-09-04-f02774b/summary.md#at-e4-s5-001--view-all-relationships).

## Objective

Make combined relationship maps include child traversal with the same complete,
deterministic behavior as the standalone `child` direction.

## Deliverables

- Identify why `all` omits reachable child nodes and hierarchy edges.
- Reuse the authoritative child traversal when assembling combined maps.
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

The acceptance fixture must return the root, parent, child, upstream, and
downstream nodes plus all corresponding edges for `--direction all`.

## Acceptance impact

`revalidate`: AT-E4-S5-001 must be checked against the unchanged E4-S5 contract
and rerun after the fix.

