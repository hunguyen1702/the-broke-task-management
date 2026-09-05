# Product Requirements Document: Agent Task

## 1. Document information

| Field | Value |
|---|---|
| Product name | Agent Task (working name) |
| Document type | Product Requirements Document |
| Product stage | MVP definition |
| Target user | Individual developer working with multiple coding agents |
| Deployment model | Local-first, one SQLite database per repository |
| Primary agent interface | Command-line interface (CLI) |
| Human interface | CLI and Visual Studio Code extension |

## 2. Product summary

Agent Task is a local-first task management tool designed for software projects in which multiple coding agents work concurrently. It provides a shared repository-level task database, an atomic claiming mechanism, dependency-aware task discovery, task discussions, and a Visual Studio Code scrum board.

The product must allow an agent to quickly find and atomically claim work that is not blocked, without accidentally selecting a task whose prerequisites are incomplete or a task already owned by another agent.

The product is intended for personal use. It does not require enterprise authentication, authorization, remote synchronization, or multi-tenant administration.

## 3. Problem statement

Ordinary task trackers are primarily designed for humans. Coding agents need a smaller and more deterministic interface that can answer the following questions reliably:

- Which task can I work on now?
- Is the task blocked by another task?
- Has another agent already claimed it?
- Can I claim it without a race condition?
- What is the task's goal, acceptance criteria, relevant code, and dependency context?
- How can a human inspect and manage the same task state from the coding workspace?

Without a common task database and atomic claim operation, multiple agents can select the same work, start blocked work, or leave repository state that is difficult for the user to understand.

## 4. Product goals

### 4.1 Primary goals

1. Provide a repository-local source of truth for coding tasks.
2. Allow multiple agents to safely claim work without claim conflicts.
3. Prevent blocked tasks from being returned as available work.
4. Represent both scope hierarchy and execution dependencies.
5. Give tasks sufficient context for coding work, including goals, acceptance criteria, estimates, references, and comments.
6. Provide a stable, machine-readable CLI suitable for coding agents.
7. Provide a complete Visual Studio Code UI for human planning and supervision.
8. Keep configuration and operation appropriate for a personal project.

### 4.2 Success indicators

The MVP is successful when:

- Two or more concurrent agents cannot successfully claim the same task.
- An agent can obtain and claim the highest-priority available task with one atomic operation.
- A task with any incomplete upstream dependency is never returned as available.
- Completing or archiving an upstream dependency can make downstream work available immediately.
- A user can manage the full task lifecycle without leaving Visual Studio Code.
- CLI JSON output is stable enough for an agent to consume without parsing human-formatted text.

## 5. Non-goals

The MVP will not provide:

- Cloud hosting or synchronization between machines.
- Multiple users, teams, roles, or access-control policies.
- Enterprise audit, compliance, retention, or backup policies.
- Authentication or security boundaries between local agents.
- Integration with external issue trackers.
- Automated scheduling, sprint forecasting, or resource allocation.
- Automatic claim expiry, background heartbeats, or a coordination daemon.
- Real-time collaborative editing.
- Custom task types.
- General-purpose project management features unrelated to coding agents.

## 6. Users and actors

### 6.1 Human user

The human user owns the local repository and has full control through the CLI and Visual Studio Code UI. The user can manage tasks, statuses, relationships, agents, claims, and all comments.

The human is represented by the logical actor `user`. This is a product convention rather than an authentication boundary.

### 6.2 Coding agent

A coding agent registers itself in a repository through the CLI. Each registration creates:

- A generated UUID.
- A user-supplied base name.
- A unique display name combining the base name and a UUID-derived suffix, for example `john-x1y2z3`.

Registering the same base name again creates a new identity rather than reusing an earlier agent.

An agent operates under its registered identity when claiming tasks, updating tasks, and creating or deleting its own comments.

## 7. Core concepts

### 7.1 Repository

Each source-code repository uses one SQLite database. Repository initialization records product configuration, including the task ID prefix.

Git linked worktrees that belong to the same common Git repository are separate working directories but one logical Agent Task repository. They resolve to one shared configuration and SQLite database stored in the main worktree's `.tbtm` directory so agents working in different worktrees observe the same identities, tasks, and claims. Linked worktrees locate that directory from trusted Git metadata rather than a user-controlled redirect.

The default prefix is the normalized repository directory name. The user may override it during initialization with a CLI option such as `--prefix`.

Renaming the repository directory after initialization does not change the stored prefix or any existing task ID.

### 7.2 Task types

The MVP supports the following fixed types:

- Epic
- Story
- Task
- Improvement
- Refactor
- Bug
- Spike
- Testing
- Proof of concept (POC)

All types may exist independently without a parent and without dependencies.

### 7.3 Task identity

A task has a stable ID in the following form:

```text
<repository-prefix>-<type-at-creation>-<short-uuid>
```

Example:

```text
agent-task-epic-a1b2c3
```

Requirements:

- The short UUID portion must be unique within the repository.
- A collision must cause regeneration or safe length expansion.
- An existing task ID never changes.
- If task type editing is allowed, the type segment continues to represent the type at creation.

### 7.4 Task fields

Each task contains:

