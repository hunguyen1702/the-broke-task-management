# Repository Guidelines

## Project Structure & Module Organization

Rust Cargo workspace for `tbtm`, repository-local task-management CLI.

- `crates/tbtm-core/`: rules, SQLite, initialization, cleanup.
- `crates/tbtm-core/migrations/`: sequential migrations, e.g. `0002_add_tasks.sql`.
- `crates/tbtm-cli/`: Clap parsing, prompts, output, exit codes.
- `docs/epics/`, `docs/tasks/`: approved behavior contracts.
- `docs/testing/`: concise QA acceptance scenarios and optional run summaries.
- `docs/decisions/`: durable technical decisions and scenario-impact records.
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
- Before changing source code, claim a `ready` task by setting its implementation status to `in_progress` in the task frontmatter, the parent epic when applicable, and `docs/STATUS.md`. Re-read the status after the update; if another session already marked the task `in_progress`, stop and choose different work. Keep the claim visible until implementation and verification finish. This is mandatory when multiple agents or Codex sessions may share a worktree.
- Treat the approved epic/task as the behavior contract, preserve core/CLI boundaries, implement and test normal, invalid, no-op, migration, and transactional paths relevant to the task.
- Run format, lint, and tests. After implementation is complete, update the epic/task frontmatter and `docs/STATUS.md` before handoff or commit.
- Record only the acceptance impact classification (`none`, `revalidate`, `add`, or `supersede`) and affected story or existing scenario IDs when they are already known. Do not create, edit, review, approve, or execute files under `docs/testing/` during implementation. A non-`none` classification is a handoff to a separate acceptance-scenario workflow, not an implementation deliverable.

### Plan a story

- Follow `docs/handoff/planning-session.md`. Read the story and referenced domain/dependency sections in `docs/PRD.md`, related approved epic/task contracts, and `docs/STATUS.md`. Story frontmatter uses `contract_depends_on`; implementation-task frontmatter uses `depends_on`.
- A story may be planned only after all transitive `contract_depends_on` stories are planned and approved. Independent eligible stories may be planned in parallel. Before planning, claim the story as `in_progress` in `docs/STATUS.md` and re-read the claim; do not work on a story claimed by another session or overwrite its files.
- Summarize the task context before planning, confirm observable decisions with the user, complete the required reviews, and write both `docs/epics/<STORY>.md` and `docs/tasks/<STORY>-T1-*.md`; do not implement source code during planning.
- After creating the epic/task files, re-read every direct dependency epic and its implementation task when present. Cross-check terminology, identity, state transitions, persistence, output/error contracts, concurrency, and shared invariants. Resolve every conflict before marking planning `done`; record a concise `NO CONFLICT` result or remaining blocker in the planning handoff.
- Once planning is complete and reviewed, update the new documents' frontmatter, `docs/STATUS.md`, and the planning handoff immediately before the docs commit, then ask the user to confirm committing the documentation.
- Plan only the product contract and implementation task. Do not create or edit acceptance scenarios, assign acceptance scenario IDs, prescribe scenario cases, review the acceptance catalog, or execute acceptance commands. Acceptance impact is determined after implementation from the actual change.

### Build acceptance scenarios

- Start this workflow only for implemented behavior, separately from planning and implementation. Read `docs/STATUS.md`, implemented story contracts, recorded acceptance impact, public CLI help, and current tests.
- Generate the catalog without interviewing the user scenario by scenario. Use one concise file per story or cohesive user journey.
- Use searchable English directory and file names combined with epic/story IDs. Do not organize scenarios around code functions or private implementation details.
- Write cases as input plus expected user-visible output. Cover the main happy path and likely user errors; leave rare boundaries, races, fault injection, transaction internals, and database checks to Rust tests.
- Ask an independent QA reviewer to check common user-flow coverage and contract consistency, resolve meaningful blockers, then present the catalog for user approval. Do not execute scenarios during this workflow.
- This is the only workflow allowed to create, edit, supersede, review, or approve acceptance scenario files and catalog entries. Do not change source code, implementation-task status, or acceptance run results here.

### Revalidate acceptance scenarios

- Treat revalidation as part of the acceptance-scenario build/maintenance workflow, never as part of implementation or execution.
- Before execution or after public behavior changes, compare affected scenarios with current story contracts and CLI behavior.
- Add or update cases only for changed common workflows or likely user errors. Preserve approved scenario history when doing so remains useful; otherwise avoid ceremony for unexecuted drafts.
- Only `approved` scenarios are eligible for execution.

### Execute approved acceptance scenarios

- Start this workflow separately after the relevant scenarios have already been revalidated and approved. Run approved scenarios in isolated temporary repositories using `docs/testing/test-setup.md`.
- Compare each command's exit code and user-visible output with its expected result. Mark cases `passed`, `failed`, or `blocked`.
- Record pass/fail and preserve actual commands and output for failures. Add a run summary under `docs/testing/runs/` only when retained evidence is useful.
- Do not create, edit, revalidate, supersede, or approve scenario definitions during execution. If a scenario is stale or unapproved, stop it and return it to the acceptance-scenario workflow.
- Report failures without modifying source code or expected behavior. Creating a remediation task follows `docs/testing/test-setup.md`; implementing that task requires a separate approved implementation workflow.

### Record acceptance impact

- At implementation completion, every task must classify scenario impact as `none`, `revalidate`, `add`, or `supersede`. Internal-only changes normally use `none` because they are covered by Rust tests. Planning must not pre-author scenario IDs or cases.
- Create a record from `docs/decisions/TD-template.md` for meaningful architecture, persistence, compatibility, concurrency, migration, or public-contract decisions. A technical decision supplements but never replaces PRD/epic updates for observable behavior.
- Non-`none` impact queues only the affected user-facing scope for a later acceptance-scenario workflow; it never authorizes scenario edits or execution inside implementation.

### Keep documentation status current

- After implementing a task, mark implementation status and dependency readiness in its epic/task frontmatter and `docs/STATUS.md`.
- After planning a story, mark planning and implementation readiness in its epic/task frontmatter, `docs/STATUS.md`, and `docs/handoff/planning-session.md` immediately before committing the planning docs.
- Keep the acceptance summary in `docs/STATUS.md` aligned with `docs/testing/README.md`; keep detailed scenario and run history out of the implementation tables.

## Commit & Pull Request Guidelines

Use Conventional Commits: `feat(e1-s1): initialize repository workspace`. Keep subject imperative, scoped. Separate docs and code when practical.

PRs: affected epic/task, user-visible behavior, verification run, migration/filesystem/output changes. Add output or screenshots only when useful.
