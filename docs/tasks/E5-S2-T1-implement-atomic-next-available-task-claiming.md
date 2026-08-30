---
id: E5-S2-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E4-S3-T1
  - E5-S1-T1
implements:
  - E5-S2
---

# E5-S2-T1: Implement atomic next-available-task claiming

## Epic

[E5-S2: Claim the next available task atomically](../epics/E5-S2-claim-the-next-available-task-atomically.md)

## Objective

Implement an agent-only command that selects the first filtered available task and creates its active claim in one immediate SQLite transaction, returning either the claimed full task or a stable successful empty result.

## Readiness

Planning is complete. E4-S3-T1 provides the caller-owned authoritative availability selector, filters, and ordering. E5-S1-T1 provides the claim table, registered-agent validation, full-task hydration, transaction conventions, and success renderer. Both dependencies are implemented, so this task is ready.

## Deliverables

- A core claim-next input and optional full-task result.
- Transaction-scoped reuse of E4-S3 selection and E5-S1 claim insertion/hydration behavior.
- `tbtm task claim-next` parsing, exact success/empty rendering, JSON envelopes, and exit mapping.
- Core and CLI tests for validation, filters, availability, ordering, rollback, concurrency, busy timeout, linked worktrees, outputs, and metadata preservation.

## Proposed structure

- Extend `crates/tbtm-core/src/task.rs` or its established availability/claim submodules. Keep one shared selector callable with a borrowed transaction and one shared claim insertion/full-task hydration path.
- Add `ClaimNext` arguments and dispatch in `crates/tbtm-cli/src/main.rs`, reusing E4-S3 repeatable filter parsing and E5-S1 claim-success rendering.
- Add focused core tests beside task behavior and process/concurrency coverage under `crates/tbtm-cli/tests/`.
- No migration or new persisted availability field is expected.

## Technical choices

- Define input containing required agent UUID plus repeatable status codes, task types, and tags. Return `Result<Option<FullTask>, ...>` or an equivalent typed optional result that distinguishes empty success from failure.
- Reuse the E4-S3 selector inside a caller-owned immediate transaction. Add a limit-one path if useful, but do not fork its predicate, filter validation, or ordering semantics.
- Reuse the E5-S1 claim row, registered-agent lookup, UTC timestamp format, full-task loader, canonical repository resolver, busy timeout, and exact human success renderer.
- Retain `PRAGMA journal_mode = DELETE` and the existing `synchronous = FULL` behavior. Do not introduce WAL files or change repository lifecycle in this task.
- Keep the transaction short: after `BEGIN IMMEDIATE`, perform only database validation, selection, timestamp generation, insertion, aggregate hydration, and commit. Do not prompt or perform network/filesystem work unrelated to SQLite state while holding the lock.
- Do not update the selected task row or introduce retry/fallback behavior. A unique-key violation is an integrity/operational failure, not a reason to silently select another candidate.

## Implementation flow

### 1. Add the core request and reuse boundary

Introduce a claim-next input for agent UUID and E4-S3 filters. Preserve this validation order:

1. CLI syntax, UUID parsing, and task-type parsing.
2. Registered-agent lookup.
3. Validation of every distinct status code.
4. Candidate selection.

The transaction-scoped selector applies the complete availability predicate, supplied filters, and `priority DESC, created_at ASC, id ASC`, returning at most the first candidate. A missing candidate returns `None`, not an availability or conflict error.

### 2. Select and claim in one immediate transaction

1. Resolve the E1-S5 canonical repository read-write and apply compatible pending migrations.
2. Start `TransactionBehavior::Immediate`, using the existing busy timeout.
3. Resolve the registered agent or return `AGENT_NOT_FOUND`.
4. Validate statuses and select the first available candidate through the shared E4-S3 primitive.
5. If absent, finish without writes and return empty success.
6. Generate one UTC claim timestamp and insert the E5-S1 active-claim row.
7. Hydrate the selected normalized `FullTask`, including claim and direct relationships, from the same transaction.
8. Commit and return the full task.

Selection must not happen before the immediate transaction. Every error, insertion failure, hydration failure, or failed commit leaves no partial claim. Task status, archive fields, content, hierarchy, dependencies, `updated_at`, and actor metadata remain unchanged.

### 3. Preserve serialized concurrency semantics

SQLite permits one writer at a time for the shared database. A contender waits up to the configured busy timeout, then reruns selection only after acquiring the lock:

- with at least two candidates, successive transactions select distinct tasks in current ordering;
- with one candidate, the winner claims it and the waiter observes no candidate and succeeds empty;
- timeout before acquiring the lock remains an operational database error;
- the command never emits `CLAIM_CONFLICT` or exit `4`.

