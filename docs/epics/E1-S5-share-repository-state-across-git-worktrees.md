---
id: E1-S5
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E1-S2
---

# E1-S5: Share repository state across Git worktrees

## Outcome

Every worktree of one Git repository uses the same TBTM configuration and SQLite database while repository-local storage remains in the main worktree's `.tbtm` directory.

## User story

As a coding agent working in a Git linked worktree, I want every worktree of the same repository to resolve one shared TBTM store so that agents coordinate identities, tasks, and claims across isolated source-code workspaces.

## Product decisions

### One canonical store

- One logical Git repository has exactly one TBTM store at `<main-worktree-root>/.tbtm/`.
- The main worktree and every linked worktree use that same configuration and database.
- A linked worktree never receives an active `.tbtm` store of its own.
- TBTM does not use symlinks and does not persist a redirect path in configuration.
- This is a pre-release MVP contract. Migration, merging, or selection among independently initialized per-worktree stores is not required.

### Trusted discovery

- Resolve the current worktree and main worktree from Git common metadata.
- Do not infer the main worktree from directory naming, manually parse a linked worktree's `.git` file, or trust a path supplied by `.tbtm/config.json`.
- Canonicalize and validate discovered paths before accessing TBTM artifacts.
- Missing, inaccessible, bare, or internally inconsistent Git metadata returns an actionable repository operational error. Permission denial retains exit code 5.
- Never fall back to a per-worktree store after Git repository discovery succeeds but canonical-root resolution fails.

### Existing repository lifecycle

- `tbtm init` from any worktree creates or validates the store under the main worktree root.
- `--force`, backups, and partial-artifact reporting operate only at the canonical root.
- `--stealth` manages the exact `/.tbtm/` rule in the main worktree's `.gitignore`.
- `tbtm uninstall` previews and removes only canonical-root TBTM artifacts and the exact main-worktree stealth rule.
- An unrelated `.tbtm` directory in a linked worktree is outside the active contract and is neither selected nor modified.

### Non-Git behavior

- A non-Git invocation keeps the E1-S1 behavior: the canonical repository root and current worktree root are the canonical current directory.
- Nested discovery outside Git is not introduced by this story; commands target the current non-Git directory.

### Resolver and output

- E1-S2's shared resolver owns worktree-aware discovery for all CLI commands and the future extension boundary.
- `repositoryRoot` means the canonical main worktree root containing `.tbtm`.
- `worktreeRoot` means the worktree containing the command's current location.
- For a main-worktree or non-Git invocation, both fields contain the same path.
- `configPath` and `databasePath` always point below `repositoryRoot/.tbtm`.

Example from a linked worktree:

```json
{
  "ok": true,
  "data": {
    "repositoryRoot": "/project-main",
    "worktreeRoot": "/project-feature",
    "configPath": "/project-main/.tbtm/config.json",
    "databasePath": "/project-main/.tbtm/tbtm.db",
    "repositoryId": "UUID",
    "prefix": "project",
    "schemaVersion": 1,
    "health": "healthy"
  },
  "error": null
}
```

### Concurrency

- Commands from separate worktrees open the same SQLite file.
- Existing transaction, busy-handling, uniqueness, and no-create access contracts apply unchanged.
- This story does not add replication, synchronization, file locking outside SQLite, or a second cache/store.

## Functional acceptance criteria

1. Main and linked worktrees resolve the same canonical repository root, configuration path, database path, repository ID, and stored prefix.
2. Invoking initialization from any linked worktree creates or validates only `<main-worktree-root>/.tbtm`.
3. Mutations from different worktrees become visible through the same SQLite database.
4. No normal operation creates a `.tbtm` directory, config, database, symlink, or pointer file in a linked worktree.
5. Repository discovery uses trusted Git common metadata and never accepts an arbitrary storage redirect from TBTM configuration.
6. A discovery failure does not fall back to isolated worktree state and returns an actionable typed error.
7. `repo status` reports both canonical `repositoryRoot` and current `worktreeRoot`, with canonical config/database paths.
8. `repo status` remains read-only from every worktree and creates no persistent SQLite side files.
9. Stealth, force initialization, backups, and uninstall target only the main worktree's canonical artifacts.
10. Non-Git and single-worktree use retain their established observable behavior apart from the added `worktreeRoot` output field.

## Non-functional acceptance criteria

1. Concurrent local processes in separate worktrees cannot corrupt the shared SQLite database.
2. Resolution performs no network access and feels immediate for a local repository.
3. Canonical path derivation is portable across practical Linux, macOS, and Windows Git worktree layouts.
4. Filesystem mutation remains bounded to the canonical repository root and does not follow symlinks.
5. Worktree discovery and read-only status inspection do not mutate Git metadata.

## Verification

- Create a main worktree plus at least two linked worktrees and compare all reported canonical paths and repository metadata.
- Initialize and register agents from different worktrees; query the canonical SQLite file to prove shared visibility.
- Run concurrent mutations from separate worktrees and verify database integrity and uniqueness constraints.
- Place an unrelated `.tbtm` in a linked worktree and prove it is ignored and unchanged.
- Exercise missing, inaccessible, bare, and inconsistent Git metadata without per-worktree fallback.
- Verify force-init, stealth, dry-run uninstall, and confirmed uninstall from a linked worktree affect only canonical-root artifacts.
- Compare config/database bytes and persistent side files before and after `repo status` from each worktree.
- Re-run single-worktree Git and non-Git E1-S1/E1-S2 regression tests.

## Out of scope

- Migration or merge of independently initialized per-worktree stores.
- User-selectable storage locations or pointer files.
- Symlink-based storage sharing.
- Bare Git repositories and worktree setups without a usable main worktree.
- Network or cross-machine database sharing.
- Final claim-race verification, covered by E9-S1 after E5-S1 and E5-S2.

## Implementation task

See [E1-S5-T1: Implement shared Git-worktree repository resolution](../tasks/E1-S5-T1-implement-shared-git-worktree-repository-resolution.md).
