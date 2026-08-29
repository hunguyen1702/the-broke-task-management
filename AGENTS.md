# Repository Guidelines

## Project Structure & Module Organization

Rust Cargo workspace for `tbtm`, repository-local task-management CLI.

- `crates/tbtm-core/`: rules, SQLite, initialization, cleanup.
- `crates/tbtm-core/migrations/`: sequential migrations, e.g. `0002_add_tasks.sql`.
- `crates/tbtm-cli/`: Clap parsing, prompts, output, exit codes.
- `docs/epics/`, `docs/tasks/`: approved behavior contracts.
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

## Task Workflows

### Implement an approved task

- Read the implementation task in `docs/tasks/`, its parent contract in `docs/epics/`, the referenced PRD sections, and `docs/STATUS.md`; verify dependencies and inspect the current code before changing it.
- Treat the approved epic/task as the behavior contract, preserve core/CLI boundaries, implement and test normal, invalid, no-op, migration, and transactional paths relevant to the task.
- Run format, lint, and tests. After implementation is complete, update the epic/task frontmatter and `docs/STATUS.md` before handoff or commit.

### Plan a story

- Follow `docs/handoff/planning-session.md`. Read the story and referenced domain/dependency sections in `docs/PRD.md`, related approved epic/task contracts, and `docs/STATUS.md`.
- Summarize the task context before planning, confirm observable decisions with the user, complete the required reviews, and write both `docs/epics/<STORY>.md` and `docs/tasks/<STORY>-T1-*.md`; do not implement source code during planning.
- Once planning is complete and reviewed, update the new documents' frontmatter, `docs/STATUS.md`, and the planning handoff immediately before the docs commit, then ask the user to confirm committing the documentation.

### Keep documentation status current

- After implementing a task, mark implementation status and dependency readiness in its epic/task frontmatter and `docs/STATUS.md`.
- After planning a story, mark planning and implementation readiness in its epic/task frontmatter, `docs/STATUS.md`, and `docs/handoff/planning-session.md` immediately before committing the planning docs.

## Commit & Pull Request Guidelines

Use Conventional Commits: `feat(e1-s1): initialize repository workspace`. Keep subject imperative, scoped. Separate docs and code when practical.

PRs: affected epic/task, user-visible behavior, verification run, migration/filesystem/output changes. Add output or screenshots only when useful.
