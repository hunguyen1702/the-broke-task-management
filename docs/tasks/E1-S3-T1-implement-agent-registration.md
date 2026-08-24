---
id: E1-S3-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S1-T1
  - E1-S2-T1
  - E1-S5-T1
implementation_commit: 015ffc8
---

# E1-S3-T1: Implement agent registration

## Parent story

[E1-S3: Register an agent](../epics/E1-S3-register-an-agent.md)

## Objective

Implement repository-local agent registration with atomic persistence, UUID-backed identity, deterministic display-name construction, and stable human/JSON CLI output.

## Deliverables

- Versioned SQLite migration for the agent registry.
- Agent registration domain model and service in `tbtm-core`.
- Shared base-name normalization and validation using the E1-S1 normalization contract.
- `tbtm agent register <base-name> [--json]` CLI handler.
- Stable result, error, and exit-code mappings.
- Unit, integration, and concurrency tests.

## Proposed structure

Extend the existing workspace without duplicating repository resolution or output infrastructure:

```text
crates/
├── tbtm-core/
│   ├── migrations/
│   │   └── 0002_agent_registry.sql
│   └── src/
│       └── agent/
│           ├── model.rs
│           ├── name.rs
│           └── register.rs
└── tbtm-cli/
    └── src/
        └── commands/agent_register.rs
```

Exact module names may follow the implemented E1-S1/E1-S2 layout. `tbtm-core` owns normalization, identity generation, persistence, and typed errors; `tbtm-cli` owns argument parsing, rendering, and process exit codes.

## Technical choices

- Reuse the E1-S2 shared resolver with read-write access that requires the database to exist.
- Add migration `0002_agent_registry.sql` through the existing migration mechanism.
- Treat config/database `schemaVersion: 1` as the repository compatibility version. Track ordered internal migrations independently in `schema_migrations`.
- Initialization applies all bundled migrations compatible with schema version 1. A mutating command may apply pending compatible migrations; read-only repository status never applies them.
- Use UUID v4 as the authoritative persisted agent ID.
- Use the existing UTC time library and serialize timestamps as RFC 3339 UTC.
- Enforce UUID and display-name uniqueness with database constraints, not only application checks.
- Keep registration synchronous and local; perform no network access.

## Data model

Add an `agents` table with equivalent logical fields:

```text
id            UUID text, primary key
base_name     normalized text, required
display_name  text, required, unique
created_at    RFC 3339 UTC timestamp, required
```

Multiple rows may share `base_name`. Do not add a unique constraint to that field. The migration must preserve repository metadata and existing status data.

Migration `0002` does not change config or repository metadata `schemaVersion`. Its successful application is recorded in `schema_migrations` using the existing ordered migration mechanism.

## Implementation flow

### 1. Parse and normalize input

- Require one positional `<base-name>`.
- Reuse the E1-S1 normalization function rather than maintaining a second transliteration algorithm.
- Normalize to lowercase ASCII with `[a-z0-9]` segments separated by single hyphens.
- Validate normalized length `1–48`; do not truncate.
- Return `INVALID_AGENT_NAME` before opening a write transaction when validation fails.

### 2. Resolve the repository

- Resolve from the current working location through E1-S2.
- Open the configured existing SQLite database in read-write, no-create mode.
- Preserve E1-S2 error codes for uninitialized repository, invalid configuration, unavailable database, and permission denial.

### 3. Apply compatible pending migrations

- After resolving the database for mutation, validate the existing migration ledger.
- Apply pending bundled migrations declared compatible with repository `schemaVersion: 1`, including `0002_agent_registry.sql`, in order.
- Record each successful migration in `schema_migrations` through the existing migration transaction mechanism.
- Do not rewrite `.tbtm/config.json` or repository metadata merely to add the agent table.
- A migration failure is an operational database failure and agent insertion does not begin.
- `tbtm repo status` may validate and report migration state but must not apply pending migrations.

### 4. Register atomically

Within one registration operation:

1. Generate UUID v4.
2. Remove hyphens from its canonical lowercase form and take the first eight hexadecimal characters as the suffix.
3. Construct `<normalized-base-name>-<suffix>`.
4. Capture one RFC 3339 UTC `createdAt` value.
5. Insert the complete agent record in a SQLite transaction.
6. On a display-name uniqueness collision, roll back that attempt, generate a new UUID, reconstruct the display name, and retry up to a small documented implementation constant.
7. On success, commit and return the persisted values.

