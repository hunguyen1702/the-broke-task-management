---
id: E2-S4
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E2-S1
---

# E2-S4: Manage tags, URLs, estimates, and code references

## Outcome

Users and registered agents can atomically replace or clear the structured implementation context of an active task while preserving stable identity and deterministic output.

## User story

As a coding agent, I want structured implementation context so that I can locate relevant code and understand effort.

## Command

E2-S4 extends the shared partial-update command:

```text
tbtm task update <id> \
  [--estimate <hours> | --clear-estimate] \
  [--tag <tag>]... [--clear-tags] \
  [--url <url>]... [--clear-urls] \
  [--code-ref <reference>]... [--clear-code-refs] \
  [--agent <uuid>] [--json]
```

E2-S4 may establish this command with structured fields before E2-S3 is implemented. E2-S3 later adds scalar fields to the same patch operation; it does not replace this contract.

## Product decisions

### Partial replacement semantics

- Omitted estimate or collection fields remain unchanged.
- Supplying one or more `--tag`, `--url`, or `--code-ref` values replaces that complete collection; the options do not append to the persisted collection.
- `--clear-tags`, `--clear-urls`, and `--clear-code-refs` replace the corresponding collection with an empty collection.
- `--clear-estimate` changes estimate to `null`; `--estimate 0` is a distinct valid value.
- A value option and its matching clear flag are mutually exclusive. A command with no mutable E2-S3 or E2-S4 field is invalid.
- When scalar fields from E2-S3 and structured fields from E2-S4 are eventually supplied together, they form one validated, atomic patch with one actor and timestamp.

### Validation and ordering

- Estimate is a finite non-negative decimal number of hours.
- Tags are trimmed and must remain non-empty. Duplicate detection occurs after trimming and remains exact and case-sensitive.
- External URLs must be absolute `http` or `https` URLs. Accepted input is preserved and exact duplicates are rejected.
- Code references reuse `path[:start[-end]][::description]`. Paths are non-empty lexical repository-relative paths, cannot be absolute or escape the repository, and need not exist. Lines are positive and one-based; an end line cannot precede its start.
- Duplicate normalized code references are rejected.
- Replacement collections preserve CLI input order, and detail output preserves their stored ordinal order.
- Empty or duplicate values fail the complete patch rather than being silently discarded.

### Mutation, actors, and no-ops

- Only active tasks can be updated. An archived task returns `TASK_ARCHIVED` without mutation.
- Without `--agent`, the actor is logical `user`. An explicit agent UUID must resolve to a registered repository agent.
- Task lookup, actor resolution, validation, effective-change detection, scalar update, collection replacement, metadata update, and aggregate reload occur in one SQLite write transaction.
- An effective patch writes one RFC 3339 UTC `updatedAt` and its resolved actor to `updatedBy`.
- A fully valid patch whose normalized estimate and collections equal persisted values succeeds without writing rows or changing update metadata.
- Task ID, type, scalar content not included in the patch, relationships, claim, archive state, and creation metadata remain unchanged.

### Output and errors

- Success returns the complete persisted task using the E2-S2 normalized detail contract in the shared JSON envelope or human detail form.
- Invalid estimate, malformed or empty URL, and malformed or empty code reference return `INVALID_ESTIMATE`, `INVALID_EXTERNAL_URL`, and `INVALID_CODE_REFERENCE` respectively. An empty or duplicate tag, duplicate URL, or duplicate normalized code reference returns `DUPLICATE_TASK_CONTEXT`; conflicting clear/value options return `CONFLICTING_ARGUMENTS`; absence of update fields returns `NO_UPDATE_FIELDS`. Each is a validation failure with exit code `2`.
- A missing task or unknown agent UUID returns `TASK_NOT_FOUND` or `AGENT_NOT_FOUND` with exit code `3`.
- Exit code `4` remains reserved for claim conflicts. Shared repository, database, permission, and unexpected error categories remain unchanged.

## Functional acceptance criteria

1. A user or registered agent can replace any supported structured collection, set or clear estimate, and combine independent structured fields in one partial patch.
2. Omitted fields remain unchanged; explicit clear flags produce `null` or empty collections and conflict with value options for the same field.
3. Estimate, tags, URLs, and code references reuse the creation-time validation rules, including trimmed/exact duplicate handling and valid line ranges.
4. Replacement collections preserve supplied order in persistence and full-detail output.
5. The complete patch either commits with one actor/timestamp pair or leaves the task aggregate unchanged.
6. A fully validated identical replacement succeeds without database writes or metadata changes.
7. Archived tasks, missing tasks, unknown agents, and invalid inputs fail with the documented stable codes and no mutation.
8. Success output matches `task view`; structured and E2-S3 scalar fields can coexist in one atomic update contract.

## Non-functional acceptance criteria

1. Structured validation, replacement, metadata, and returned aggregate remain transactionally consistent under concurrent local processes and linked worktrees.
2. The command performs no network or target-file existence checks and feels immediate for a local personal repository.
3. Parameterized SQLite operations and typed core patch values keep business rules outside the CLI.
4. Collection order is deterministic, timestamps use RFC 3339 UTC, and JSON fields use camelCase.
5. Tests cover practical Linux, macOS, and Windows parsing and path behavior.

## Verification

- Exercise set, replace, clear, omit, conflict, duplicate, invalid, effective-update, and valid-no-op paths for every structured field.
- Combine multiple structured fields and force a late validation or persistence failure to prove complete rollback.
- Inspect SQLite ordinals and metadata, and compare human/JSON success output with `task view`.
- Update from the main worktree and a linked worktree against the same canonical store, including competing write transactions.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Scalar title, Markdown, type, status, and priority semantics; E2-S3.
- Append/remove-one collection operations or automatic duplicate removal.
- Parent, dependency, claim, comment, archive, or unarchive mutation.
- Network validation, checking whether code-reference targets exist, permanent deletion, or interactive editing.

## Implementation task

See [E2-S4-T1: Implement structured task-context updates](../tasks/E2-S4-T1-implement-structured-task-context-updates.md).