| Field | Requirement |
|---|---|
| ID | Stable generated identifier |
| Title | Required concise title |
| Description | Markdown text |
| Goal | Markdown text describing the desired outcome |
| Acceptance criteria | Markdown text; checklists are supported as content |
| Type | One of the fixed task types |
| Status | One configured repository status |
| Priority | Integer from 0 to 1,000,000; default 50; larger is more important |
| Estimate | Optional number of hours |
| Tags | Zero or more string tags |
| External URLs | Zero or more URLs |
| Code references | Zero or more structured file/code references |
| Parent | Optional single parent subject to hierarchy rules |
| Dependencies | Zero or more upstream task relationships |
| Claim | Optional active claim containing agent and claim time |
| Archived | Boolean lifecycle flag |
| Archive reason | Nullable Markdown reason for the current archived state |
| Created at | Creation timestamp |
| Updated at | Timestamp of the latest task mutation, including archive/unarchive |
| Created by | Agent identity or logical `user` actor |
| Updated by | Agent identity or logical `user` actor |

A code reference contains:

- Repository-relative file path.
- Optional start line.
- Optional end line.
- Optional description.

### 7.5 Status

The default statuses are:

| Code | Display name | Completed |
|---|---|---:|
| `to_do` | Todo | False |
| `in_progress` | In progress | False |
| `done` | Done | True |

Each status contains:

- A stable UUID used for database relationships.
- An immutable, repository-unique machine code matching `[a-z][a-z0-9_]*`.
- Name.
- `completed` boolean.
- Display order for the scrum board.
- Whether it is a default or custom status.

The user can create, rename, reorder, and delete custom statuses. Custom-status creation requires an explicit code; renaming changes only the display name. A status cannot be deleted while any task uses it. Changing `completed` must warn the user that task availability and dependency resolution can change.

A custom status such as `Cancelled` or `Won't fix` may be marked completed.

### 7.6 Effective completion

A task is effectively completed when either condition is true:

```text
task.archived = true
OR
task.status.completed = true
```

Effective completion is used to resolve dependencies. Archive therefore counts as completion even if the task's current status is incomplete.

### 7.7 Hierarchy

Hierarchy represents scope or containment, not execution order.

Rules:

| Child type | Allowed parent type |
|---|---|
| Epic | None |
| Story | Epic |
| Task, improvement, refactor, bug, spike, testing, POC | Epic or story |

Additional rules:

- A task may have at most one parent.
- A parent is optional.
- Hierarchy cycles are forbidden.
- A type or parent change that would violate the hierarchy is rejected.

### 7.8 Dependency

Dependency represents execution blocking. If task A depends on task B, B is upstream of A and A is downstream of B.

Rules:

- Dependencies are many-to-many.
- All dependencies are mandatory; the MVP has no optional dependency type.
- Dependencies may connect any task types.
- Dependency cycles are forbidden.
- A downstream task remains blocked until every upstream task is effectively completed.
- Hierarchy does not imply dependency, and dependency does not imply hierarchy.

### 7.9 Claim

A claim expresses current work ownership by an agent. It is independent of task status.

An active claim contains:

- Agent identity.
- `claimed_at` timestamp.

Rules:

- A task has at most one active claim.
- Claims do not expire automatically.
- Claim creation must be atomic.
- Only an available task can be claimed normally.
- Updating status does not create, release, or transfer a claim.
- Setting a completed status does not automatically unclaim the task.
- An owning agent can unclaim its task explicitly.
- The user can explicitly force-unclaim a task.
- A successful archive automatically removes its active claim. A foreign active claim blocks normal archive; only the logical `user` may explicitly force the archive after confirming the observed claim.

### 7.10 Comment

A task can contain comments from agents and the human user.

Each comment contains:

- Stable comment ID.
- Task ID.
- Content.
- Author actor.
- Creation timestamp.

Rules:

- Comments can be added to active and archived tasks.
- Comments cannot be edited.
- An agent can delete only its own comments.
- The human user can delete any comment.
- A user or agent may add comments with supplemental archive context before or after archive; comments do not replace the task's canonical archive reason.

### 7.11 Archive

Archive removes a task from active planning without deleting its record.

Rules:

- Archive requires a non-empty Markdown reason, sets the archived flag and archive reason, and updates `updated_at` and `updated_by`.
- Normal archive is allowed for an unclaimed task or a task claimed by the invoking agent. A task claimed by another agent is rejected unless the logical `user` explicitly force-confirms the observed claim.
- A successful archive atomically releases any active claim. Force confirmation is bound to the displayed claim identity and claim time so it cannot silently release a replacement claim.
- Archive counts as effective completion and unblocks downstream tasks.
- Archived tasks are excluded from available-task queries and active board views by default.
- Archived tasks remain visible when explicitly requested and remain present in hierarchy and dependency maps.
- Task content and relationships are not changed while archived.
- New comments may be added and permitted comments may be deleted while archived.
- An archived task may be unarchived.
- Unarchiving clears the archive reason, restores completion semantics from the task's current status, and can block downstream work again.
- Before unarchive, the CLI and UI must warn when downstream availability or active claims may be affected.

## 8. Availability and selection rules

### 8.1 Available task definition

A task is available only when all conditions are true:

```text
task.archived = false
AND task.status.completed = false
AND task has no active claim
AND every upstream dependency is effectively completed
```

Task type, hierarchy position, and estimate do not independently affect availability.

### 8.2 Default ordering

Available tasks are ordered deterministically by:

1. Priority descending.
2. Creation time ascending.
3. Task ID ascending.

### 8.3 Query versus claim

The system distinguishes two operations:

- Query available tasks: read-only and informative; results may become stale immediately.
- Claim next available task: selection and claim occur as one atomic operation.

Agents should use the atomic claim operation when acquiring work. A separate query followed by a separate claim is allowed for inspection but does not guarantee ownership.

### 8.4 Availability changes

Availability must be recalculated from current state after relevant changes, including:

