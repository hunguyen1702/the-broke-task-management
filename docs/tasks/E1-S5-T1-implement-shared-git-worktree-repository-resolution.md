---
id: E1-S5-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - E1-S2-T1
---

# E1-S5-T1: Implement shared Git-worktree repository resolution

## Parent story

[E1-S5: Share repository state across Git worktrees](../epics/E1-S5-share-repository-state-across-git-worktrees.md)

## Objective

Extend repository discovery so every worktree of one Git repository targets the main worktree's single `.tbtm` store. Integrate the canonical root into initialization, resolution, health inspection, stealth, force initialization, uninstall, and all existing repository consumers.

## Deliverables

- Worktree-aware Git repository discovery in `tbtm-core`.
- Distinct canonical repository-root and current-worktree-root values.
- Retrofit the completed E1-S1 lifecycle code: `init`, `--force`, backup handling, `--stealth`, and `uninstall`.
- Retrofit the completed E1-S2 shared resolver, health service, result types, and `repo status` command.
- Audit every existing resolver consumer and remove direct construction of `<current-worktree>/.tbtm` paths.
- Preserve the canonical-resolver integration contract for E1-S3 `agent register` and E1-S4 `agent list`, whether those commands exist when this task begins or are implemented later.
- Updated human and JSON repository-status output.
- Typed discovery errors and existing exit-code integration.
- Multi-worktree, concurrency, filesystem-safety, and regression tests.

## Proposed structure

Extend existing repository modules rather than introducing a parallel resolver:

```text
crates/tbtm-core/src/repository/
├── discovery.rs
├── resolver.rs
└── health.rs

crates/tbtm-cli/src/commands/
└── repo_status.rs
```

Exact filenames may follow the implemented workspace. Core code owns Git discovery, validated root types, and artifact targeting. CLI code owns rendering and exit mapping.

## Retrofit boundary

E1-S1 and E1-S2 may already be implemented when this task starts. E1-S5 therefore changes their existing code paths directly; updating older planning documents is not a substitute for this work.

The implementation must audit and adapt at least:

- `RepositoryRoot::discover` or its implemented equivalent.
- `resolve_repository`, its resolved-repository type, and all callers.
- `initialize`, existing-workspace validation, force-init backup selection, and result paths.
- Stealth-rule addition and removal at the canonical main-worktree `.gitignore`.
- `uninstall`, including dry-run planning and bounded removal.
- `inspect_repository_health`, `RepositoryHealth`, human output, and JSON serialization.
- CLI help and confirmation text whose meaning depends on which repository root owns `.tbtm`.
- All unit and integration fixtures that assume the nearest/current worktree owns `.tbtm`.

Consumer audit matrix:

| Consumer | Required E1-S5 behavior |
|---|---|
| `tbtm init` | Create or validate only canonical main-root storage |
| `tbtm init --force` | Back up and replace only canonical main-root storage |
| `tbtm init --stealth` | Edit only the main worktree `.gitignore` |
| `tbtm uninstall` | Preview/remove only canonical main-root artifacts and stealth rule |
| `tbtm repo status` | Read canonical DB and report both root fields without mutation |
| `tbtm agent register` | Open canonical DB through shared resolver; never construct a local path |
| `tbtm agent list` | Read canonical DB through shared resolver; never construct a local path |

If E1-S3 or E1-S4 is not implemented yet, its row remains a required API contract and integration-test case when that command is added. E1-S5 must still leave no alternative resolver or path-building API that encourages a per-worktree store.

## Technical choices

- Reuse the selected Git-discovery library and its structured metadata APIs; do not manually parse `.git` files.
- Resolve the current worktree, common Git directory, and registered main worktree as one discovery result.
- Canonicalize existing paths and validate their relationship against Git's registered worktree metadata before constructing TBTM paths.
- Represent `repository_root` and `worktree_root` as distinct typed fields even when their values match.
- Keep `database: "tbtm.db"` in schema-version-1 config; it remains a fixed filename under canonical `.tbtm`, never a redirect.
- Preserve synchronous local operation and SQLite as the sole concurrency boundary.

## Implementation flow

### 1. Discover Git topology

Starting from the command's current directory:

1. Ask the Git library for the containing worktree and common repository metadata.
2. Identify the registered main worktree root from that metadata.
3. Canonicalize and validate both roots.
4. Return a discovery value containing the current `worktree_root`, canonical `repository_root`, and trusted common-Git identity needed for consistency checks.
5. If no Git repository exists, preserve E1-S1 fallback: both roots are the canonical current directory.

Do not derive the main root by guessing a parent directory name. Do not read a path redirect from TBTM config. Treat bare repositories or layouts without a usable main worktree as unsupported repository operational failures.

### 2. Classify discovery failures

- No TBTM store at a successfully resolved canonical root remains `REPOSITORY_NOT_INITIALIZED`, exit code 3.
- Malformed, missing, or inconsistent Git common/worktree metadata returns `REPOSITORY_UNAVAILABLE`, exit code 1, with the failed discovery phase and remediation.
- Permission-denied discovery returns exit code 5.
- Existing `INVALID_CONFIGURATION` and `DATABASE_UNAVAILABLE` behavior begins only after canonical-root discovery succeeds.
- Never retry against `worktree_root/.tbtm` when `repository_root` differs.

### 3. Target lifecycle operations

Pass the canonical root to every E1-S1 filesystem operation:

