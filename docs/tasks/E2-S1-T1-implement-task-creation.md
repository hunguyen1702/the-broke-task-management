---
id: E2-S1-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S1-T1
  - E1-S3-T1
  - E3-S1-T1
---

# E2-S1-T1: Implement task creation

## Parent story

[E2-S1: Create a task](../epics/E2-S1-create-a-task.md)

## Objective

Implement atomic creation of typed tasks with stable IDs, validated scalar and structured context, status-code resolution, explicit actor attribution, and stable human/JSON output.

## Readiness

Planning is complete and E3-S1-T1 has established immutable status codes in the fresh baseline schema and lookup by code. This task is ready for implementation.

## Deliverables

- Versioned task-core migration following the existing repository and agent-registry migrations.
- Task creation input, domain model, validation, ID generation, and persistence in `tbtm-core`.
- Structured persistence for tags, external URLs, and code references.
- `tbtm task create` CLI parsing and rendering.
- Stable JSON result, typed errors, and shared exit-code mappings.
- Unit, migration, integration, transaction, and concurrency tests.

## Proposed structure

Extend the existing workspace without moving CLI behavior into core:

```text
crates/tbtm-core/
├── migrations/
│   └── <next>_task_core.sql
└── src/task/
    ├── model.rs
    ├── create.rs
    ├── id.rs
    └── validation.rs

crates/tbtm-cli/src/commands/
└── task_create.rs
```

Exact modules may follow the implemented layout. Core owns parsing-independent domain validation, repository mutation, and typed errors. CLI owns Clap arguments, `--code-ref` text parsing, output rendering, and exit mapping.

## Technical choices

- Resolve the canonical database through the E1-S5 shared resolver in read-write, no-create mode.
- Apply pending schema-version-1-compatible migrations before mutation. Keep config `schemaVersion: 1`; internal migrations remain separately ordered.
- Require the completed E3-S1 status-code baseline and reuse its transaction-friendly lookup. Resolve status by immutable code and persist its UUID foreign key.
- Use the E1-S3 agent UUID as the only explicit agent actor identifier.
- Generate eight-character UUID-v4-derived task suffixes and rely on separate database uniqueness constraints for the suffix and complete ID plus bounded retry.
- Store timestamps as RFC 3339 UTC and serialize JSON fields in camelCase.
- Preserve repeatable input order with explicit ordinals while enforcing per-task uniqueness.
- Keep the operation synchronous, local, and network-free.

## Data model

Add equivalent logical tables for:

- `tasks`: ID, repository-unique short suffix, current type, title, Markdown fields, status UUID, priority, optional estimate hours, archived flag, actor metadata, and timestamps.
- `task_tags`: task ID, ordinal, trimmed tag value.
- `task_external_urls`: task ID, ordinal, preserved validated URL.
- `task_code_references`: task ID, ordinal, repository-relative path, optional start/end lines, and optional description.

Use foreign keys with cascade cleanup for child rows even though permanent task deletion is not exposed in the MVP. Add check/unique constraints for bounded scalar fields and collection invariants where SQLite can express them reliably; retain domain validation for clear product errors.

Represent each actor with a tagged database form equivalent to `user` or `agent + UUID`. Agent actor rows reference the agent registry and must satisfy a constraint that user actors have no agent UUID while agent actors have one. Render this tagged storage as the JSON string `"user"` or the agent UUID.

Parent, dependency, claim, and comment tables are not part of this migration. The normalized shared full-task contract uses `hierarchy: {parent, children}`, `dependencies: {upstream, downstream}`, and `claim`; creation returns null or empty relationship values without persisting placeholder rows. E2-S2-T1 retrofits the already implemented create serializer from its original `parentId` and dependency-array shape.

## Implementation flow

### 1. Parse CLI input

Provide:

```text
tbtm task create \
  --title <title> \
  --type <type> \
  [--description <markdown>] \
  [--goal <markdown>] \
  [--acceptance-criteria <markdown>] \
  [--status <code>] \
  [--priority <integer>] \
  [--estimate <decimal-hours>] \
  [--tag <tag>]... \
  [--url <absolute-http-url>]... \
  [--code-ref <reference>]... \
  [--agent <uuid>] \
  [--json]
```

