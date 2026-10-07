# Repository Guidelines

## Project Structure & Module Organization

Rust Cargo workspace for `tbtm`, repository-local task-management CLI.

- `crates/tbtm-core/`: rules, SQLite, initialization, cleanup.
- `crates/tbtm-core/migrations/`: sequential migrations, e.g. `0002_add_tasks.sql`.
- `crates/tbtm-cli/`: Clap parsing, prompts, output, exit codes.
- `docs/epics/`, `docs/tasks/`: approved behavior contracts.
- `docs/testing/`: concise QA acceptance scenarios and optional run summaries.
- `mise.toml`: Rust pin and verification tasks.

Keep CLI code out of core; core operations must be testable without process boundary.

## Build, Test, and Development Commands

Prefix shell commands with `rtk`.

```bash
rtk cargo build -p tbtm       # Build CLI binary
rtk mise run format           # Check Rust formatting
rtk mise run lint             # Run Clippy with warnings denied
rtk mise run test             # Run workspace unit and integration tests
rtk cargo run -p tbtm -- init --help
```

Run all three before handoff. Never commit `target/`.

## Coding Style & Naming Conventions

Use Rust 2024, four-space indentation, `cargo fmt`, Clippy `-D warnings`.

Use `snake_case` functions/modules/fields, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants. Persist JSON camelCase via Serde. Keep error codes stable uppercase, e.g. `INVALID_INITIALIZATION`.

## Testing Guidelines

Put focused core unit tests beside behavior in `crates/tbtm-core/src/lib.rs`. Put process-output and exit-code coverage in `crates/tbtm-cli/tests/`. Name by behavior, e.g. `uninstall_dry_run_does_not_write`.

Use temporary directories for filesystem tests. Cover normal, invalid, no-op paths; prove migrations and related inserts are transactional.

## Commit & Pull Request Guidelines

Use Conventional Commits: `feat(e1-s1): initialize repository workspace`. Keep subject imperative, scoped. Separate docs and code when practical.

PRs: affected epic/task, user-visible behavior, verification run, migration/filesystem/output changes. Add output or screenshots only when useful.

## Agent skills

### Issue tracker

Engineering skill tickets use local Markdown under `.scratch/<feature>/`.
Read `docs/agents/issue-tracker.md` before ticket operations.

### Triage labels

Use the five default triage roles.
Read `docs/agents/triage-labels.md` when triaging tickets.

### Domain docs

Single-context layout: root `GLOSSARY.md`.
Read `docs/agents/domain.md` before domain exploration or decision work.
