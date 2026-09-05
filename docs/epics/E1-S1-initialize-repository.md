---
id: E1-S1
kind: epic
planning_status: done
implementation_status: done
contract_depends_on: []
---

# E1-S1: Initialize a repository

## Outcome

A user can initialize one repository-local TBTM workspace, optionally exclude it from Git, force a fresh initialization with backup, and uninstall all TBTM repository files.

## User story

As a user, I want to initialize TBTM in a repository so that the repository has an independent task database and stable configuration.

## Commands

```text
tbtm init [--prefix <prefix>] [--stealth] [--force] [--yes] [--json]
tbtm uninstall [--yes] [--dry-run] [--json]
```

## Product decisions

### Technology

- CLI name: `tbtm`.
- Language: Rust, pinned and installed through `mise`.
- Storage: bundled SQLite.
- Workspace files:

  ```text
  .tbtm/config.json
  .tbtm/tbtm.db
  ```

### Repository root

- In a Git repository, use the main worktree root as the canonical repository root, including when invoked from a linked worktree.
- Resolve the main worktree through trusted Git common metadata; do not store a user-controlled redirect in TBTM configuration.
- Outside Git, use the current directory as the repository root.
- Running from a nested directory still targets the resolved canonical root.
- See E1-S5 for the linked-worktree resolution contract.

### Prefix

- Default input is the repository directory name; `--prefix` overrides it.
- Normalize Latin text to lowercase ASCII.
- After Latin transliteration, replace runs outside ASCII `[a-z0-9]` with `-`, then trim leading/trailing `-`.
- Normalize first, then require length `1–48`.
- Never truncate silently.
- Empty or overlong output returns `INVALID_PREFIX`.

### Configuration

`.tbtm/config.json` stores:

```json
{
  "schemaVersion": 1,
  "repositoryId": "UUID v4",
  "prefix": "repository-prefix",
  "database": "tbtm.db",
  "createdAt": "RFC 3339 UTC timestamp"
}
```

### Default statuses

Initialization creates UUID-backed statuses in this order:

| Name | Completed | Order |
|---|---:|---:|
| Todo | false | 0 |
| In progress | false | 1 |
| Done | true | 2 |

The initial schema and statuses are created in one SQLite transaction.

E3-S1 later extends the status model with stable codes `to_do`, `in_progress`, and `done`. Because no released repository format requires compatibility, that story updates baseline migration `0001` and the fresh-initialization seed path directly; it does not backfill development-only databases. The completed E1-S1 implementation does not itself claim status-code behavior.

### Existing and forced initialization

- A valid existing workspace returns `ALREADY_INITIALIZED` without changing it.
- A missing, corrupt, or mismatched config/DB pair returns `INVALID_INITIALIZATION`.
- `--force` requires confirmation when `.tbtm` exists; `--yes` skips the prompt.
- Confirmed force initialization moves the old workspace to `.tbtm.backup-<UTC timestamp>-<short UUID>`, then creates a new workspace.
- Backups are not automatically removed.
- If `.tbtm` does not exist, `--force` behaves like normal init: no prompt and no backup.

### Failure model

- Filesystem initialization is intentionally simple and best-effort.
- Order: create DB, write config, update `.gitignore` only when `--stealth` is used.
- When a step fails, stop, report that step and artifacts already created, then suggest `tbtm uninstall` if cleanup is wanted.
- No automatic filesystem rollback or recovery is required.
- DB/config may already form a valid workspace when stealth fails. A later init then returns `ALREADY_INITIALIZED`; the user may fix `.gitignore` manually or uninstall and retry.

### Stealth

- `--stealth` adds exact line `/.tbtm/` to root `.gitignore` if that exact line is absent.
- Create `.gitignore` when missing.
- Only write when `.gitignore` is a regular file; symlink or other file type returns an error.
- Preserve existing content and its line-ending style where practical. An unterminated last line receives a separator before the new rule.
- Without `--stealth`, TBTM does not modify `.gitignore`.

### Uninstall

- Removes `.tbtm`, direct-root names beginning `.tbtm.backup-` or `.tbtm.staging-`, and every exact `/.tbtm/` line from root `.gitignore`.
- Never deletes `.gitignore` itself.
- Never follows symlinks; a matching symlink is unlinked only.
- Never removes paths outside the resolved repository root.
- `--dry-run` lists targets without modifying anything.
- Interactive execution asks for confirmation; non-interactive or JSON execution requires `--yes`, except dry-run.
- Cleanup continues after individual failures and reports them.
- Exit code is 5 if any permission failure occurs, otherwise 1 for another cleanup failure, otherwise 0.

## Functional acceptance criteria

1. `tbtm init` creates matching `.tbtm/tbtm.db` and `.tbtm/config.json` at the resolved canonical root; invocation from any linked worktree targets the main worktree.
2. Repository discovery works from root, nested Git directories, and non-Git directories.
3. Default and custom prefixes follow the documented normalization and length rules.
4. Config and database contain matching repository UUID, prefix, schema version, and creation time.
5. Initial migration and three default statuses commit together or roll back together inside SQLite.
6. Existing valid or invalid initialization states return distinct errors without silent overwrite.
7. Confirmed force initialization preserves the previous workspace as a uniquely named backup; force on a missing workspace acts like normal init.
8. Each failed init phase reports its failure and existing artifacts, without automatic filesystem rollback.
9. `--stealth` manages only the exact `/.tbtm/` rule and rejects non-regular `.gitignore` paths.
10. `tbtm uninstall --dry-run` changes nothing; confirmed uninstall removes only bounded TBTM artifacts and exact stealth rules.
11. Human and JSON output distinguish success, cancellation, validation failure, permission failure, and partial cleanup failure.

## Non-functional acceptance criteria

1. SQLite setup is transactional.
2. No cleanup follows symlinks or removes a path outside the repository root.
3. Existing workspaces are never overwritten without confirmed `--force`.
4. Runtime initialization performs no network access.
5. Common initialization feels immediate for a local personal repository.
6. Tests cover Linux, macOS, and Windows behavior at a practical level.

## Verification

- Query SQLite directly to verify schema version, repository metadata, and default statuses.
- Compare files before and after rejected/no-op operations.
- Test failure at DB, config, and stealth phases and inspect reported artifacts.
- Test uninstall dry-run, confirmation, symlink handling, and bounded deletion.
- Snapshot human/JSON results and assert exit codes.
- Run formatting, lint, and tests through `mise`.

## Out of scope

- Full repository health command; covered by E1-S2.
- Automatic filesystem rollback, crash recovery, or adversarial concurrent-filesystem protection.
- Agent registration, task CRUD, and final task schema.
- Permanent deletion of individual tasks.

## Implementation task

See [E1-S1-T1: Implement repository initialization](../tasks/E1-S1-T1-implement-repository-initialization.md).