- Dependency added or removed.
- Upstream status completion changed.
- Upstream task archived or unarchived.
- Claim created, released, or force-released.
- Task archived or unarchived.
- Status `completed` semantics changed.

## 9. Functional requirements

### FR-1 Repository lifecycle

- Initialize Agent Task in a repository.
- Create one repository-local SQLite database.
- Derive or accept the repository prefix.
- Detect an existing initialization and avoid accidental replacement.
- Optionally add the exact `/.tbtm/` rule to the repository `.gitignore` during initialization.
- Report every filesystem artifact created, modified, or left incomplete when initialization fails.
- Provide an explicit, confirmed uninstall operation that removes repository-local TBTM artifacts and the exact `/.tbtm/` rule without following symlinks or deleting paths outside the repository root.
- Show repository configuration and database health.
- Resolve linked worktrees of one common Git repository to the same task configuration and database without relying on unsafe user-controlled path redirection.

### FR-2 Agent registry

- Register a new agent using a user-supplied base name.
- Generate a UUID and unique display name for every registration.
- List registered agents and their current claims.
- Select or provide an agent identity for agent-scoped CLI operations.

### FR-3 Task management

- Create and view tasks.
- Update task fields while preserving stable task identity.
- Validate type, status, priority, estimate, URLs, and code references.
- Archive and unarchive tasks.
- List active tasks by default and archived tasks on request.
- Filter and sort tasks by relevant task attributes.
- Do not provide permanent task deletion in the MVP.

### FR-4 Hierarchy management

- Assign, change, and remove a parent.
- Validate the type matrix and one-parent limit.
- Reject hierarchy cycles.
- View parent, direct children, and recursive descendants.

### FR-5 Dependency management

- Add and remove dependency relationships.
- Reject self-dependency, duplicates, and cycles.
- Recompute blocking and availability from the current graph.
- Display upstream, downstream, or combined dependency views.

### FR-6 Status management

- Seed the three default statuses.
- Create, rename, reorder, and delete custom statuses.
- Set and change completed semantics.
- Prevent deletion of a status in use.
- Warn before a semantics change that affects dependencies or availability.

### FR-7 Availability query

- List only tasks satisfying the complete availability predicate.
- Apply deterministic default ordering.
- Support agent-relevant filters without weakening availability rules.
- Explain why an unavailable task is blocked when requested.

### FR-8 Claim management

- Atomically claim a specified available task.
- Atomically select and claim the next available task.
- Return a distinct conflict result when another agent wins the claim race.
- Allow the owner to unclaim.
- Allow the user to force-unclaim.
- Preserve `claimed_at` until claim removal.
- Display current claim owner and age.

### FR-9 Comments

- Add comments to active and archived tasks.
- List comments chronologically with author and timestamp.
- Delete comments according to actor ownership rules.
- Reject comment editing.

### FR-10 Relationship map

For any task, support these views:

- `upstream`: recursive dependencies required by the task.
- `downstream`: recursive tasks that depend on the task.
- `parent`: parent and ancestor context.
- `child`: direct children and recursive descendants.
- `all`: combined hierarchy and dependency context.

CLI output must distinguish relationship type, direction, completion, archive, blocking, and claim state. The Visual Studio Code UI must provide an interactive graph or equivalent navigable map.

### FR-11 CLI experience

- Cover every core repository, agent, task, status, relationship, claim, comment, archive, and query operation.
- Provide human-readable output by default.
- Provide stable machine-readable JSON output for query and mutation results.
- Use documented exit codes for success, validation failure, not found, claim conflict, permission failure, and unexpected error.
- Avoid interactive prompts in agent-oriented operation when explicit options are provided.
- Make destructive or high-impact actions such as force-unclaim, unarchive, force initialization, and repository uninstall explicit.

The PRD defines command capabilities, not final command names or flag syntax.

### FR-12 Visual Studio Code extension

The extension must provide:

- Repository initialization or connection state.
- Scrum board with status columns in configured order.
- Task cards showing at least ID, type, title, priority, claim owner, and blocked state.
- Drag-and-drop status changes with validation.
- Task creation, viewing, and editing.
- Archive and unarchive actions.
- Claim, unclaim, and force-unclaim actions.
- Comment list, creation, and permitted deletion.
- Parent and dependency editing.
- Dependency/hierarchy map in all supported directions.
- Filters for type, status, priority, tags, claim owner, archived state, and availability.
- Status configuration and ordering.
- Agent registry and claim visibility.
- Clear refresh behavior when another local process changes the database.

### FR-13 Actor metadata

- Task creation and updates record the responsible actor.
- Agent-scoped actions use the registered agent UUID.
- Human operations use the logical `user` actor.
- Actor rules are consistency guardrails, not authentication guarantees.

## 10. Key workflows

### 10.1 Agent registration and work acquisition

1. Agent registers a base name in the repository.
2. System returns its UUID and unique display name.
3. Agent requests the next available task with optional filters.
4. System calculates availability and ordering inside the claim operation.
5. System atomically claims the selected task.
6. Agent receives task data and relationship context as JSON.
7. If no task is available, the CLI returns a successful empty result distinct from an error.
8. If a claim race is lost for a specified task, the CLI returns a claim-conflict result.

### 10.2 Agent work lifecycle

1. Agent claims a task.
2. Agent independently changes status when appropriate.
3. Agent adds progress or decision comments.
4. Agent may update task context or references.
5. Agent sets a completed status when work is complete.
6. Agent explicitly unclaims the task.

Status completion and unclaim remain separate operations even when agents commonly perform them consecutively.

### 10.3 Dependency resolution

