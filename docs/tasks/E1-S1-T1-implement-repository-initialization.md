---
id: E1-S1-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on: []
implementation_commit: 8721ecf
---

# E1-S1-T1: Implement repository initialization

## Parent story

[E1-S1: Initialize a repository](../epics/E1-S1-initialize-repository.md)

## Objective

Create the Rust project foundation and implement `tbtm init` and `tbtm uninstall` according to E1-S1. Keep filesystem behavior simple: execute phases in order, stop on failure, report partial artifacts, and let the user decide whether to uninstall and retry.

## Deliverables

- Rust workspace and `tbtm` binary.
- Rust toolchain and verification commands in `mise.toml`.
- Versioned initial SQLite migration.
- Repository-root discovery and prefix normalization.
- Init, force init, stealth, and uninstall command handlers.
- Stable human/JSON results and exit codes.
- Unit and integration tests.

## Proposed structure

```text
Cargo.toml
Cargo.lock
mise.toml
crates/
├── tbtm-core/
│   ├── migrations/0001_repository_foundation.sql
│   └── src/
└── tbtm-cli/
    └── src/main.rs
```

`tbtm-core` owns repository, config, database, init, stealth, and uninstall rules. `tbtm-cli` owns argument parsing, confirmation, output rendering, and process exit codes.

## Technical choices

- Pin a stable Rust version through `mise`; commit `Cargo.lock`.
- Use `clap` for CLI parsing.
- Use `rusqlite` with bundled SQLite.
- Use `serde`/`serde_json`, `uuid`, and `time` for persisted/output values.
- Use a Git-discovery crate rather than parsing `.git` manually.
- Use Unicode normalization/transliteration crates for deterministic Latin prefix normalization.
- Keep operations synchronous; no async runtime is needed.

Choose current compatible crate patch versions during implementation and record them in `Cargo.lock`; exact patch versions are not part of the product contract.

## Implementation design

### 1. Repository discovery

- Resolve the current Git worktree and its main worktree from trusted Git common metadata.
- Use the main worktree as the canonical repository root, including when invoked from a linked worktree.
- Fall back to the canonical current directory when outside Git.
- Wrap the current worktree root and canonical repository root in distinct path types so repository artifacts are always direct children of the canonical root.
- Do not parse a user-controlled path from TBTM configuration and do not create per-worktree stores. The detailed discovery and validation contract is defined by E1-S5.

### 2. Prefix normalization

- Normalize Unicode before transliteration so composed and decomposed Latin names match.
- Lowercase and transliterate Latin characters to ASCII.
- Replace other character runs with one `-`; trim `-` at both ends.
- Validate normalized length `1–48`; return `INVALID_PREFIX` otherwise.
- Apply the same function to directory-derived and explicit prefixes.

Required examples:

| Input | Result |
|---|---|
| `The Broke Task Management` | `the-broke-task-management` |
| `Quản Lý Công Việc` | `quan-ly-cong-viec` |
| `alpha___beta` | `alpha-beta` |
| `中文` | `INVALID_PREFIX` |

### 3. SQLite bootstrap

Migration 1 creates at least:

- `schema_migrations`.
- `repository_metadata` containing repository UUID, prefix, and creation time.
- `statuses` containing ID, unique name, completed flag, display order, and default flag.

Run migration metadata, repository metadata, and the three default-status inserts in one SQLite transaction. Enable foreign keys and a safe synchronous journal setting. After commit, run an integrity check before writing config.

E3-S1 owns the later addition of immutable codes `to_do`, `in_progress`, and `done`. Before the first released repository format, it may update baseline migration `0001` and the fresh-init inserts directly; compatibility backfill for development-only databases is not required.

### 4. Config

Write `.tbtm/config.json` after DB success. Use camelCase JSON, UUID v4, RFC 3339 UTC, `schemaVersion: 1`, and `database: "tbtm.db"`. Config and DB metadata must match.

### 5. Init flow

1. Resolve repository root.
2. Validate prefix and CLI input.
3. Inspect `.tbtm` without following symlinks.
4. Return `ALREADY_INITIALIZED` for a valid config/DB pair or `INVALID_INITIALIZATION` for another existing state unless force is confirmed.
5. For confirmed force with existing `.tbtm`, rename it to `.tbtm.backup-<timestamp>-<short UUID>`.
6. If force is passed but `.tbtm` is missing, continue as normal init without prompt or backup.
7. Create `.tbtm`, then DB, then config, then optional stealth rule.
8. Stop on the first failure. Report the failed phase and artifacts already created. Do not roll back filesystem changes.