Use `to_do`, priority `50`, empty Markdown strings, no estimate, and empty collections when omitted. Parse fixed task-type machine tokens exactly as documented by the story.

For each `--code-ref`, split an optional `::description`, then recognize only a final numeric `:start` or `:start-end` suffix as a line range. This keeps ordinary colons in a path from being treated as line syntax unless they form the documented numeric suffix.

### 2. Validate without mutation

- Trim title and require non-empty content.
- Require priority in `0..=1_000_000`.
- Parse estimate as a finite non-negative decimal number of hours, including zero.
- Trim tags, reject empty tags, and reject exact duplicates after trimming.
- Parse external URLs with the selected URL library; require absolute `http` or `https`, preserve the accepted input value, and reject exact duplicates.
- Normalize code references into structured values. Require a non-empty lexical repository-relative path, reject absolute paths and `..` escape, use positive one-based lines, and require `end >= start`.
- Do not require a code-reference target to exist.
- Reject exact duplicate normalized code references.
- Parse `--agent` as UUID before opening the write transaction.

### 3. Resolve the repository

1. Resolve the shared canonical repository and open its existing database without creating a new one.
2. Apply any pending compatible mutation migrations.

Status and agent lookup intentionally wait for the creation transaction so resolution and insertion observe one SQLite snapshot.

### 4. Construct identity and metadata

1. Generate UUID v4 and take the first eight lowercase hex characters without hyphens.
2. Build `<stored-prefix>-<type-at-creation>-<suffix>`.
3. Capture one UTC timestamp for both creation and initial update metadata.
4. Build the complete task aggregate with `archived = false`, no claim, no parent, and no dependencies.

### 5. Resolve references and persist atomically

Within one transaction:

1. Resolve the requested status code, defaulting to `to_do`; do not resolve by display name or UUID.
2. Without `--agent`, select logical actor `user`. With it, query the exact registered UUID.
3. Return `STATUS_NOT_FOUND` or `AGENT_NOT_FOUND`, exit `3`, and roll back before insertion when lookup fails.
4. Insert the task row with the resolved status UUID, repository-unique suffix, and actor.
5. Insert tags, URLs, and code references in input order.
6. Commit only after every row succeeds, then render the status and actor values observed by that transaction.

On suffix or complete-ID uniqueness collision, roll back the attempt, generate a new UUID suffix, and retry up to a small documented constant. This includes suffix collision with a task of another type. Do not classify other constraints or database failures as collisions. Retry exhaustion and any child-row failure leave no task or context row.

### 6. Render output

Human output shows at least ID, title, type, status display name, priority, and actor.

JSON uses the shared envelope and this data shape:

```json
{
  "ok": true,
  "data": {
    "id": "project-task-a1b2c3d4",
    "title": "Implement parser",
    "description": "",
    "goal": "",
    "acceptanceCriteria": "",
    "type": "task",
    "status": {
      "id": "UUID",
      "code": "to_do",
      "name": "Todo",
      "completed": false
    },
    "priority": 50,
    "estimate": null,
    "tags": [],
    "externalUrls": [],
    "codeReferences": [],
    "hierarchy": {
      "parent": null,
      "children": []
    },
    "dependencies": {
      "upstream": [],
      "downstream": []
    },
    "claim": null,
    "archived": false,
    "createdAt": "RFC 3339 UTC timestamp",
    "updatedAt": "RFC 3339 UTC timestamp",
    "createdBy": "user",
    "updatedBy": "user"
  },
  "error": null
}
```

Each code reference serializes as `{path, startLine, endLine, description}`; absent optional scalar values are `null`. Preserve collection input order in the result.

### 7. Map errors and exit codes

| Exit | Conditions |
|---:|---|
| 0 | Task created |
| 1 | Database/operational failure or exhausted ID retries |
| 2 | Invalid title, type, priority, estimate, URL, code reference, or duplicate/empty collection value |
| 3 | Repository, agent, or status not found |
| 4 | Reserved for claim conflict; never returned by create |
| 5 | Permission denied |