1. A downstream task is incomplete and unclaimed but has at least one incomplete upstream dependency.
2. The downstream task is not available.
3. The final incomplete upstream task becomes effectively completed through status or archive.
4. The downstream task becomes available if all other availability conditions are true.

### 10.4 Archive and unarchive

1. User or agent requests archive with a reason.
2. A foreign active claim blocks normal archive; the logical `user` may explicitly confirm a force override bound to the observed claim.
3. System atomically releases any permitted active claim, stores the reason, marks the task archived, and updates actor metadata.
4. Downstream availability is recalculated; archive counts as completion.
5. User or agent can add a supplemental comment about the archive.
6. If the user later requests unarchive, the system evaluates downstream impact.
7. System warns if the task's incomplete status will block downstream tasks, especially already claimed tasks.
8. After confirmation, the task is unarchived, its archive reason is cleared, and availability is recalculated.

### 10.5 Concurrent claim

1. Two agents request the same available task at nearly the same time.
2. Exactly one claim succeeds.
3. The database contains exactly one owner and one `claimed_at` value.
4. The losing operation receives a deterministic conflict response and does not overwrite the winner.

## 11. Validation and edge-case rules

- Reject unknown task types and statuses.
- Reject priority outside 0 to 1,000,000.
- Reject negative estimates.
- Reject malformed external URLs and invalid code-reference line ranges.
- Reject a story parent that is not an epic.
- Reject a leaf task parent that is not an epic or story.
- Reject any parent for an epic.
- Reject hierarchy and dependency cycles before state mutation.
- Reject claiming archived, completed, claimed, or dependency-blocked tasks.
- Reject normal unclaim by a non-owner.
- Reject agent deletion of another actor's comment.
- Reject comment editing for every actor.
- Reject status deletion while in use.
- Preserve archived tasks when showing graph context.
- Treat archived upstream tasks as resolved dependencies.
- Warn before unarchive can make downstream work blocked.
- Do not silently alter an existing active claim when a status changes.

## 12. Personal-project non-functional requirements

### 12.1 Data integrity

- Repository task content is persisted in SQLite.
- Claim uniqueness and graph validity must remain correct under concurrent local processes.
- Mutations contained within SQLite must either fully succeed or leave prior database state intact.
- Repository filesystem operations such as initialization, force initialization, stealth configuration, and uninstall are best-effort. Interruption or I/O failure may leave partial filesystem artifacts.
- A failed repository filesystem operation must report the affected artifacts and provide an explicit cleanup path. Failed initialization directs the user to `tbtm uninstall`.
- Application interruption must not leave partial claims or partial relationship changes.

### 12.2 Performance

- Common task list, available query, and claim operations should feel immediate for a personal repository.
- Dependency traversal should remain usable for repositories containing thousands of tasks.
- The extension must not block the Visual Studio Code UI while loading or refreshing data.

No enterprise-scale throughput target is required for the MVP.

### 12.3 Portability

- The CLI and extension should work on the major desktop platforms supported by Visual Studio Code.
- Code references use repository-relative paths where possible.
- The repository database can be backed up by copying it while the application is not writing, or through a documented safe backup operation.

### 12.4 Usability and observability

- Errors must state what failed and, when possible, how to resolve it.
- Blocking explanations must identify unresolved upstream task IDs.
- Claim conflicts must identify the current owner when available.
- Human-readable CLI output and UI must distinguish blocked, claimed, completed, and archived states.

## 13. MVP acceptance criteria

The MVP is accepted when all of the following are demonstrably true:

1. A repository can be initialized and reopened without losing configuration.
2. All default types and statuses are available.
3. Tasks can be created with all defined fields and stable IDs.
4. Hierarchy validation enforces the type matrix and rejects cycles.
5. Dependencies support many-to-many relationships and reject cycles.
6. Available-task query implements the exact availability predicate and ordering.
7. Concurrent claim testing proves that only one agent can claim a task.
8. Claim, unclaim, and force-unclaim follow ownership rules.
9. Status completion and archive both resolve dependencies as specified.
10. Status completion does not implicitly unclaim, while archive does.
11. Unarchive recalculates blocking and warns about affected downstream work.
12. Comments can be added to archived tasks, cannot be edited, and can be deleted only according to ownership rules.
13. CLI supports both human-readable and JSON output with stable exit behavior.
14. Relationship maps support upstream, downstream, parent, child, and all modes.
15. Visual Studio Code provides the complete board and task-management scope defined in FR-12.
16. Two local processes can read and mutate repository state without corrupting the database.
17. Failed initialization identifies partial artifacts, and under normal local operation `tbtm uninstall` can preview and remove repository-local TBTM artifacts without following symlinks or traversing outside the repository root.
18. Agents operating from linked worktrees of the same Git repository observe one shared repository identity, task database, and claim state.

## 14. Epic and story backlog

The backlog below defines product-level stories. Technical implementation tasks and estimates should be created during detailed planning after architecture decisions are recorded.

Every `Dependencies` entry in this section is a **contract dependency**: the
referenced story contract must be planned and approved before planning of the
dependent story can finish. It does not by itself require the referenced
implementation task to be complete. Implementation ordering is recorded
separately by task-level `depends_on`. An epic identifier in the prose below is
shorthand for every story currently listed under that epic; `docs/STATUS.md`
expands those shorthands to explicit story IDs for planning coordination.

### Epic E1: Repository foundation and identity

**Outcome:** A repository has a stable local task store, configuration, and actor identities.

#### Story E1-S1: Initialize a repository

As a user, I want to initialize Agent Task in a repository so that the repository has an independent task workspace.