Keep `journal_mode = DELETE`; WAL is not needed for correctness and does not add simultaneous writers. Test across separate connections, processes, and linked worktrees resolving the same canonical database.

### 4. Add CLI parsing and output

Expose:

```text
tbtm task claim-next --agent <uuid> [--status <code>]... [--type <type>]... [--tag <tag>]... [--json]
```

For a claimed task, reuse E5-S1's renderer exactly:

```text
Claimed task: <id>
Agent: <display-name>
Claimed at: <timestamp>
```

JSON returns the shared success envelope with the normalized full task. For no candidate, human output is exactly `No available task to claim.` and JSON uses the shared success envelope with `data: null`.

Map outcomes:

| Exit | Outcome |
|---:|---|
| 0 | Claimed task or empty success |
| 1 | Database, busy-timeout, or unexpected operational failure |
| 2 | Malformed/missing arguments, malformed UUID, or `INVALID_TASK_TYPE` |
| 3 | `AGENT_NOT_FOUND` or `STATUS_NOT_FOUND` |
| 5 | Inherited permission failure |

Exit `4` is never produced. Failures use the shared error envelope and never return partial task data.

## Test plan

### Selection, validation, and state

- Select a zero-dependency active, incomplete, unclaimed task; independently exclude archived, completed, claimed, and unresolved-dependency tasks.
- Cover completed-status, archived, mixed-resolved, and unresolved upstream combinations.
- Exercise status/type/tag filters independently, repeated OR values, cross-group AND, duplicates, tag case, completed-status empty success, invalid type, and unknown status.
- Prove validation precedence for malformed UUID/type, missing agent plus unknown status, and valid agent plus unknown status.
- Cover priority, creation-time, and task-ID tie-breaks, including candidates excluded by filters or availability.
- Assert the stored claim agent/timestamp and full-task claim shape, plus unchanged task status, archive state, relationships, timestamps, and actors.
- Verify empty repositories and empty-after-filter return success without claim rows or task writes.

### Atomicity and concurrency

- Inject claim insertion and post-insert full-task hydration failures and verify rollback leaves no active claim.
- Race two contenders with at least two candidates and assert two successful distinct claims following serialized current ordering.
- Race two contenders with one candidate and assert one claimed result plus one empty success, never `CLAIM_CONFLICT`.
- Repeat both races through separate linked worktrees and verify one canonical database, unique claim rows, and database integrity.
- Hold the write lock beyond the busy timeout and verify an operational error, then verify no fabricated conflict or partial state.
- Protect the no-fallback rule by forcing an insertion/integrity failure and asserting the command does not claim the next candidate.

### CLI and regression coverage

- Snapshot exact claimed and empty human output, JSON full-task and `data: null` success, validation errors, and exits.
- Re-run E4-S3 available-query cases to prove shared predicate/filter/ordering behavior remains identical.
- Re-run E5-S1 specified-claim, task view/list, unclaim, archive, hierarchy, and dependency regressions relevant to shared claim hydration.
- Verify no journal-mode change or persistent WAL/SHM artifacts are introduced.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Also smoke-test two registered agents running `claim-next` concurrently from separate linked worktrees with two candidates and then one candidate.

## Definition of done

- Every functional and non-functional criterion in E5-S2 passes.
- The exact E4-S3 selector chooses the first current candidate only inside the immediate transaction.
- A claim or empty result is one atomic outcome with complete rollback on failure and no task mutation.
- Concurrent one- and multi-candidate behavior, linked-worktree sharing, busy timeout, and claim uniqueness are verified.
- Human/JSON output, validation precedence, stable errors, and exits match the contract.
- Journal mode remains `DELETE`; the locked section contains no prompts or external work.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E5-S2 epic](../epics/E5-S2-claim-the-next-available-task-atomically.md)
- [PRD availability definition](../PRD.md#81-available-task-definition)
- [PRD default ordering](../PRD.md#82-default-ordering)
- [PRD query versus claim](../PRD.md#83-query-versus-claim)
- [PRD claim management](../PRD.md#fr-8-claim-management)
- [PRD agent registration and work acquisition](../PRD.md#101-agent-registration-and-work-acquisition)
- [PRD E5-S2 story](../PRD.md#story-e5-s2-claim-the-next-available-task-atomically)
- [E4-S3 availability contract](../epics/E4-S3-query-available-tasks.md)
- [E4-S3 implementation selector](E4-S3-T1-implement-available-task-querying.md)
- [E5-S1 specified-claim contract](../epics/E5-S1-claim-a-specified-task-atomically.md)
- [E5-S1 implementation task](E5-S1-T1-implement-atomic-specified-task-claiming.md)
- [E1-S5 shared worktree state](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [E2-S2 full-task contract](../epics/E2-S2-view-and-list-tasks.md)
