---
id: E1-S1-T2
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S1-T1
implements:
  - E1-S1
acceptance_failure:
  scenario: AT-E1-S1-001
  run: 2026-09-04-f02774b
  classification: scenario_drift
---

# E1-S1-T2: Align repeat-init acceptance exit

## Parent story

[E1-S1: Initialize a repository](../epics/E1-S1-initialize-repository.md)

## Failure evidence

The 2026-09-04 acceptance run repeated `tbtm init --prefix acc --json` in an
initialized repository. The CLI returned `ALREADY_INITIALIZED` with exit `2`,
while AT-E1-S1-001 expected exit `4`.

This is scenario drift, not an implementation defect. E1-S1 defines exit `2`
for validation and repository-state errors and reserves exit `4` for claim
conflicts introduced by E5.

See the [acceptance run summary](../testing/runs/2026-09-04-f02774b/summary.md#at-e1-s1-001--initialize-twice).

## Objective

Restore consistency between the approved E1-S1 contract and its acceptance
scenario without changing public CLI behavior.

## Deliverables

- Change AT-E1-S1-001 “Initialize twice” to expect exit `2` while retaining
  the `ALREADY_INITIALIZED` assertion.
- Revalidate the full E1-S1 scenario against the epic and current CLI help.
- Obtain the required scenario approval before executing the updated case.
- Execute the re-approved E1-S1 scenario in an isolated temporary repository
  and update the acceptance result and run history.

## Verification

- The updated scenario agrees with E1-S1's exit-code table.
- Re-execution observes exit `2` and `ALREADY_INITIALIZED`.
- No source code or unrelated scenario is changed.

## Acceptance impact

`revalidate`: AT-E1-S1-001 must be updated, approved, and rerun.

## Completion

- AT-E1-S1-001 now expects exit `2` with `ALREADY_INITIALIZED` for repeat
  initialization, matching the E1-S1 contract and current CLI help.
- The user approved the corrected expectation on 2026-09-05.
- The approved scenario was rerun in isolated temporary repositories: 5 passed,
  0 failed, 0 blocked.
- Evidence: [2026-09-05 targeted acceptance run](../testing/runs/2026-09-05-cda9671/summary.md).