Acceptance criteria:

- Initialization creates one SQLite task store and repository configuration.
- Default prefix comes from the normalized repository directory name.
- A custom prefix can be supplied.
- Re-initialization does not silently replace existing data.
- Default statuses are created.
- Optional stealth initialization adds the exact `/.tbtm/` rule while preserving existing `.gitignore` bytes as a prefix; it may add one line terminator when the existing final line is unterminated.
- A retry validates the artifacts that exist: an incomplete or mismatched config/DB pair is invalid, while a matching config/DB pair remains a valid workspace even when a prior stealth update failed.
- Repository uninstall supports confirmation, dry-run, structured output, and bounded removal of TBTM artifacts.

Dependencies: None.

#### Story E1-S2: Resolve repository configuration

As a CLI or extension client, I want to discover the repository task configuration so that commands target the correct database.

Acceptance criteria:

- Configuration can be discovered from within the repository workspace.
- Stored prefix is stable after directory rename.
- Configuration and database errors are distinguishable.

Dependencies: E1-S1.

#### Story E1-S3: Register an agent

As a coding agent, I want to register a base name so that my actions and claims have a stable identity.

Acceptance criteria:

- Every registration generates a new UUID.
- Reusing a base name creates a distinct identity.
- Display name combines base name and UUID-derived suffix.
- Registration result is available as JSON.

Dependencies: E1-S1.

#### Story E1-S4: List agents and claims

As a user, I want to list agents and their claims so that I can understand current ownership.

Acceptance criteria:

- Registered identity, display name, and current claimed tasks are visible.
- Agents with no claims remain discoverable.
- Human and JSON output are supported.

Dependencies: E1-S3, E5-S1.

#### Story E1-S5: Share repository state across Git worktrees

As a coding agent working in a Git linked worktree, I want every worktree of the same repository to resolve one shared Agent Task database so that agents coordinate identities, tasks, and claims across isolated source-code workspaces.

Acceptance criteria:

- The main worktree and every linked worktree resolve the same logical repository identity, configuration, and SQLite database.
- Non-Git directories and repositories without linked worktrees retain the existing repository-local behavior.
- Shared-state discovery is derived from trusted Git repository metadata rather than arbitrary database paths from configuration.
- Normal worktree use does not require symlinking `.tbtm` or the SQLite database.
- Concurrent commands from different worktrees preserve SQLite integrity and atomic claim behavior.
- Missing or inaccessible common Git metadata, configuration, or database paths produce the existing actionable repository, database, and permission error categories.
- This pre-release MVP defines only the canonical single-store layout. Independently initialized per-worktree stores are never selected, merged, migrated, or removed automatically.
- Repository status from any worktree remains read-only and does not create persistent SQLite journal, WAL, or shared-memory artifacts as a side effect.

Dependencies: E1-S2.

### Epic E2: Task content and lifecycle

**Outcome:** Users and agents can create, understand, update, archive, and restore coding tasks.

#### Story E2-S1: Create a task

As a user or agent, I want to create a typed task with coding context so that the work is actionable.

Acceptance criteria:

- Required and optional core fields are accepted and validated.
- Priority defaults to 50.
- Task receives a collision-safe stable ID.
- Created and updated actor metadata is recorded.
- Status selection uses an immutable machine code and defaults to `to_do`.

Dependencies: E1-S1, E1-S3, E3-S1.

#### Story E2-S2: View and list tasks

As a user or agent, I want to view task details and task lists so that I can inspect repository work.

Acceptance criteria:

- Full task context, status, claim, hierarchy, and dependency summary are visible.
- Active tasks are shown by default.
- Archived tasks can be explicitly included.
- Human and JSON output are supported.

Dependencies: E2-S1.

#### Story E2-S3: Update task content

As a user or agent, I want to update task context so that it remains accurate during implementation.

Acceptance criteria:

- Mutable fields can be changed with validation.
- Task ID never changes.
- A type change is rejected if it would make the task's current parent or direct children invalid; assigning, changing, and removing a parent is owned by E4-S1.
- Updated actor and timestamp are recorded.

Dependencies: E2-S1, E4-S1.

#### Story E2-S4: Manage tags, URLs, estimates, and code references

As a coding agent, I want structured implementation context so that I can locate relevant code and understand effort.

Acceptance criteria:

- Multiple tags and URLs are supported.
- Estimate is expressed in hours and cannot be negative.
- File references support optional line range and description.
- Invalid URLs and line ranges are rejected.

Dependencies: E2-S1.

#### Story E2-S5: Archive a task

As a user or agent, I want to archive abandoned or obsolete work so that it leaves active planning without blocking downstream tasks.

Acceptance criteria:

- Archive requires and stores a non-empty Markdown reason and updates task actor metadata.
- An unclaimed task or a task claimed by the invoking agent can be archived normally.
- A foreign active claim blocks archive unless the logical `user` explicitly confirms a force override of that observed claim.
- Any active claim is released atomically on successful archive.
- Archived task is effectively completed.
- Downstream availability is recalculated.
- Task remains visible in explicit archive and graph views.
- Status, content, hierarchy, and dependency relationships remain unchanged.

Dependencies: E2-S1, E4-S3, E5-S3.

#### Story E2-S6: Unarchive a task safely

As a user, I want to restore an archived task so that work can return to active planning with clear downstream impact.

Acceptance criteria:

- System evaluates the current status and downstream impact.
- Only the logical user can unarchive a task.
- User is warned when active, incomplete direct downstream tasks gain the restored task as an unresolved dependency, including any task with an active claim.
- Human confirmation or explicit non-interactive confirmation is required when impact exists.
- Confirmed unarchive recalculates availability.
- Task remains unclaimed after unarchive.
- The current archive reason is cleared.

