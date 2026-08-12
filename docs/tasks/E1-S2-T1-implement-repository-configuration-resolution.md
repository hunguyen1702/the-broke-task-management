# E1-S2-T1: Implement repository configuration resolution

## Status

Ready for implementation.

## Parent story

[E1-S2: Resolve repository configuration](../epics/E1-S2-resolve-repository-configuration.md)

## Objective

Implement a shared, safe repository resolver and the read-only `tbtm repo status` command. Ordinary repository operations should do only the work required to locate and open the configured database; the status command should perform full config/database health validation.

## Deliverables

- Shared repository resolver in `tbtm-core`.
- Typed schema-version-1 config parsing and validation.
- Existing-database opening with explicit read-only/read-write access intent and no implicit creation.
- Full repository health inspection service.
- `tbtm repo status [--json]` CLI handler.
- Stable human/JSON output, errors, and exit-code mapping.
- Unit and integration tests.

## Proposed structure

Extend the E1-S1 workspace without duplicating its root-discovery or config types:

```text
crates/
├── tbtm-core/
│   └── src/
│       └── repository/
│           ├── config.rs
│           ├── discovery.rs
│           ├── resolver.rs
│           └── health.rs
└── tbtm-cli/
    └── src/
        └── commands/repo_status.rs
```

Exact module boundaries may follow the E1-S1 implementation, but resolution, full health inspection, and CLI rendering must remain separate responsibilities.

## Technical choices

- Reuse E1-S1 repository-root discovery and persisted config model.
- Use typed errors with stable product error codes; do not classify errors from rendered message text.
- Open SQLite with flags that require the file to exist. The resolver receives an access intent such as read-only or read-write; it never creates a missing DB.
- Use a read-only connection for `repo status` and execute `PRAGMA quick_check` after metadata validation.
- Keep implementation synchronous and perform no network access.

## Implementation flow

### 1. Resolve repository root

- Use the nearest Git worktree root when available.
- Outside Git, use the canonical current directory.
- Resolve direct child paths `.tbtm/config.json` and `.tbtm/tbtm.db` from the root-owned path type.
- Treat an absent `.tbtm` as `REPOSITORY_NOT_INITIALIZED`.

### 2. Parse and validate config

Deserialize `.tbtm/config.json` into a version-aware config type. For schema version 1, validate:

- `schemaVersion` equals `1`.
- `repositoryId` is a UUID.
- `prefix` satisfies the same normalized-prefix contract as E1-S1.
- `database` equals exactly `tbtm.db`.
- `createdAt` is an RFC 3339 UTC timestamp.

Missing config, malformed JSON, unknown fields when they would make interpretation ambiguous, unsupported schema versions, and invalid field values return `INVALID_CONFIGURATION`. Reject absolute, nested, traversal, or alternative database paths before database access.

### 3. Open the existing database

- Accept explicit caller access intent.
- Map read-only intent to SQLite read-only flags.
- Map mutation intent to read-write flags without create.
- Never use default SQLite behavior that creates a missing configured file.
- Map missing, non-SQLite, corrupt, locked beyond the normal busy policy, or otherwise unusable DB states to `DATABASE_UNAVAILABLE`; preserve permission-denied classification for exit code 5.

The shared resolver stops here. It does not compare duplicated metadata or run `quick_check` for every command.

### 4. Inspect repository health

The health service used by `repo status`:

1. Runs resolution with read-only access.
2. Reads repository metadata from the database.
3. Compares `repositoryId`, `prefix`, `schemaVersion`, and `createdAt` with config.
4. Returns `INVALID_CONFIGURATION` with the mismatched field names when the pair disagrees.
5. Runs `PRAGMA quick_check` and requires the successful SQLite result.
6. Returns a typed healthy result without writing config, database, journal, or WAL files.

Configure the read-only connection so health inspection cannot trigger migrations or persistent journal changes.

### 5. Render output

Human success output shows:

- Repository root.
- Config path.
- Database path.
- Repository ID.
- Stored prefix.
- Schema version.
- Health.

JSON success uses the shared E1-S1 envelope and camelCase fields. `health` is `healthy` when the command succeeds. Error details include the failed phase/check, relevant path, mismatched fields where applicable, and a short remediation suggestion.

### 6. Map exit codes

| Exit | Condition |
|---:|---|
| 0 | Healthy repository |
| 1 | Database unavailable or another operational failure |
| 2 | Invalid or unsupported configuration, including config/DB mismatch |
| 3 | Repository not initialized |
| 5 | Permission denied |

## Test plan

### Unit tests

- Schema-version-1 config parsing and field validation.
- Exact `tbtm.db` acceptance; absolute, traversal, nested, and alternative paths rejected.
- UUID, prefix, timestamp, and unsupported-version validation.
- Typed error and exit-code mapping.
- Health-result serialization and human rendering.

### Integration tests

- Resolve from Git root and nested worktree directories.
- Resolve from a non-Git current directory.
- Rename the repository directory and retain stored prefix and identity.
- Missing `.tbtm`, missing config, malformed JSON, invalid fields, and unsupported schema.
- Missing DB does not create a file.
- Unreadable, invalid SQLite, corrupt, and operationally unavailable DB classification.
- Mismatch each of repository ID, prefix, schema version, and creation time.
- Healthy and failed `PRAGMA quick_check` behavior.
- Prove `repo status` does not change config or DB bytes and creates no journal/WAL or other workspace artifacts.
- Human and JSON snapshots plus exact exit-code assertions.
- Practical platform tests for Linux, macOS, and Windows path and permission differences.

## Verification

Provide or reuse `mise` tasks for:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Smoke verification:

1. Initialize a temporary repository using E1-S1.
2. Run `tbtm repo status --json` from its root and a nested directory.
3. Confirm output uses the stored identity and prefix.
4. Rename the repository directory and confirm output values remain stable.
5. Copy config and DB bytes, rerun status, and confirm no bytes or persistent SQLite side files changed.
6. Test one invalid-config case and one unavailable-database case and confirm distinct error codes and exits.

## Definition of done

- Parent story functional and non-functional acceptance criteria pass.
- All repository consumers can call the shared resolver with explicit access intent.
- Missing databases cannot be created as a side effect of resolution.
- `repo status` performs full metadata comparison and `quick_check` read-only.
- Human and JSON contracts distinguish uninitialized, invalid-config, database, and permission failures.
- Formatting, lint, and tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E1-S1 epic](../epics/E1-S1-initialize-repository.md)
- [E1-S1 implementation task](E1-S1-T1-implement-repository-initialization.md)
- [SQLite URI filenames](https://www.sqlite.org/uri.html)
- [SQLite opening flags](https://www.sqlite.org/c3ref/c_open_autoproxy.html)
- [SQLite PRAGMA quick_check](https://www.sqlite.org/pragma.html#pragma_quick_check)
- [rusqlite OpenFlags](https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html)