- Normal and force initialization.
- Workspace validation and backup naming.
- Config and database creation.
- Main `.gitignore` stealth-rule insertion/removal.
- Uninstall preview and removal.

The current worktree path provides invocation context only. All mutation bounds and symlink checks remain anchored at the canonical root. Do not inspect, migrate, merge, rename, or delete a linked worktree's local `.tbtm` path.

### 4. Extend shared resolution

Update E1-S2 resolution to return:

```text
repository_root  canonical main worktree root
worktree_root    current worktree root
config_path      repository_root/.tbtm/config.json
database_path    repository_root/.tbtm/tbtm.db
```

All existing and future consumers use this result. Remove assumptions that the nearest worktree root necessarily owns the database. Preserve explicit read-only/read-write access intent and no-create SQLite flags.

### 5. Render output

Add `worktreeRoot` to successful `repo status` human and JSON output. Keep `repositoryRoot` as the canonical store owner. Paths use the established platform-aware absolute-path serialization contract.

Errors use the shared envelope. `REPOSITORY_UNAVAILABLE` details include the failed Git discovery phase and an actionable suggestion, without exposing unsafe unvalidated paths as authoritative storage locations.

### 6. Preserve read-only and concurrency behavior

- `repo status` opens only the canonical database read-only and retains E1-S2 metadata and `quick_check` validation.
- It must not write Git metadata, TBTM files, SQLite bytes, journal, WAL, or shared-memory files.
- Concurrent commands from separate worktrees rely on the same SQLite transaction and busy-handling policies as commands from one directory.
- This task does not introduce external lock files or cross-database coordination.

## Test plan

### Unit tests

- Discovery result distinguishes equal and different repository/worktree roots.
- Canonical TBTM paths always derive from `repository_root`.
- Non-Git fallback returns equal canonical roots.
- Typed Git discovery, permission, uninitialized, configuration, and database errors map to stable exits.
- JSON and human rendering include both root fields.

### Integration tests

- Main worktree and two linked worktrees resolve identical config/database paths and repository metadata.
- Nested directories in every worktree retain the correct `worktreeRoot` and canonical `repositoryRoot`.
- Init from a linked worktree creates only main-root `.tbtm` and main-root stealth rule.
- Re-init and force-init from a linked worktree validate, back up, or replace only canonical-root storage.
- Stealth init from a linked worktree edits only the main worktree `.gitignore`.
- Uninstall dry-run and confirmed uninstall from a linked worktree report and affect only canonical-root artifacts.
- Repository status from a linked worktree reads the canonical database and reports distinct canonical/current roots.
- A linked-worktree `.tbtm` fixture is ignored and remains byte-for-byte unchanged.
- Agent registrations from multiple worktrees are visible in one database and retain uniqueness under concurrent processes.
- Agent listing from every worktree returns the same identities and claims when E1-S4 is present.
- Missing or inconsistent registered-main/common metadata returns `REPOSITORY_UNAVAILABLE` without fallback or filesystem mutation.
- Permission failures retain exit code 5.
- Bare repository invocation is rejected without creating TBTM artifacts.
- `repo status` from each worktree leaves config, DB, Git metadata, and SQLite side-file set unchanged.
- Replace existing fixtures and assertions that encode "nearest worktree root owns `.tbtm`" with canonical-main-root assertions.
- Preserve single-worktree Git and non-Git behavior with updated root-field snapshots.

Platform coverage should account for path separators, canonicalization, drive prefixes, and practical Git worktree layouts on Linux, macOS, and Windows.

## Verification

Reuse the workspace checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Smoke verification:

1. Initialize a temporary Git repository and create two linked worktrees.
2. Run `tbtm init --stealth --json` from one linked worktree.
3. Run `tbtm repo status --json` from all three worktrees and compare repository identity and canonical paths.
4. Register different agents concurrently from separate worktrees and verify both rows in the one canonical database.
5. Run uninstall dry-run from another linked worktree and confirm it lists only main-root artifacts.
6. Run confirmed uninstall and verify linked-worktree files remain untouched.

## Definition of done

- Parent story functional and non-functional acceptance criteria pass.
- Every Git worktree uses the main worktree's one `.tbtm` store.
- Existing repository commands consume one worktree-aware shared resolver.
- Init, force-init, stealth, uninstall, status, and every implemented agent consumer pass linked-worktree integration tests.
- No production code path independently joins `<current-worktree>/.tbtm`; canonical artifact paths come from the shared discovery/resolution boundary.
- No linked-worktree-local store, symlink, redirect, or fallback is created or selected.
- Lifecycle commands remain bounded to the canonical root.
- Status remains read-only and reports both root concepts accurately.
- Multi-process tests prove shared visibility and database integrity.
- Formatting, lint, and tests pass through `mise`.

## References

- [Product requirements](../PRD.md)
- [E1-S1 epic](../epics/E1-S1-initialize-repository.md)
- [E1-S1 implementation task](E1-S1-T1-implement-repository-initialization.md)
- [E1-S2 epic](../epics/E1-S2-resolve-repository-configuration.md)
- [E1-S2 implementation task](E1-S2-T1-implement-repository-configuration-resolution.md)
- [Git worktree documentation](https://git-scm.com/docs/git-worktree)
- [Git repository layout](https://git-scm.com/docs/gitrepository-layout)
- [Git rev-parse path discovery](https://git-scm.com/docs/git-rev-parse)
- [SQLite transactions](https://www.sqlite.org/transactional.html)
- [rusqlite open flags](https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html)