Dependencies: E2-S5, E4-S4.

### Epic E3: Status workflow

**Outcome:** Repositories have a simple configurable workflow with explicit completion semantics.

#### Story E3-S1: Use default statuses

As a user, I want todo, in-progress, and done statuses so that a new repository is immediately usable.

Acceptance criteria:

- Default statuses exist in board order with codes `to_do`, `in_progress`, and `done`.
- Todo and in-progress are incomplete.
- Done is completed.
- Before the first released repository format, the baseline schema may be updated directly; compatibility migration and backfill of earlier development-only databases are not required.

Dependencies: E1-S1.

#### Story E3-S2: Create and organize custom statuses

As a user, I want custom ordered statuses so that the board matches my workflow.

Acceptance criteria:

- A status has an immutable unique code, display name, completed value, and display order.
- Custom statuses can be created and renamed; both default and custom statuses can be reordered.
- Custom-status creation requires an explicit code; rename changes only the display name.
- Names are validated for repository-level uniqueness.

Dependencies: E3-S1.

#### Story E3-S3: Change status completion semantics

As a user, I want to mark a status completed or incomplete so that dependency behavior matches its meaning.

Acceptance criteria:

- Change impact is calculated before confirmation.
- User receives a warning about affected availability.
- Confirmed changes immediately affect effective completion and availability.
- Active claims are not automatically changed.

Dependencies: E3-S2, E4-S3.

#### Story E3-S4: Delete an unused custom status

As a user, I want to remove an unused custom status so that obsolete board columns disappear safely.

Acceptance criteria:

- Status in use cannot be deleted.
- Default status deletion is rejected in the MVP.
- Deleting an unused custom status preserves task data.

Dependencies: E3-S2.

### Epic E4: Hierarchy, dependencies, and availability

**Outcome:** The system represents planning structure and returns only executable work.

#### Story E4-S1: Manage task hierarchy

As a user or agent, I want to assign valid parents so that epics and stories organize work.

Acceptance criteria:

- Parent assignment follows the hierarchy type matrix.
- Parent is optional and singular.
- Invalid parent types and cycles are rejected.
- Parent can be changed or removed from an active task.

Dependencies: E2-S1.

#### Story E4-S2: Manage dependencies

As a user or agent, I want to add and remove mandatory dependencies so that execution order is explicit.

Acceptance criteria:

- Any task types may be connected.
- Duplicate, self, and cyclic dependencies are rejected.
- All dependencies are blocking until effectively completed.
- Relationship changes update blocking results immediately.

Dependencies: E2-S1.

#### Story E4-S3: Query available tasks

As a coding agent, I want to query unblocked and unclaimed tasks so that I do not select unavailable work.

Acceptance criteria:

- Exact availability predicate is applied.
- Results use priority, creation time, and ID ordering.
- Supported filters cannot admit a blocked, completed, archived, or claimed task.
- Empty result is distinct from failure.

Dependencies: E3-S1, E4-S2, E5-S1.

#### Story E4-S4: Explain blocking

As a user or agent, I want to know why a task is unavailable so that I can resolve its blockers.

Acceptance criteria:

- Explanation distinguishes archive, completion, claim, and dependency blocking.
- Every unresolved upstream dependency is identifiable.
- Recursive context is available without treating hierarchy as dependency.

Dependencies: E4-S2, E4-S3.

#### Story E4-S5: View relationship maps

As a user or agent, I want directional relationship maps so that I can understand a task in context.

Acceptance criteria:

- Upstream, downstream, parent, child, and all modes are supported.
- Recursive results do not loop or duplicate nodes ambiguously.
- Archived and completed nodes remain visible with their state.
- Relationship type and direction are explicit in JSON output.

Dependencies: E4-S1, E4-S2.

### Epic E5: Multi-agent claiming

**Outcome:** Multiple coding agents can coordinate ownership without claiming the same task.

#### Story E5-S1: Claim a specified task atomically

As a coding agent, I want to atomically claim an available task so that another agent cannot claim it simultaneously.

Acceptance criteria:

- Claim succeeds only if the task is currently available.
- Exactly one active claim can exist per task.
- Claim records agent and claimed time.
- Losing a race returns a deterministic conflict and never overwrites the winner.

Dependencies: E1-S3, E2-S1, E3-S1, E4-S2.

#### Story E5-S2: Claim the next available task atomically

As a coding agent, I want selection and claim in one operation so that a prior availability query cannot become stale.

Acceptance criteria:

- Selection uses the exact availability predicate.
- Selection uses deterministic priority ordering.
- Selection and claim are one atomic outcome.
- Optional filters are supported without weakening availability.

Dependencies: E4-S3, E5-S1.

#### Story E5-S3: Unclaim owned work

As a coding agent, I want to release my claim so that another agent can acquire the task.

Acceptance criteria:

- Owner can unclaim its task.
- Non-owner normal unclaim is rejected.
- Task availability is recalculated after release.
- Status is not changed.

Dependencies: E5-S1.

#### Story E5-S4: Force-unclaim stale work

As a user, I want to force-release a claim so that a crashed or abandoned agent cannot hold work forever.

Acceptance criteria:

- Operation is explicit and identifies current owner and claim time.
- Force-unclaim removes only the claim.
- Task status remains unchanged.
- Updated availability is returned.

Dependencies: E5-S1.

#### Story E5-S5: Preserve claim/status independence

As an agent, I want claim ownership and workflow status to remain independent so that I control each transition explicitly.

