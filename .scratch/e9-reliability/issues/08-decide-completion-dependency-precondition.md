# Decide whether status completion requires satisfied dependencies

Type: grilling
Status: open
Parent: [E9 reliability and guidance implementation route](../map.md)
Blocked by: none

## Question

Should marking a task with a completed status require every direct upstream dependency to be effectively completed?

## Context

- An upstream task is effectively completed when it is archived or has a completed status. Claim ownership does not affect effective completion.
- Current contracts require satisfied direct dependencies for availability and claim acquisition, but do not impose that precondition on task status updates. See [task updates](../../../docs/epics/E2-S3-update-task-content.md) and [claim/status independence](../../../docs/epics/E5-S5-preserve-claim-status-independence.md).
- Example: C depends on B, and B depends on A. With all three incomplete and unclaimed, A is available first. The CLI nevertheless permits marking B with a completed status while A remains incomplete. B then satisfies C's direct dependency.
- Requiring satisfied dependencies before marking B completed would change existing product behavior rather than repair a violation of the current contracts.

## Decisions to settle

- Retain the current behavior or require every direct upstream to be effectively completed before a task status update can mark it completed.
- If a precondition is selected, decide how task creation with a completed status, repository-wide status completion changes, archive, and later upstream reopening relate to the rule. These operations can also produce effective completion or change dependency satisfaction.
- Define any rejection code, transaction boundary, compatibility impact, and authoritative contract changes before implementation.

## Resolution criterion

Agree on the rule and its operation boundaries. Record the decision and, if behavior changes, prepare a separately approved contract and implementation plan.

## Comments

- 2026-10-07: user requested a separate unresolved decision ticket. This ticket does not block E9-S2 planning or verification. E9-S2 continues under the current contracts; no new completion precondition is approved here.
