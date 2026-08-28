---
id: E2-S5-T1
kind: implementation_task
planning_status: done
implementation_status: blocked
depends_on:
  - E2-S1-T1
  - E2-S2-T1
  - E4-S3-T1
  - E5-S3-T1
---

# E2-S5-T1: Implement safe task archival

## Parent story

[E2-S5: Archive a task](../epics/E2-S5-archive-a-task.md)

## Objective

Implement atomic, actor-aware task archival with a required durable reason, safe handling of active claims, immediate effective-completion semantics, and stable human/JSON behavior.

## Deliverables

- Schema and shared full-detail model support for nullable `archiveReason`, including E2-S1 create and E2-S2 view regression coverage.
- A typed core archive operation covering actor lookup, archive idempotency, claim guards, conditional force release, metadata, and aggregate reload.
- `tbtm task archive` parsing, confirmation, rendering, stable errors, and exit mappings.
- Core, CLI, concurrency, migration, availability, and relationship-preservation tests.

## Proposed structure

Extend the authoritative task, claim, availability, and CLI modules present after the dependencies are implemented. A likely shape is:

```text
crates/tbtm-core/src/task/archive.rs
crates/tbtm-core/src/task/model.rs
crates/tbtm-cli/src/commands/task_archive.rs
```

Exact files should follow the repository layout at implementation time. Core owns normalization, authorization rules, transactional claim comparison/release, lifecycle mutation, and typed results. CLI owns argument parsing, TTY detection, confirmation, human rendering, JSON envelopes, and exit mapping.

## Technical choices

- Add nullable archive-reason storage through the migration strategy current at implementation time; expose it as `archiveReason` in every shared full-task JSON and human detail result. Create returns `null`, view returns `null|string`, and archive returns the stored string. Keep it out of compact list rows.
- Enforce the domain invariant for product-managed state: active tasks have `archiveReason = null`, while archived tasks have a normalized non-empty reason. Apply the repository's release-stage migration policy so pre-feature development data cannot bypass that invariant; do not fabricate user-authored reasons silently.
- Reuse E2-S2's full-task loader and renderer, E1-S3 actor resolution, E4-S3 effective-completion/availability logic, and E5's authoritative active-claim model. Do not create parallel claim or availability representations.
- Normalize the reason by trimming outer whitespace and require non-empty content. Preserve accepted inner Markdown exactly.
- Represent force approval with an observed-claim token containing agent ID and `claimedAt`. Core must conditionally compare that token before deleting a foreign claim.
- Use one SQLite write transaction for task and actor resolution, archive-state/idempotency checks, current-claim read, observed-claim comparison, optional claim deletion, task mutation, and aggregate reload.
- Availability remains derived from `archived`, status completion, claim absence, and upstream effective completion. Archive writes no availability cache.

## CLI and confirmation contract

Expose:

```text
tbtm task archive <id> --reason <markdown> [--agent <uuid>] [--force] [--yes] [--json]
```

- Without `--agent`, the actor is logical `user`; with it, resolve an exact registered agent UUID.
- `--yes` without `--force` returns `CONFLICTING_ARGUMENTS` before mutation.
- Unclaimed and own-claimed tasks require no prompt. If the task is foreign-claimed, normal archive returns `TASK_CLAIMED` with owner details.
- Only logical `user` may use `--force`. In an interactive human terminal, show claimant ID/display name, `claimedAt`, task ID, and reason, then request confirmation unless `--yes` is present.
- Declining prints `Archive cancelled.` in human mode, exits `0`, and returns without mutation. It does not use the full-task archive-success renderer. Non-interactive or JSON force execution requires `--yes`; otherwise return `CONFIRMATION_REQUIRED`, so JSON has no prompt or cancellation branch.
- The CLI passes the exact observed claim identity/time into core. It must not authorize releasing whichever claim happens to exist later.

## Implementation flow

1. Resolve the canonical writable repository and apply compatible pending migrations.
2. Parse options, reject invalid flag combinations, resolve execution mode, and normalize/validate the required reason.
3. For a possible force operation, read enough current task/claim state to render the warning. If confirmation is declined, render the distinct human cancellation result and return before opening the mutation transaction.
4. Begin the authoritative write transaction and load the exact task and actor.
5. If already archived, compare normalized reasons: return a write-free full-detail no-op for equality or `TASK_ARCHIVED` for a difference.
6. Read the current claim. Permit no claim or the invoking agent's own claim. Reject a foreign claim unless the actor is logical `user`, force was explicit, and the observed claim token matches.
7. If the previously observed claim disappeared, continue. If another foreign claim replaced it, return `TASK_CLAIMED` with the new claimant and make no change.
8. Delete any permitted active claim, set archive flag/reason and one `updated_at`/`updated_by` pair, then reload full detail in the same transaction.
9. Commit before rendering. Effective completion and downstream availability queries must immediately reflect the committed archive flag.

