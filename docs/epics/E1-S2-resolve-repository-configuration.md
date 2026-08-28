---
id: E1-S2
kind: epic
planning_status: done
implementation_status: done
depends_on:
  - E1-S1
---

# E1-S2: Resolve repository configuration

## Outcome

CLI commands and the Visual Studio Code extension can consistently discover an initialized TBTM workspace, target its configured SQLite database, and report repository health without changing repository state.

## User story

As a CLI or extension client, I want to discover the repository task configuration so that commands target the correct database.

## Command

```text
tbtm repo status [--json]
```

## Product decisions

### Shared repository resolver

- `tbtm-core` provides one resolver for all CLI commands and the future extension boundary.
- Repository-root discovery follows E1-S1 and E1-S5: use the main worktree root for every worktree of a Git repository, or the current directory when outside Git.
- Every repository operation performs only the resolution required to locate its database:
  1. Resolve the repository root.
  2. Find and parse `.tbtm/config.json`.
  3. Validate the configuration fields needed for safe database access.
  4. Require schema version 1 to contain the exact database value `tbtm.db`.
  5. Open the existing database without implicitly creating it.
- The resolver accepts the caller's access intent so read operations can open read-only and later mutation commands can request read-write access.
- Ordinary commands do not compare every duplicated config/database metadata field and do not run SQLite health checks.

### Repository health command

`tbtm repo status` performs full read-only validation:

1. Run shared repository resolution.
2. Validate all schema-version-1 config fields.
3. Compare `repositoryId`, `prefix`, `schemaVersion`, and `createdAt` with database repository metadata.
4. Validate that applied internal migrations form a known, ordered, internally consistent history.
5. Run `PRAGMA quick_check`.

`schemaVersion` describes repository/config compatibility; internal migration numbers are tracked separately. Success reports `health: "healthy"`. The command may report pending compatible migrations, but never applies them, repairs data, or otherwise mutates the config or database.

### Configuration rules

Schema-version-1 configuration must contain:

- `schemaVersion: 1`.
- A valid UUID `repositoryId`.
- A valid stored normalized `prefix`.
- The exact value `database: "tbtm.db"`; absolute paths, traversal, nested paths, and alternative filenames are invalid.
- A valid RFC 3339 UTC `createdAt` value.

The stored prefix is authoritative. Repository discovery never derives it again, so renaming the repository directory does not change the prefix.

### Failure classification

- Git was discovered but its common/worktree metadata cannot resolve a usable canonical main worktree: `REPOSITORY_UNAVAILABLE`, exit code 1. This failure occurs before config or database resolution and never falls back to a per-worktree store.
- No `.tbtm` workspace: `REPOSITORY_NOT_INITIALIZED`, exit code 3.
- Missing, malformed, unsupported, or unsafe configuration: `INVALID_CONFIGURATION`, exit code 2.
- Config/database identity or metadata mismatch found by `repo status`: `INVALID_CONFIGURATION`, exit code 2.
- Config is usable but the database is missing, cannot be opened, is not valid SQLite, is corrupt, or fails `quick_check`: `DATABASE_UNAVAILABLE`, exit code 1.
- A permission-denied variant of repository, config, or database access uses exit code 5.

Errors identify the failed check and affected path where available, and give an actionable next step. This story does not provide automatic repair.

### Output

Human output gives a concise repository summary. JSON reuses the E1-S1 envelope:

```json
{
  "ok": true,
  "data": {
    "repositoryRoot": "/project",
    "worktreeRoot": "/project-feature",
    "configPath": "/project/.tbtm/config.json",
    "databasePath": "/project/.tbtm/tbtm.db",
    "repositoryId": "UUID",
    "prefix": "project",
    "schemaVersion": 1,
    "health": "healthy"
  },
  "error": null
}
```

Failures use the same stable `error.code`, `error.message`, and `error.details` structure defined by E1-S1.

## Functional acceptance criteria

1. Repository configuration resolves from the main Git worktree, a linked worktree, a nested directory in either worktree, and a non-Git directory.
2. All repository clients use the shared resolver rather than independently constructing config or database paths.
3. Resolution uses the stored prefix and repository identity; renaming the enclosing repository directory does not change them.
4. Schema version 1 accepts only the exact database filename `tbtm.db` and rejects paths that could escape or redirect outside `.tbtm`.
5. Opening an absent database never creates a new file.
6. `tbtm repo status` validates config, config/database metadata agreement, internal migration history, and SQLite `quick_check` without changing filesystem or database bytes.
7. Successful human and JSON output includes canonical repository root, current worktree root, config path, database path, repository ID, stored prefix, schema version, and healthy state.
8. Unavailable repository topology, uninitialized repositories, invalid configuration, unavailable databases, and permission failures have distinct documented errors and exit behavior.
9. Errors identify what failed and provide an actionable recovery direction without attempting repair.

## Non-functional acceptance criteria

1. Resolution and health inspection are read-only and perform no network access.
2. Ordinary resolution avoids full metadata comparison and SQLite health checks, keeping common local commands responsive.
3. Configuration cannot redirect database access outside the repository's `.tbtm` directory.
4. Existing database files are never created, migrated, repaired, or modified by `repo status`.
5. Tests cover Linux, macOS, and Windows path and permission behavior at a practical level.

## Verification

- Resolve initialized repositories from root and nested directories, both inside and outside Git.
- Exercise missing, inaccessible, bare, and inconsistent Git common/worktree metadata and confirm `REPOSITORY_UNAVAILABLE` occurs before config/database access.
- Rename a repository directory and confirm the stored identity and prefix remain unchanged.
- Exercise missing, malformed, unsupported, and unsafe config variants.
- Exercise missing, unreadable, invalid, corrupt, and metadata-mismatched databases.
- Compare config and database bytes before and after `repo status` success and failure.
- Snapshot human and JSON results and assert exit codes.
- Run formatting, lint, and tests through `mise`.

## Out of scope

- Automatic config or database repair.
- Applying migrations or defining compatibility policy for future repository schema versions. This story only validates the internal migration history for supported repository schema version 1.
- Backup creation, restoration, or recovery workflows.
- Background or periodic repository health monitoring.
- Agent registration, task operations, or extension refresh behavior.

## Implementation task

See [E1-S2-T1: Implement repository configuration resolution](../tasks/E1-S2-T1-implement-repository-configuration-resolution.md).