Acceptance criteria:

- Claim does not change status.
- Status change does not create or release a claim.
- Completed tasks may retain their active claim until explicit release.
- Archived tasks are the documented exception and release claims.

Dependencies: E3-S1, E5-S1, E5-S3, E2-S5.

### Epic E6: Task comments and collaboration context

**Outcome:** Humans and agents can leave durable task context with simple ownership rules.

#### Story E6-S1: Add and view comments

As a user or agent, I want to comment on a task so that decisions and progress are retained.

Acceptance criteria:

- Comment records author and creation time.
- Comments are listed chronologically.
- Active and archived tasks accept new comments.
- Comment data is available in human and JSON output.

Dependencies: E1-S3, E2-S1.

#### Story E6-S2: Delete a comment under ownership rules

As an agent, I want to delete my own comment, and as a user I want to delete any comment, so that incorrect or unwanted content can be removed.

Acceptance criteria:

- Agent can delete its own comment.
- Agent cannot delete another actor's comment.
- User can delete any comment.
- The same rules apply to archived tasks.

Dependencies: E6-S1.

#### Story E6-S3: Keep comments immutable

As a reader, I want existing comments not to change so that previously read context is not silently rewritten.

Acceptance criteria:

- No CLI or UI operation edits comment content.
- Correction is performed by deletion and creation of a new comment.

Dependencies: E6-S1, E6-S2.

### Epic E7: Agent-first CLI

**Outcome:** Every core capability is accessible reliably to coding agents and humans from the terminal.

#### Story E7-S1: Provide consistent command output

As a coding agent, I want stable JSON and exit behavior so that I can invoke commands programmatically.

Acceptance criteria:

- Query and mutation commands support JSON.
- Human-readable output is the default.
- Success, empty result, validation error, not found, claim conflict, permission failure, and unexpected error are distinguishable.
- Agent-oriented commands can run non-interactively.

Dependencies: E1-S1.

#### Story E7-S2: Expose repository, agent, and task operations

As a user or agent, I want core entity operations in the CLI so that the complete workflow is terminal-accessible.

Acceptance criteria:

- Repository initialization and inspection are covered.
- Agent registration and listing are covered.
- Task create, view, list, update, archive, and unarchive are covered.
- Actor identity is explicit for scoped operations.

Dependencies: E1-S1, E1-S2, E1-S3, E1-S4, E1-S5, E2-S1, E2-S2, E2-S3, E2-S4, E2-S5, E2-S6, E7-S1.

#### Story E7-S3: Expose planning relationships and status operations

As a user or agent, I want hierarchy, dependency, map, and status commands so that planning can be managed without the UI.

Acceptance criteria:

- Parent and dependency relationships can be added, removed, and queried.
- All five relationship map modes are available.
- Status configuration operations are available.
- Impact warnings can be confirmed explicitly or rejected non-interactively.

Dependencies: E3-S1, E3-S2, E3-S3, E3-S4, E4-S1, E4-S2, E4-S3, E4-S4, E4-S5, E7-S1.

#### Story E7-S4: Expose availability and claim operations

As a coding agent, I want dedicated availability and atomic claim commands so that work acquisition is safe.

Acceptance criteria:

- Available list and blocking explanation are supported.
- Specified claim and claim-next are distinct operations.
- Unclaim and explicit force-unclaim are supported.
- Claim conflicts have stable machine-readable results.

Dependencies: E4-S3, E4-S4, E5-S1, E5-S2, E5-S3, E5-S4, E5-S5, E7-S1.

#### Story E7-S5: Expose comment operations

As a user or agent, I want comment commands so that task context can be managed from the terminal.

Acceptance criteria:

- Add, list, and permitted delete operations are supported.
- No edit operation is exposed.
- Archived-task comments follow the same command flow.

Dependencies: E6-S1, E6-S2, E6-S3, E7-S1.

### Epic E8: Visual Studio Code scrum experience

**Outcome:** The human user can plan, supervise, and manage all repository work inside Visual Studio Code.

#### Story E8-S1: Connect the extension to repository state

As a user, I want the extension to detect Agent Task configuration so that it opens the correct repository board.

Acceptance criteria:

- Initialized and uninitialized states are clear.
- Database or configuration failures are actionable.
- External CLI changes can be refreshed without restarting Visual Studio Code.

Dependencies: E1-S2, E7-S1.

#### Story E8-S2: View and filter the scrum board

As a user, I want an ordered status board so that I can understand work at a glance.

Acceptance criteria:

- Columns follow configured status order.
- Cards show required summary fields and visual states.
- Filters cover type, status, priority, tags, owner, archive, and availability.
- Archived tasks are excluded by default.

Dependencies: E2-S2, E3-S2, E4-S3, E5-S1, E8-S1.

#### Story E8-S3: Change status using the board

As a user, I want to move cards between status columns so that workflow state is easy to update.

Acceptance criteria:

- Drag-and-drop changes only task status.
- Claim remains unchanged.
- Completion changes recalculate downstream availability.
- Validation or impact failures are displayed clearly.

Dependencies: E3-S3, E8-S2.

#### Story E8-S4: Manage task details

As a user, I want a complete task editor so that I can manage all task context in Visual Studio Code.

Acceptance criteria:

- All core task fields can be viewed and valid mutable fields updated.
- Parent, dependencies, tags, URLs, and code references are manageable.
- Archive and unarchive are available with required warnings.
- Created/updated and claim metadata are visible.

Dependencies: E2-S1, E2-S2, E2-S3, E2-S4, E2-S5, E2-S6, E4-S1, E4-S2, E8-S1.