A UUID uniqueness collision follows the same regeneration path. Other constraint or database errors are not treated as collisions. Exhausting the retry bound returns a typed operational failure without a partial record.

Migration application commits before the registration transaction. Therefore a successful compatible migration may remain applied when the later agent insert fails; atomicity applies to the agent record, not to the entire upgrade-plus-command sequence.

### 5. Render output

Human success output shows:

- Agent UUID.
- Normalized base name.
- Unique display name.
- Creation timestamp.

JSON uses the shared envelope and camelCase fields:

```json
{
  "ok": true,
  "data": {
    "id": "UUID",
    "baseName": "claude",
    "displayName": "claude-a1b2c3d4",
    "createdAt": "RFC 3339 UTC timestamp"
  },
  "error": null
}
```

Write only the JSON document to stdout in JSON mode. Diagnostic text belongs in the shared error envelope rather than mixed human text.

### 6. Map errors and exit codes

| Exit | Condition |
|---:|---|
| 0 | Agent registered |
| 1 | Database/operational or unexpected retry-exhaustion failure |
| 2 | `INVALID_AGENT_NAME` or inherited invalid configuration |
| 3 | Repository not initialized |
| 5 | Permission denied |

Error details for `INVALID_AGENT_NAME` include the accepted normalized length and the invalid normalized result where safe. Do not expose internal SQL text as the product error contract.

### 7. Preserve explicit identity semantics

- Return the UUID so callers can retain it.
- Establish `--agent <uuid>` as the input contract for future agent-scoped CLI handlers.
- Do not write agent selection into repository config, environment files, or machine-global state.
- Do not resolve an actor from display name or base name.

## Test plan

### Unit tests

- Shared normalization for ASCII, whitespace/separators, Vietnamese composed/decomposed input, empty output, and `1–48` boundaries.
- UUID suffix extraction uses exactly eight lowercase hexadecimal characters.
- Display-name construction and maximum resulting length.
- Agent result serialization and human rendering.
- Typed error and exit-code mapping.

### Migration tests

- Fresh E1-S1 database migrates to the agent registry schema.
- A fresh initialization applies all bundled schema-version-1 migrations, including `0002`.
- An older schema-version-1 database with only `0001` applies `0002` during registration without changing config or repository metadata version.
- Existing repository metadata and statuses remain unchanged.
- UUID and display name reject duplicates while base name permits duplicates.
- Migration is transactional, records internal migration `0002`, and leaves compatibility `schemaVersion` at `1`.

### Integration tests

- Register from repository root and nested directory.
- Register the same input twice and verify two distinct UUIDs/display names and two stored rows.
- Verify input normalization and stored normalized base name.
- Invalid input writes no record.
- Force a first-attempt UUID/display collision and verify regeneration succeeds without modifying the existing row.
- Force retry exhaustion and verify no partial record remains.
- Preserve inherited E1-S2 error codes for repository/config/database/permission failures.
- Snapshot human and JSON success/error output and assert exact exit codes.
- Run simultaneous registrations and verify all successful UUIDs/display names are unique and the database remains usable.

## Verification

Provide or reuse `mise` tasks for:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Smoke verification:

1. Initialize a temporary repository.
2. Register `Claude Agent` twice with JSON output.
3. Confirm both responses store `baseName: "claude-agent"` but return different IDs and display names.
4. Query SQLite and confirm both complete records exist with unique constraints active.
5. Try an input that normalizes to empty and confirm `INVALID_AGENT_NAME`, exit code 2, and no additional row.

## Definition of done

- Parent story functional and non-functional acceptance criteria pass.
- Migration, registration service, CLI command, and stable output are implemented.
- Every successful registration creates exactly one new identity.
- Duplicate base names remain allowed; UUID/display-name duplicates do not.
- Failed validation, collision retries, or database writes leave no partial record.
- Repository/config `schemaVersion` remains `1`; internal migration `0002` is tracked separately.
- No implicit current-agent selection is persisted.
- Formatting, lint, and tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E1-S1 epic](../epics/E1-S1-initialize-repository.md)
- [E1-S1 implementation task](E1-S1-T1-implement-repository-initialization.md)
- [E1-S2 epic](../epics/E1-S2-resolve-repository-configuration.md)
- [E1-S2 implementation task](E1-S2-T1-implement-repository-configuration-resolution.md)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [SQLite uniqueness constraints](https://www.sqlite.org/lang_createtable.html#uniqueconst)
- [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
- [UUID crate](https://docs.rs/uuid/latest/uuid/)
