# Acceptance test setup

Acceptance scenarios exercise normal user workflows through the public `tbtm`
CLI. Detailed transaction, migration, race, locking, and fault-injection checks
belong in the automated Rust test suite.

This document governs execution only. Scenario creation, editing, revalidation,
review, and approval must already be complete before execution begins. Never
change scenario text or expected results during a run; return stale definitions
to the separate acceptance-scenario build workflow.

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

### Failed case to remediation task

For every failed case, create or update one normal implementation task under
`docs/tasks/` before closing the run. A scenario failure means something must
be corrected before that scenario can pass; the correction may be to the
scenario, documentation/contract alignment, setup, or product code.

The acceptance executor creates the remediation task but does not perform it:

- Search existing tasks and prior run summaries first; update the existing open
  task when it tracks the same failure instead of creating a duplicate.
- Use the affected story's next task ID. Use the standard task frontmatter with
  `kind: implementation_task`, `planning_status: done`, and
  `implementation_status: ready`, then add it to `docs/STATUS.md` as `ready`.
- Never introduce a new task status for acceptance. Remediation tasks use the
  same `ready` → `in_progress` → `done` implementation lifecycle as every other
  implementation task. Scenario state and run results are tracked only under
  `docs/testing/`.
- Classify the failure as an implementation defect, scenario drift, contract
  ambiguity, or environment/setup problem. Do not assume every mismatch needs
  a source-code change.
- Treat `docs/PRD.md`, the approved epic, and approved task contracts as the
  source of truth. If they disagree, record the ambiguity instead of choosing a
  new behavior during execution.

Every remediation task must be implementable without rereading the full run.
Include:

- the failed scenario ID and run-summary link;
- exact reproduction input or command, including required fixture state;
- actual exit code and relevant user-visible output;
- expected exit code and user-visible output from the source of truth;
- the authoritative PRD/epic/task references and failure classification;
- the correction scope and explicit out-of-scope behavior;
- normal, invalid, no-op, regression, and transactional coverage relevant to
  the correction;
- verification commands and acceptance impact (`revalidate`, `add`, or
  `supersede`).

### Implement the remediation task

A different agent may implement the task through the normal approved-task
workflow. Before changing files, it claims the task by setting
`implementation_status: in_progress` in the task and `docs/STATUS.md`. After
the requested correction and task-level verification are complete, it sets
`implementation_status: done` in both places just like any other task.

Task completion means the correction described by the task is implemented and
verified. It does not mean the failed acceptance scenario has passed. Unless
the user separately asks that agent to run acceptance, the implementation agent
must not re-execute the scenario or update the acceptance result. It must not
add `acceptance_status`, `testing_status`, or any other task lifecycle field.

### Revalidate and rerun separately

After the remediation task is `done`, start a separate acceptance workflow:

1. In the acceptance-scenario build workflow, revalidate the affected scenario
   against the current source of truth and public CLI help.
2. If the scenario text changed, obtain the required approval and finish that
   workflow before execution.
3. Start a distinct execution workflow and run only approved scenarios in a
   fresh isolated repository.
4. Record pass/fail/blocked evidence and update `docs/testing/README.md` and the
   acceptance summary in `docs/STATUS.md`.
5. Link the rerun to the remediation task, but do not reopen or redefine the
   completed task merely because the scenario still fails. A remaining mismatch
   produces a new or updated remediation task through this same procedure.

These are deliberately separate workflows: creating a remediation task,
implementing and completing that task, and revalidating/executing acceptance.