#### Story E8-S5: Manage claims from the UI

As a user, I want to view and release claims so that I can recover abandoned work.

Acceptance criteria:

- Owner and claimed time are visible.
- Claim and unclaim actions follow ordinary rules.
- Force-unclaim is explicit and confirmed.
- Status does not change as a side effect.

Dependencies: E5-S1, E5-S2, E5-S3, E5-S4, E5-S5, E8-S4.

#### Story E8-S6: Manage comments from the UI

As a user, I want to add and delete task comments so that I can record decisions and remove unwanted content.

Acceptance criteria:

- Comments show author and time.
- User can add to active and archived tasks.
- User can delete any comment.
- Editing is unavailable.

Dependencies: E6-S1, E6-S2, E6-S3, E8-S4.

#### Story E8-S7: Explore relationship maps

As a user, I want an interactive task map so that I can explore blocking and hierarchy visually.

Acceptance criteria:

- All five map modes are selectable.
- Nodes distinguish type, status, archive, claim, and blocking state.
- Relationship direction and kind are visually distinct.
- Selecting a node opens its task detail.

Dependencies: E4-S5, E8-S4.

#### Story E8-S8: Configure statuses and inspect agents

As a user, I want to manage board columns and inspect agents so that I can control workflow and ownership from Visual Studio Code.

Acceptance criteria:

- Custom statuses can be created, renamed, reordered, and safely deleted.
- Completion-semantic changes show impact warnings.
- Registered agents and their claims are visible.

Dependencies: E1-S4, E3-S1, E3-S2, E3-S3, E3-S4, E8-S1.

### Epic E9: Reliability, verification, and product guidance

**Outcome:** The personal project is safe to use concurrently and understandable to future users and agents.

#### Story E9-S1: Verify concurrent claim safety

As a user, I want claim concurrency verified so that duplicate agent work is prevented.

Acceptance criteria:

- Automated concurrent tests repeatedly produce one winner.
- Losing operations return claim conflict.
- No test produces duplicate active claims or database corruption.

Dependencies: E1-S5, E5-S1, E5-S2.

#### Story E9-S2: Verify graph and availability invariants

As a user, I want graph and availability rules tested so that agents do not receive blocked work.

Acceptance criteria:

- Hierarchy and dependency cycles are rejected.
- Multi-level dependencies resolve correctly.
- Archive, unarchive, and status-semantics changes recalculate availability correctly.
- Deterministic ordering is verified.

Dependencies: E3-S3, E4-S1, E4-S2, E4-S3, E4-S4, E4-S5, E2-S5, E2-S6.

#### Story E9-S3: Verify data recovery behavior

As a user, I want interrupted mutations to remain safe so that local task data is trustworthy.

Acceptance criteria:

- Interrupted claim and relationship mutations do not leave partial state.
- A documented safe backup approach exists.
- Common configuration and database failure messages are actionable.

Dependencies: E1-S1, E4-S2, E5-S1.

#### Story E9-S4: Document agent and human workflows

As a user, I want concise product guidance so that humans and coding agents use the safe workflows.

Acceptance criteria:

- Initialization and agent registration are documented.
- Query-versus-atomic-claim distinction is explicit.
- Claim, status, archive, and comment semantics are documented.
- JSON output and exit behavior are documented for agents.

Dependencies: E7-S1, E7-S2, E7-S3, E7-S4, E7-S5, E8-S1, E8-S2, E8-S3, E8-S4, E8-S5, E8-S6, E8-S7, E8-S8.

## 15. Suggested delivery sequence

The stories should not be implemented strictly epic-by-epic. A dependency-oriented sequence reduces rework:

1. **Foundation:** E1-S1, E1-S2, E1-S3, E3-S1.
2. **Minimal task core:** E2-S1, E2-S2, E2-S3.
3. **Graph core:** E4-S1, E4-S2.
4. **Shared multi-worktree foundation and claim core:** E1-S5, E5-S1, E5-S3, E5-S4.
5. **Availability and atomic acquisition:** E4-S3, E4-S4, E5-S2.
6. **Lifecycle completion:** E2-S5, E2-S6, E5-S5, E3-S2 through E3-S4.
7. **Task context:** E2-S4 and E6.
8. **Complete CLI:** E7.
9. **Relationship map:** E4-S5.
10. **Visual Studio Code experience:** E8.
11. **Reliability and documentation:** E9, with concurrency and invariant tests started alongside their corresponding core stories.

## 16. Planning guidance for the next phase

For detailed planning, each story should be decomposed into technical tasks only after the following solution decisions are made:

- CLI packaging and distribution model.
- Database access boundary shared by CLI and extension.
- Canonical shared-state location and upgrade policy for Git linked worktrees.
- Database migration strategy.
- Transaction and local concurrency strategy.
- JSON response envelope and exit-code contract.
- Visual Studio Code extension architecture and refresh mechanism.
- Graph-query and graph-rendering approach.
- Test strategy for concurrent processes and state transitions.

These are solution-design decisions, not unresolved product requirements. They should not change the domain semantics defined in this PRD.

## 17. Future considerations

The following ideas are intentionally outside MVP but remain compatible with the model:

- Optional claim expiry or lease renewal.
- Claim history and activity feed.
- Automated stale-claim suggestions based on `claimed_at`.
- External issue-tracker import/export.
- Remote synchronization.
- Multiple human users and authentication.
- Custom task types and configurable hierarchy matrices.
- Agent capability matching during task selection.
- Sprint, milestone, or release concepts.
- Automated dependency-impact notifications.
