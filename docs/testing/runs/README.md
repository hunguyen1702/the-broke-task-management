# Acceptance run reports

When a run needs to be retained, create
`docs/testing/runs/<date>-<commit>/summary.md` containing:

- code revision and operating system;
- scenarios run;
- passed, failed, and blocked cases;
- actual command and output for failures;
- the follow-up task linked for every failed case;
- cleanup result.

Do not store temporary repositories, databases, binary output, hashes, or
successful command transcripts unless needed to diagnose a failure.

Before closing a failed run, follow the failure-to-task procedure in
[`test-setup.md`](../test-setup.md#failed-case-to-remediation-task). Creating the
task, implementing it through the normal `ready` → `in_progress` → `done`
lifecycle, and rerunning acceptance are separate workflows. A later run should
reuse the same open task for the same unresolved defect and link new evidence
rather than create a duplicate.