All lookup, validation, authorization, concurrency, or persistence failures roll back claim removal, archive state, reason, and actor metadata together.

## Output and exit contract

Success uses the E2-S2 full-task detail shape and shared envelope. It includes `archived: true`, normalized `archiveReason`, `claim: null`, preserved status/content/relationships, and current metadata. Human success uses the shared detail renderer where practical.

The schema/model retrofit also makes E2-S1 create return `archiveReason: null` and E2-S2 view return `null` or the stored reason. Compact list output does not add the field. A declined TTY confirmation prints only the human cancellation result `Archive cancelled.`, exits `0`, and must not be serialized as archive success.

`TASK_CLAIMED` human and JSON failures include claimant `{id, displayName}` and `claimedAt`. Confirmation output must never be emitted into JSON stdout.

| Condition | Code/category | Exit |
|---|---|---:|
| Required reason omitted or syntactically missing | parser/shared validation | 2 |
| Reason blank after trimming | `INVALID_ARCHIVE_REASON` | 2 |
| `--yes` without `--force` | `CONFLICTING_ARGUMENTS` | 2 |
| Non-interactive/JSON force lacks `--yes` | `CONFIRMATION_REQUIRED` | 2 |
| Already archived with different reason | `TASK_ARCHIVED` | 2 |
| Task missing | `TASK_NOT_FOUND` | 3 |
| Agent missing | `AGENT_NOT_FOUND` | 3 |
| Foreign claim blocks or changed after confirmation | `TASK_CLAIMED` | 4 |
| Agent attempts `--force` | `PERMISSION_DENIED` | 5 |

An accepted archive, same-reason no-op, or declined interactive confirmation exits `0`. Shared repository/configuration/database/permission/unexpected failures retain existing codes and exits.

## Test plan

### Core tests

- Archive unclaimed and own-claimed tasks as logical user and registered agents.
- Verify archive flag, normalized reason, one actor/timestamp pair, claim deletion, preserved status/content/identity/creation metadata, and unchanged hierarchy/dependency rows.
- Verify foreign-claim rejection and logical-user force approval; reject agent force.
- Cover observed claim unchanged, removed, and replaced before the write transaction. Prove a replacement claim is never deleted.
- Verify same-reason archived no-op issues no writes and preserves metadata; different reason returns `TASK_ARCHIVED` without mutation.
- Cover missing task/agent, blank reason, constraint failure, and rollback after claim deletion is attempted.
- Verify E4-S3 availability changes for downstream tasks and the archived task immediately after commit.
- Exercise concurrent archive, claim, unclaim, and force operations through linked worktrees sharing the canonical store.

### CLI integration tests

- Cover required/blank reasons and every `--force`/`--yes` combination.
- Exercise TTY confirmation accept and decline, non-interactive input, and JSON execution without leaking prompt text into JSON stdout.
- Snapshot full-detail human/JSON success, `TASK_CLAIMED` details, stable errors, and exact exits.
- Regression-test create with `archiveReason: null`, active and archived view with `null|string`, archived detail/list/graph visibility, and compact-list omission of the field.
- Compare archive output with a subsequent `task view` and inspect database state after every rejected path.

## Verification

Run:

```text
mise run format
mise run lint
mise run test
```

Smoke-test a temporary initialized repository with an incomplete upstream task, downstream tasks, and claims owned by multiple agents. Exercise normal and forced archive from the main and a linked worktree, inspect SQLite state and actor metadata, and confirm downstream availability and archived views.

## Definition of done

- Every E2-S5 functional and non-functional acceptance criterion passes.
- Archive reason storage plus E2-S1 create and E2-S2 view full-detail output are migrated and regression-tested without changing compact lists.
- A successful archive and permitted claim release are one atomic mutation; no rejected or raced operation partially changes state.
- Foreign claims are never silently interrupted, and force confirmation applies only to the claimant shown to the user.
- Effective completion, availability, and relationship views consume the authoritative committed archive state.
- Human/JSON output, confirmation behavior, stable codes, and exits are deterministic and tested.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E2-S5 epic](../epics/E2-S5-archive-a-task.md)
- [E2-S1 create-task contract](../epics/E2-S1-create-a-task.md)
- [E2-S2 task-detail contract](../epics/E2-S2-view-and-list-tasks.md)
- [E4-S3 available-task story](../PRD.md#story-e4-s3-query-available-tasks)
- [E5-S3 unclaim story](../PRD.md#story-e5-s3-unclaim-owned-work)
- [E5-S4 force-unclaim story](../PRD.md#story-e5-s4-force-unclaim-stale-work)
- [E6 comment model](../PRD.md#710-comment)
- [E1-S1 confirmation contract](../epics/E1-S1-initialize-repository.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