Use stable typed codes such as `INVALID_TASK_TITLE`, `INVALID_TASK_TYPE`, `INVALID_PRIORITY`, `INVALID_ESTIMATE`, `INVALID_EXTERNAL_URL`, `INVALID_CODE_REFERENCE`, `DUPLICATE_TASK_CONTEXT`, `AGENT_NOT_FOUND`, and `STATUS_NOT_FOUND`. Do not expose SQL text or parser internals in product messages.

## Test plan

### Unit tests

- Title trimming/empty handling and all fixed task-type tokens.
- Priority boundaries and default.
- Finite non-negative decimal estimate parsing, including zero and rejected negative/NaN/infinite values.
- Tag trimming, empty values, case-sensitive exact duplicate detection, and stable order.
- Absolute HTTP/HTTPS URL validation, preserved values, unsupported schemes, and exact duplicates.
- Code-reference grammar, lexical path safety, optional fields, one-based line ranges, nonexistent targets, duplicates, and serialization.
- Task ID construction, `poc` token, repository-wide suffix uniqueness, cross-type collision retry, and retry exhaustion.
- Actor and complete task JSON serialization.
- Typed errors and shared exit-code mapping.

### Migration tests

- Apply the task-core migration to the fresh baseline with agent registry and status codes without changing config `schemaVersion`.
- Fresh initialization applies all compatible migrations and supports immediate task creation.
- Repository metadata, agents, status UUIDs/codes, and default ordering remain unchanged while the task-core migration is applied.
- Task and collection constraints reject invalid direct inserts without corrupting the database.
- Migration ledger ordering and transaction rollback remain valid.

### Integration tests

- Minimum create uses `to_do`, priority `50`, user actor, empty strings/collections, and null estimate.
- Full create persists every field and preserves collection order.
- Create as a registered agent; reject malformed and unknown UUIDs without mutation.
- Select each default status by code; reject display names, UUID selectors, and unknown codes.
- Create from main and linked worktrees and observe the same database.
- Force same-type and cross-type suffix collisions, then retry exhaustion, and verify no partial aggregate remains.
- Race creation with future status mutation/deletion behavior and verify lookup plus insert use one transaction snapshot.
- Inject failure into each child collection insert and verify complete rollback.
- Run concurrent creates and verify unique IDs and a usable database.
- Snapshot human and JSON success/error output and exact exit codes.

## Verification

Run the repository tasks:

```text
mise run format
mise run lint
mise run test
```

Smoke verification:

1. Initialize a temporary repository and register one agent.
2. Create a minimum task as `user` and a fully populated task as the agent.
3. Query SQLite for IDs, status foreign keys, actor values, timestamps, and all child rows.
4. Submit invalid and duplicate context and verify exit `2` with no new task.
5. Submit an unknown status/agent and verify exit `3` with no new task.

## Definition of done

- The parent story's functional and non-functional acceptance criteria pass.
- E3-S1 status codes are consumed by UUID-backed foreign key without duplicating status ownership.
- Minimum and full creation persist exactly one complete task aggregate.
- Task IDs remain collision-safe under controlled retries and concurrent local creation.
- All validation and lookup failures leave the database unchanged.
- Human and JSON output, typed errors, and exit codes are stable and tested.
- No parent, dependency, claim, comment, archive transition, or separate assignment behavior is introduced.
- Formatting, lint, and workspace tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E2-S1 epic](../epics/E2-S1-create-a-task.md)
- [E1-S1 repository initialization](../epics/E1-S1-initialize-repository.md)
- [E1-S3 agent registration](../epics/E1-S3-register-an-agent.md)
- [E3-S1 default-status story](../PRD.md#story-e3-s1-use-default-statuses)
- [E1-S5 shared worktree resolution](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [SQLite foreign keys](https://www.sqlite.org/foreignkeys.html)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
- [UUID crate](https://docs.rs/uuid/latest/uuid/)
