# Acceptance run reports

When a run needs to be retained, create
`docs/testing/runs/<date>-<commit>/summary.md` containing:

- code revision and operating system;
- scenarios run;
- passed, failed, and blocked cases;
- actual command and output for failures;
- cleanup result.

Do not store temporary repositories, databases, binary output, hashes, or
successful command transcripts unless needed to diagnose a failure.
