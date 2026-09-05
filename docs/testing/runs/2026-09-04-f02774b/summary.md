# Acceptance run summary

- Date: 2026-09-04
- Revision: `f02774b` with pre-existing uncommitted E6-S1 changes
- Platform: Darwin 25.6.0 arm64
- Scope: all 23 approved scenarios in the acceptance regression catalog
- Result: 86 passed, 2 failed, 0 blocked
- Cleanup: all temporary repositories and linked worktrees were removed

## Failures

### AT-E1-S1-001 — Initialize twice

Command:

```console
$ "$TB_BIN" init --prefix acc --json
```

Expected exit `4`; actual exit `2`:

```json
{"ok":false,"data":null,"error":{"code":"ALREADY_INITIALIZED","message":"ALREADY_INITIALIZED: valid TBTM workspace already exists","details":{}}}
```

The error code is correct, but the public process exit code does not match the
approved scenario.

Follow-up: [E1-S1-T2](../../../tasks/E1-S1-T2-align-repeat-init-acceptance-exit.md)
classifies this as scenario drift against the authoritative E1-S1 exit contract.

### AT-E4-S5-001 — View all relationships

Setup created a root task with one direct parent, child, upstream dependency,
and downstream dependent. Command:

```console
$ "$TB_BIN" task map "$TASK_ID" --direction all --json
```

Follow-up: [E4-S5-T2](../../../tasks/E4-S5-T2-restore-child-traversal-in-all-maps.md)
tracks the implementation defect.

Expected the root plus all four related tasks. Actual exit was `0`, but the
child task was absent from both `nodes` and `edges`:

```json
{"ok":true,"data":{"rootTaskId":"acc-task-e5d18f88","direction":"all","nodes":[{"id":"acc-task-e5d18f88","title":"Root"},{"id":"acc-story-a9fe5909","title":"Parent"},{"id":"acc-task-39c79aa8","title":"Upstream"},{"id":"acc-task-71c7563a","title":"Downstream"}],"edges":[{"type":"hierarchy","direction":"parent","fromTaskId":"acc-story-a9fe5909","toTaskId":"acc-task-e5d18f88"},{"type":"dependency","direction":"upstream","fromTaskId":"acc-task-39c79aa8","toTaskId":"acc-task-e5d18f88"},{"type":"dependency","direction":"downstream","fromTaskId":"acc-task-e5d18f88","toTaskId":"acc-task-71c7563a"}]},"error":null}
```

## Verification baseline

`rtk mise run format`, `rtk mise run lint`, and `rtk mise run test` all passed.
