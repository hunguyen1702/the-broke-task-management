# Acceptance test setup

Acceptance scenarios exercise normal user workflows through the public `tbtm`
CLI. Detailed transaction, migration, race, locking, and fault-injection checks
belong in the automated Rust test suite.

## Common setup

1. Build the CLI with `rtk cargo build -p tbtm` and set `TB_BIN` to its absolute
   path.
2. Create a new temporary directory for each scenario.
3. Run `git init` there when a Git repository is needed.
4. Run `"$TB_BIN" init --prefix acc --json` unless initialization itself is
   under test.
5. Capture generated agent and task IDs from JSON output for later commands.
6. Never run a scenario against the development repository's `.tbtm` data.

## Common fixtures

- **Initialized repository:** a temporary repository initialized with prefix
  `acc`.
- **Two agents:** an initialized repository with `agent-a` and `agent-b`.
- **Basic tasks:** three active tasks and one completed task.
- **Related tasks:** basic tasks with a parent-child relation and a dependency.
- **Linked worktree:** an initialized Git repository with one linked worktree.

Scenarios may add the small amount of state needed by their cases. A setup
failure blocks the scenario; the executor should not invent replacement data.

## Result and cleanup

A case passes when its command has the expected exit code and its user-visible
output contains the stated values. Record the command and output for a failure.
Remove the temporary directory and linked worktree after the scenario.

For every failed case, create or update one follow-up file under `docs/tasks/`
before closing the run:

- Search existing tasks and prior run summaries first; update the existing open
  task when it tracks the same failure instead of creating a duplicate.
- Use the affected story's next task ID and record the scenario ID, run ID,
  failure evidence, and classification in the task.
- Classify the failure as an implementation defect, scenario drift, contract
  ambiguity, or environment/setup problem. Do not assume every mismatch needs
  a source-code change.
- Define the expected correction, verification, and acceptance impact. A failed
  common workflow normally requires `revalidate` and rerun after correction.
- Add the follow-up task to `docs/STATUS.md` with status `ready`, or document why
  it is blocked. Acceptance execution still must not modify implementation or
  expected behavior while diagnosing the failure.