Timestamp plus UUID is sufficient backup-name uniqueness for this personal local tool. If the generated path already exists, generate another UUID.

### 6. Stealth

- If root `.gitignore` is missing, create it and add `/.tbtm/`.
- If present, verify it is a regular file before writing. Reject symlinks and other file types.
- Add the exact line only when absent.
- Preserve existing content and prevailing line-ending style where practical.
- Write directly; failure is `GITIGNORE_UPDATE_FAILED` and does not undo DB/config.

No adversarial concurrent-filesystem protection is required for MVP. A basic type check is sufficient for this personal local tool.

### 7. Uninstall

Build a plan from direct children of the resolved root:

- Exact `.tbtm`.
- Names starting `.tbtm.backup-`.
- Names starting `.tbtm.staging-` for cleanup compatibility.
- Exact `/.tbtm/` lines in root `.gitignore`.

For matching entries, recursively remove real directories, remove regular files, and unlink symlinks without following them. Never accept a user-provided cleanup path. Never delete `.gitignore` itself.

`--dry-run` returns the plan without writing. Normal execution requires confirmation unless `--yes` is supplied. Continue after individual removal errors and return all failures.

### 8. Output and exit codes

JSON mode emits one object to stdout:

```json
{
  "ok": true,
  "data": {},
  "error": null
}
```

Errors use:

```json
{
  "ok": false,
  "data": null,
  "error": {
    "code": "INVALID_INITIALIZATION",
    "message": "...",
    "details": {}
  }
}
```

Init results include repository root, prefix, repository ID when created, config/DB paths, backup path when created, stealth state, and artifact states. Failures include failed phase, artifacts already created, and suggestion to run `tbtm uninstall` when relevant.

Uninstall results include `dryRun`, `cancelled`, `planned`, `removed`, `failed`, and number of stealth lines planned/removed.

Exit codes:

| Code | Meaning |
|---:|---|
| 0 | Success, no-op, dry-run, or declined confirmation |
| 1 | Operational failure or partial uninstall |
| 2 | Validation/state error or missing confirmation |
| 3 | Not found, reserved for later stories |
| 4 | Claim conflict, reserved for E5 |
| 5 | Permission failure |

During uninstall, permission failure takes precedence over other failure codes.

## Test plan

### Unit tests

- Prefix normalization, including Vietnamese composed/decomposed input, separators, empty result, and length boundaries.
- Config serialization and validation.
- Exact stealth-line addition/removal with LF, CRLF, empty file, and missing final newline.
- Safe direct-child uninstall target matching.
- Exit-code mapping and uninstall precedence.

### Integration tests

- Init in Git root, nested Git directory, and non-Git directory.
- Default/custom prefix and invalid prefix.
- DB/config identity match and exact default status UUID/name/order/completion values.
- Injected SQLite migration failure proves database records roll back together.
- Existing valid, corrupt, missing-config, and missing-DB workspace classification.
- Force confirmation, backup creation, and force on missing workspace.
- Failure during DB, config, or stealth reports correct phase and existing artifacts.
- Stealth missing/existing regular `.gitignore`; reject symlink/non-regular path.
- Uninstall dry-run, confirmation, no-op, backups, partial failures, and exact rule removal.
- Symlink cleanup does not remove its target; similarly named and nested unrelated files survive.
- Human and JSON snapshots plus exit-code assertions.

## Verification

Provide `mise` tasks for:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Smoke verification:

1. Initialize a temporary repository with `--prefix "Quản Lý" --stealth --json`.
2. Query SQLite for schema version, repository metadata, and ordered statuses.
3. Confirm config matches DB and `.gitignore` contains one exact rule.
4. Run uninstall dry-run and confirm no bytes changed.
5. Run confirmed uninstall and confirm only TBTM artifacts/rule were removed.

## Definition of done

- Parent story functional and non-functional acceptance criteria pass.
- Formatting, lint, and tests pass through `mise`.
- CLI help documents force, stealth, partial-init cleanup, and destructive uninstall.
- Failed init names the failed phase and existing artifacts; no automatic filesystem recovery is implemented.
- Uninstall never follows symlinks or removes paths outside repository root in supported tests.

## References

- [Product requirements](../PRD.md)
- [mise Rust backend](https://mise.jdx.dev/lang/rust.html)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [SQLite pragmas](https://www.sqlite.org/pragma.html)
- [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
- [clap](https://docs.rs/clap/latest/clap/)
- [Rust filesystem APIs](https://doc.rust-lang.org/std/fs/)
