use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, IsTerminal, Write},
    process::ExitCode,
};
use tbtm_core::{
    AgentRegistration, Error, RepositoryHealth,
    agent::{AgentList, list_agents},
    comment::{
        AddCommentInput, DeleteCommentInput, TaskComment, add_comment, delete_comment,
        list_comments,
    },
    exit_code, initialize, inspect_repository_health, register_agent,
    status::{
        CreateStatusInput, MoveStatusResult, Placement, SetCompletedInput, SetCompletedResult,
        Status, StatusCompletionImpact, create_status, delete_status, list_statuses, move_status,
        preview_set_completed, rename_status, set_completed,
    },
    task::{
        ArchiveScope, ArchiveTaskInput, AvailabilityReason, AvailableTasksInput,
        ClaimNextTaskInput, ClaimTaskInput, CreateTaskInput, CreatedTask, DependencyInput,
        DependencyResult, ForceUnclaimTaskInput, ForceUnclaimTaskResult, FullTask, HierarchyResult,
        ListTasksInput, ObservedClaim, ParentMutationResult, ParentRemoveInput, ParentSetInput,
        PatchValue, RelationshipDirection, RelationshipEdge, RelationshipMap, RelationshipNode,
        TaskBlockingExplanation, TaskListItem, TaskType, UnarchiveImpact, UnarchiveTaskInput,
        UnarchiveTaskResult, UnclaimTaskInput, UnclaimTaskResult, UpdateTaskInput, add_dependency,
        archive_task, available_tasks, claim_next_task, claim_task, create_task,
        explain_task_blocking, force_unclaim_task, list_tasks, parse_code_reference,
        preview_unarchive, remove_dependency, remove_parent, set_parent, task_hierarchy,
        task_relationship_map, unarchive_task, unclaim_task, update_task, view_task,
    },
    uninstall,
};

#[derive(Parser)]
#[command(name = "tbtm", version, about = "Repository-local task management")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(
        about = "Create repository-local .tbtm workspace",
        long_about = "Create .tbtm/config.json and .tbtm/tbtm.db in the canonical main worktree. If initialization partially fails, inspect reported artifacts and run `tbtm uninstall` before retrying."
    )]
    Init(InitArgs),
    #[command(
        about = "Remove repository-local TBTM artifacts",
        long_about = "Remove .tbtm, direct-root TBTM backups and staging directories, plus exact /.tbtm/ rules from the canonical main worktree. This is destructive; use --dry-run to inspect targets first."
    )]
    Uninstall(UninstallArgs),
    #[command(about = "Inspect repository configuration and health")]
    Repo(RepoArgs),
    #[command(about = "Manage repository-local agent identities")]
    Agent(AgentArgs),
    #[command(about = "Manage repository-local tasks")]
    Task(Box<TaskArgs>),
    #[command(about = "Manage repository workflow statuses")]
    Status(StatusArgs),
}

#[derive(Args)]
struct StatusArgs {
    #[command(subcommand)]
    command: StatusCommand,
}

#[derive(Subcommand)]
enum StatusCommand {
    #[command(about = "List statuses in board order")]
    List(StatusListArgs),
    #[command(about = "Create a custom status")]
    Create(StatusCreateArgs),
    #[command(about = "Rename a custom status")]
    Rename(StatusRenameArgs),
    #[command(about = "Move a status in board order")]
    Move(StatusMoveArgs),
    #[command(about = "Delete an unused custom status")]
    Delete(StatusDeleteArgs),
    #[command(about = "Change whether a status represents completed work")]
    SetCompleted(StatusSetCompletedArgs),
}

#[derive(Args)]
struct StatusListArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct StatusCreateArgs {
    #[arg(long)]
    code: String,
    #[arg(long)]
    name: String,
    #[arg(long, conflicts_with = "after")]
    before: Option<String>,
    #[arg(long, conflicts_with = "before")]
    after: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct StatusRenameArgs {
    code: String,
    #[arg(long)]
    name: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct StatusDeleteArgs {
    code: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
#[command(group(clap::ArgGroup::new("placement").required(true).args(["before", "after"])))]
struct StatusMoveArgs {
    code: String,
    #[arg(long, conflicts_with = "after")]
    before: Option<String>,
    #[arg(long, conflicts_with = "before")]
    after: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct StatusSetCompletedArgs {
    code: String,
    #[arg(long, action = clap::ArgAction::Set)]
    completed: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommand,
}

#[derive(Subcommand)]
enum TaskCommand {
    #[command(about = "Create a task")]
    Create(TaskCreateArgs),
    #[command(about = "View a task")]
    View(TaskViewArgs),
    #[command(about = "List tasks")]
    List(TaskListArgs),
    #[command(about = "List currently available tasks")]
    Available(TaskAvailableArgs),
    #[command(about = "Explain why a task is unavailable")]
    Blockers(TaskBlockersArgs),
    #[command(about = "Update a task")]
    Update(TaskUpdateArgs),
    #[command(about = "Manage task dependencies")]
    Dependency(DependencyArgs),
    #[command(about = "Manage a task's parent")]
    Parent(ParentArgs),
    #[command(about = "View task hierarchy")]
    Hierarchy(TaskHierarchyArgs),
    #[command(about = "View recursive task relationships")]
    Map(TaskMapArgs),
    #[command(about = "Atomically claim an available task")]
    Claim(TaskClaimArgs),
    #[command(about = "Atomically claim the next available task")]
    ClaimNext(TaskClaimNextArgs),
    #[command(about = "Release an owned task claim")]
    Unclaim(TaskUnclaimArgs),
    #[command(about = "Archive a task with a durable reason")]
    Archive(TaskArchiveArgs),
    #[command(about = "Return an archived task to active planning")]
    Unarchive(TaskUnarchiveArgs),
    #[command(about = "Add, list, and delete task comments")]
    Comment(TaskCommentArgs),
}

#[derive(Args)]
struct TaskCommentArgs {
    #[command(subcommand)]
    command: TaskCommentCommand,
}

#[derive(Subcommand)]
enum TaskCommentCommand {
    #[command(about = "Add a comment to a task")]
    Add(TaskCommentAddArgs),
    #[command(about = "List a task's comments chronologically")]
    List(TaskCommentListArgs),
    #[command(about = "Delete a task comment under the selected actor identity")]
    Delete(TaskCommentDeleteArgs),
}

#[derive(Args)]
struct TaskCommentAddArgs {
    task_id: String,
    #[arg(long)]
    content: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskCommentListArgs {
    task_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskCommentDeleteArgs {
    task_id: String,
    comment_id: uuid::Uuid,
    #[arg(
        long,
        help = "Act as this registered agent; omission selects logical-user authority"
    )]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskBlockersArgs {
    task_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskArchiveArgs {
    id: String,
    #[arg(long)]
    reason: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskUnarchiveArgs {
    id: String,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskClaimArgs {
    task_id: String,
    #[arg(long)]
    agent: uuid::Uuid,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskClaimNextArgs {
    #[arg(long)]
    agent: uuid::Uuid,
    #[arg(long = "status")]
    statuses: Vec<String>,
    #[arg(long = "type")]
    task_types: Vec<String>,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskUnclaimArgs {
    task_id: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DependencyArgs {
    #[command(subcommand)]
    command: DependencyCommand,
}

#[derive(Subcommand)]
enum DependencyCommand {
    #[command(about = "Add a mandatory upstream dependency")]
    Add(DependencyMutationArgs),
    #[command(about = "Remove a mandatory upstream dependency")]
    Remove(DependencyMutationArgs),
}

#[derive(Args)]
struct DependencyMutationArgs {
    task_id: String,
    #[arg(long)]
    depends_on: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ParentArgs {
    #[command(subcommand)]
    command: ParentCommand,
}

#[derive(Subcommand)]
enum ParentCommand {
    #[command(about = "Set or replace a task's parent")]
    Set(ParentSetArgs),
    #[command(about = "Remove a task's parent")]
    Remove(ParentRemoveArgs),
}

#[derive(Args)]
struct ParentSetArgs {
    task_id: String,
    #[arg(long)]
    parent: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ParentRemoveArgs {
    task_id: String,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskHierarchyArgs {
    task_id: String,
    #[arg(long)]
    recursive: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskMapArgs {
    task_id: String,
    #[arg(
        long,
        default_value = "all",
        value_parser = ["upstream", "downstream", "parent", "child", "all"]
    )]
    direction: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskUpdateArgs {
    id: String,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    goal: Option<String>,
    #[arg(long)]
    acceptance_criteria: Option<String>,
    #[arg(long = "type")]
    task_type: Option<String>,
    #[arg(long = "status")]
    status_code: Option<String>,
    #[arg(long)]
    priority: Option<i64>,
    #[arg(long)]
    estimate: Option<f64>,
    #[arg(long)]
    clear_estimate: bool,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long)]
    clear_tags: bool,
    #[arg(long = "url")]
    urls: Vec<String>,
    #[arg(long)]
    clear_urls: bool,
    #[arg(long = "code-ref")]
    code_references: Vec<String>,
    #[arg(long)]
    clear_code_refs: bool,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskViewArgs {
    id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskListArgs {
    #[arg(long)]
    archived: bool,
    #[arg(long)]
    all: bool,
    #[arg(long = "status")]
    statuses: Vec<String>,
    #[arg(long = "type")]
    task_types: Vec<String>,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskAvailableArgs {
    #[arg(long = "status")]
    statuses: Vec<String>,
    #[arg(long = "type")]
    task_types: Vec<String>,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TaskCreateArgs {
    #[arg(long)]
    title: String,
    #[arg(long = "type")]
    task_type: String,
    #[arg(long, default_value = "")]
    description: String,
    #[arg(long, default_value = "")]
    goal: String,
    #[arg(long, default_value = "")]
    acceptance_criteria: String,
    #[arg(long, default_value = "to_do")]
    status: String,
    #[arg(long, default_value_t = 50)]
    priority: i64,
    #[arg(long)]
    estimate: Option<f64>,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long = "url")]
    urls: Vec<String>,
    #[arg(long = "code-ref")]
    code_references: Vec<String>,
    #[arg(long)]
    agent: Option<uuid::Uuid>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Subcommand)]
enum AgentCommand {
    #[command(about = "Register a new repository-local agent identity")]
    Register(AgentRegisterArgs),
    #[command(about = "List registered agents and active claims")]
    List(AgentListArgs),
}

#[derive(Args)]
struct AgentRegisterArgs {
    base_name: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AgentListArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RepoArgs {
    #[command(subcommand)]
    command: RepoCommand,
}

#[derive(Subcommand)]
enum RepoCommand {
    #[command(about = "Validate repository config and database without changing them")]
    Status(RepoStatusArgs),
}

#[derive(Args)]
struct RepoStatusArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct InitArgs {
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long)]
    stealth: bool,
    #[arg(
        long,
        help = "Move an existing workspace to a unique backup before initializing"
    )]
    force: bool,
    #[arg(long, help = "Skip required confirmation")]
    yes: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct UninstallArgs {
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    ok: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

#[derive(Serialize)]
struct ApiError {
    code: String,
    message: String,
    details: serde_json::Value,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((error, json)) => {
            render_error(&error, json);
            ExitCode::from(exit_code(&error) as u8)
        }
    }
}

fn run() -> Result<(), (Error, bool)> {
    let cli = Cli::parse();
    let json = command_uses_json(&cli.command);
    let current = std::env::current_dir()
        .map_err(|error| (Error::phase("REPOSITORY_DISCOVERY_FAILED", error), json))?;
    match cli.command {
        Command::Init(args) => {
            let initial = initialize(
                &current,
                args.prefix.as_deref(),
                args.stealth,
                args.force,
                args.yes,
            );
            let result = match initial {
                Ok(result) => result,
                Err(error) if error.code() == "FORCE_CONFIRMATION_REQUIRED" && !args.json => {
                    let confirmed =
                        confirm("Force initialization moves the main worktree's existing .tbtm to a backup. Continue?")
                            .map_err(|e| (Error::phase("CONFIRMATION_FAILED", e), args.json))?;
                    if !confirmed {
                        render_success(&serde_json::json!({"cancelled": true}), args.json);
                        return Ok(());
                    }
                    initialize(
                        &current,
                        args.prefix.as_deref(),
                        args.stealth,
                        args.force,
                        true,
                    )
                    .map_err(|error| (error, args.json))?
                }
                Err(error) => return Err((error, args.json)),
            };
            render_success(&result, args.json);
        }
        Command::Uninstall(args) => {
            if !args.dry_run && !args.yes && (args.json || !io::stdin().is_terminal()) {
                return Err((
                    Error::phase(
                        "CONFIRMATION_REQUIRED",
                        io::Error::other("pass --yes for non-interactive uninstall"),
                    ),
                    args.json,
                ));
            }
            if !args.dry_run
                && !args.yes
                && !confirm("Uninstall removes TBTM files from the main worktree. Continue?")
                    .map_err(|e| (Error::phase("CONFIRMATION_FAILED", e), args.json))?
            {
                render_success(&serde_json::json!({"cancelled": true}), args.json);
                return Ok(());
            }
            let result = uninstall(&current, args.dry_run).map_err(|error| (error, args.json))?;
            let permission_failure = result
                .failed
                .iter()
                .any(|failure| failure.contains("Permission denied"));
            render_success(&result, args.json);
            if !result.failed.is_empty() {
                return Err((
                    Error::phase(
                        if permission_failure {
                            "UNINSTALL_PERMISSION_FAILED"
                        } else {
                            "UNINSTALL_FAILED"
                        },
                        io::Error::other("partial cleanup failure"),
                    ),
                    args.json,
                ));
            }
        }
        Command::Repo(args) => match args.command {
            RepoCommand::Status(args) => {
                let result =
                    inspect_repository_health(&current).map_err(|error| (error, args.json))?;
                render_repository_health(&result, args.json);
            }
        },
        Command::Agent(args) => match args.command {
            AgentCommand::Register(args) => {
                let result = register_agent(&current, &args.base_name)
                    .map_err(|error| (error, args.json))?;
                render_agent_registration(&result, args.json);
            }
            AgentCommand::List(args) => {
                let result = list_agents(&current).map_err(|error| (error, args.json))?;
                render_agent_list(&result, args.json);
            }
        },
        Command::Status(args) => match args.command {
            StatusCommand::List(args) => {
                let result = list_statuses(&current).map_err(|error| (error, args.json))?;
                render_status_list(&result, args.json);
            }
            StatusCommand::Create(args) => {
                let placement = placement(args.before.as_deref(), args.after.as_deref());
                let result = create_status(
                    &current,
                    CreateStatusInput {
                        code: &args.code,
                        name: &args.name,
                        placement,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_status_mutation("Created", &result, args.json);
            }
            StatusCommand::Rename(args) => {
                let result = rename_status(&current, &args.code, &args.name)
                    .map_err(|error| (error, args.json))?;
                render_status_mutation("Renamed", &result, args.json);
            }
            StatusCommand::Move(args) => {
                let placement = placement(args.before.as_deref(), args.after.as_deref())
                    .expect("clap requires a placement");
                let result = move_status(&current, &args.code, placement)
                    .map_err(|error| (error, args.json))?;
                render_status_move(&result, args.json);
            }
            StatusCommand::Delete(args) => {
                let result =
                    delete_status(&current, &args.code).map_err(|error| (error, args.json))?;
                render_status_delete(&result, args.json);
            }
            StatusCommand::SetCompleted(args) => {
                let preview = preview_set_completed(&current, &args.code, args.completed)
                    .map_err(|error| (error, args.json))?;
                let mut confirmed = args.yes;
                if preview.status.completed != args.completed
                    && !preview.impact.is_empty()
                    && !confirmed
                    && !args.json
                    && io::stdin().is_terminal()
                {
                    render_status_completion_impact(
                        &preview.status,
                        args.completed,
                        &preview.impact,
                        true,
                    );
                    confirmed = confirm("Change status completion semantics?")
                        .map_err(|error| (Error::phase("CONFIRMATION_FAILED", error), args.json))?;
                    if !confirmed {
                        println!("Status completion change cancelled.");
                        return Ok(());
                    }
                }
                let result = set_completed(
                    &current,
                    SetCompletedInput {
                        code: &args.code,
                        completed: args.completed,
                        confirmed,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_status_completion(&result, preview.status.completed, args.json);
            }
        },
        Command::Task(args) => match args.command {
            TaskCommand::Create(args) => {
                let task_type =
                    TaskType::parse(&args.task_type).map_err(|error| (error, args.json))?;
                let code_references = args
                    .code_references
                    .iter()
                    .map(|value| parse_code_reference(value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| (error, args.json))?;
                let result = create_task(
                    &current,
                    CreateTaskInput {
                        title: args.title,
                        task_type,
                        description: args.description,
                        goal: args.goal,
                        acceptance_criteria: args.acceptance_criteria,
                        status_code: args.status,
                        priority: args.priority,
                        estimate: args.estimate,
                        tags: args.tags,
                        external_urls: args.urls,
                        code_references,
                        agent_id: args.agent,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_created_task(&result, args.json);
            }
            TaskCommand::View(args) => {
                let result = view_task(&current, &args.id).map_err(|error| (error, args.json))?;
                render_task_detail(&result, args.json);
            }
            TaskCommand::List(args) => {
                if args.archived && args.all {
                    return Err((Error::ConflictingArguments, args.json));
                }
                let task_types = args
                    .task_types
                    .iter()
                    .map(|value| TaskType::parse(value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| (error, args.json))?;
                let result = list_tasks(
                    &current,
                    &ListTasksInput {
                        archive_scope: if args.archived {
                            ArchiveScope::Archived
                        } else if args.all {
                            ArchiveScope::All
                        } else {
                            ArchiveScope::Active
                        },
                        status_codes: args.statuses,
                        task_types,
                        tags: args.tags,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_task_list(&result, args.json);
            }
            TaskCommand::Available(args) => {
                let task_types = args
                    .task_types
                    .iter()
                    .map(|value| TaskType::parse(value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| (error, args.json))?;
                let result = available_tasks(
                    &current,
                    &AvailableTasksInput {
                        status_codes: args.statuses,
                        task_types,
                        tags: args.tags,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_available_tasks(&result, args.json);
            }
            TaskCommand::Blockers(args) => {
                let result = explain_task_blocking(&current, &args.task_id)
                    .map_err(|error| (error, args.json))?;
                render_task_blockers(&result, args.json);
            }
            TaskCommand::Update(args) => {
                if (args.estimate.is_some() && args.clear_estimate)
                    || (!args.tags.is_empty() && args.clear_tags)
                    || (!args.urls.is_empty() && args.clear_urls)
                    || (!args.code_references.is_empty() && args.clear_code_refs)
                {
                    return Err((Error::ConflictingArguments, args.json));
                }
                let references = args
                    .code_references
                    .iter()
                    .map(|value| parse_code_reference(value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| (error, args.json))?;
                let task_type = args
                    .task_type
                    .as_deref()
                    .map(TaskType::parse)
                    .transpose()
                    .map_err(|error| (error, args.json))?;
                let result = update_task(
                    &current,
                    &args.id,
                    UpdateTaskInput {
                        title: args.title,
                        description: args.description,
                        goal: args.goal,
                        acceptance_criteria: args.acceptance_criteria,
                        task_type,
                        status_code: args.status_code,
                        priority: args.priority,
                        estimate: patch(args.estimate, args.clear_estimate),
                        tags: collection_patch(args.tags, args.clear_tags),
                        external_urls: collection_patch(args.urls, args.clear_urls),
                        code_references: collection_patch(references, args.clear_code_refs),
                        agent_id: args.agent,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_task_detail(&result, args.json);
            }
            TaskCommand::Dependency(args) => match args.command {
                DependencyCommand::Add(args) => {
                    let result = add_dependency(
                        &current,
                        DependencyInput {
                            task_id: args.task_id,
                            depends_on: args.depends_on,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_dependency(&result, true, args.json);
                }
                DependencyCommand::Remove(args) => {
                    let result = remove_dependency(
                        &current,
                        DependencyInput {
                            task_id: args.task_id,
                            depends_on: args.depends_on,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_dependency(&result, false, args.json);
                }
            },
            TaskCommand::Parent(args) => match args.command {
                ParentCommand::Set(args) => {
                    let result = set_parent(
                        &current,
                        ParentSetInput {
                            task_id: args.task_id,
                            parent_id: args.parent,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_parent_mutation(&result, true, args.json);
                }
                ParentCommand::Remove(args) => {
                    let result = remove_parent(
                        &current,
                        ParentRemoveInput {
                            task_id: args.task_id,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_parent_mutation(&result, false, args.json);
                }
            },
            TaskCommand::Hierarchy(args) => {
                let result = task_hierarchy(&current, &args.task_id, args.recursive)
                    .map_err(|error| (error, args.json))?;
                render_hierarchy(&result, args.json);
            }
            TaskCommand::Map(args) => {
                let direction = match args.direction.as_str() {
                    "upstream" => RelationshipDirection::Upstream,
                    "downstream" => RelationshipDirection::Downstream,
                    "parent" => RelationshipDirection::Parent,
                    "child" => RelationshipDirection::Child,
                    "all" => RelationshipDirection::All,
                    _ => unreachable!("direction is validated by clap"),
                };
                let result = task_relationship_map(&current, &args.task_id, direction)
                    .map_err(|error| (error, args.json))?;
                render_relationship_map(&result, args.json);
            }
            TaskCommand::Claim(args) => {
                let result = claim_task(
                    &current,
                    ClaimTaskInput {
                        task_id: args.task_id,
                        agent_id: args.agent,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_claim(&result, args.json);
            }
            TaskCommand::ClaimNext(args) => {
                let task_types = args
                    .task_types
                    .iter()
                    .map(|value| TaskType::parse(value))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| (error, args.json))?;
                let result = claim_next_task(
                    &current,
                    ClaimNextTaskInput {
                        agent_id: args.agent,
                        status_codes: args.statuses,
                        task_types,
                        tags: args.tags,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_claim_next(result.as_ref(), args.json);
            }
            TaskCommand::Unclaim(args) => {
                if args.agent.is_some() == args.force || (args.yes && !args.force) {
                    return Err((Error::ConflictingArguments, args.json));
                }
                if args.force {
                    if !args.yes && (args.json || !io::stdin().is_terminal()) {
                        return Err((
                            Error::phase(
                                "CONFIRMATION_REQUIRED",
                                io::Error::other("pass --yes for non-interactive force unclaim"),
                            ),
                            args.json,
                        ));
                    }
                    let observed_claim = if args.yes {
                        None
                    } else {
                        let task = view_task(&current, &args.task_id)
                            .map_err(|error| (error, args.json))?;
                        let claim = task.claim.ok_or_else(|| {
                            (
                                Error::ClaimNotFound {
                                    task_id: args.task_id.clone(),
                                },
                                args.json,
                            )
                        })?;
                        let prompt = format!(
                            "Task: {}\nCurrent claim: {} ({}) at {}\nForce-unclaim this claim?",
                            args.task_id,
                            claim.agent.display_name,
                            claim.agent.id,
                            claim.claimed_at
                        );
                        if !confirm(&prompt).map_err(|error| {
                            (Error::phase("CONFIRMATION_FAILED", error), args.json)
                        })? {
                            println!("Unclaim cancelled.");
                            return Ok(());
                        }
                        Some(ObservedClaim {
                            agent_id: claim.agent.id,
                            claimed_at: claim.claimed_at,
                        })
                    };
                    let result = force_unclaim_task(
                        &current,
                        ForceUnclaimTaskInput {
                            task_id: args.task_id,
                            observed_claim,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_force_unclaim(&result, args.json);
                } else {
                    let result = unclaim_task(
                        &current,
                        UnclaimTaskInput {
                            task_id: args.task_id,
                            agent_id: args.agent.expect("validated owner path"),
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_unclaim(&result, args.json);
                }
            }
            TaskCommand::Archive(args) => {
                if args.yes && !args.force {
                    return Err((Error::ConflictingArguments, args.json));
                }
                if args.reason.trim().is_empty() {
                    return Err((Error::InvalidArchiveReason, args.json));
                }
                if args.force && args.agent.is_some() {
                    return Err((Error::ArchivePermissionDenied, args.json));
                }
                if args.force && !args.yes && (args.json || !io::stdin().is_terminal()) {
                    return Err((
                        Error::phase(
                            "CONFIRMATION_REQUIRED",
                            io::Error::other("pass --yes for non-interactive force archive"),
                        ),
                        args.json,
                    ));
                }
                let observed_task = if args.force {
                    Some(view_task(&current, &args.id).map_err(|error| (error, args.json))?)
                } else {
                    None
                };
                let observed_claim = observed_task.as_ref().and_then(|task| {
                    task.claim.as_ref().map(|claim| ObservedClaim {
                        agent_id: claim.agent.id.clone(),
                        claimed_at: claim.claimed_at.clone(),
                    })
                });
                if args.force
                    && !args.yes
                    && let Some(claim) = observed_task.and_then(|task| task.claim)
                {
                    let prompt = format!(
                        "Force archive task {} claimed by {} ({}) at {}?\nReason: {}",
                        args.id,
                        claim.agent.display_name,
                        claim.agent.id,
                        claim.claimed_at,
                        args.reason
                    );
                    if !confirm(&prompt)
                        .map_err(|error| (Error::phase("CONFIRMATION_FAILED", error), args.json))?
                    {
                        println!("Archive cancelled.");
                        return Ok(());
                    }
                }
                let result = archive_task(
                    &current,
                    ArchiveTaskInput {
                        task_id: args.id,
                        reason: args.reason,
                        agent_id: args.agent,
                        force: args.force,
                        observed_claim,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_task_detail(&result, args.json);
            }
            TaskCommand::Unarchive(args) => {
                let preview =
                    preview_unarchive(&current, &args.id).map_err(|error| (error, args.json))?;
                if !preview.task.archived {
                    render_unarchive(&preview, args.json);
                    return Ok(());
                }
                let mut confirmed = args.yes;
                if !preview.impact.is_empty()
                    && !confirmed
                    && !args.json
                    && io::stdin().is_terminal()
                {
                    render_unarchive_impact(&preview.impact, true);
                    confirmed = confirm(&format!("Unarchive task {}?", args.id))
                        .map_err(|error| (Error::phase("CONFIRMATION_FAILED", error), args.json))?;
                    if !confirmed {
                        println!("Unarchive cancelled.");
                        return Ok(());
                    }
                }
                let result = unarchive_task(
                    &current,
                    UnarchiveTaskInput {
                        task_id: args.id,
                        confirmed,
                    },
                )
                .map_err(|error| (error, args.json))?;
                render_unarchive(&result, args.json);
            }
            TaskCommand::Comment(args) => match args.command {
                TaskCommentCommand::Add(args) => {
                    let result = add_comment(
                        &current,
                        AddCommentInput {
                            task_id: args.task_id,
                            content: args.content,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_comment(&result, args.json);
                }
                TaskCommentCommand::List(args) => {
                    let result = list_comments(&current, &args.task_id)
                        .map_err(|error| (error, args.json))?;
                    render_comment_list(&result, args.json);
                }
                TaskCommentCommand::Delete(args) => {
                    let result = delete_comment(
                        &current,
                        DeleteCommentInput {
                            task_id: args.task_id,
                            comment_id: args.comment_id,
                            agent_id: args.agent,
                        },
                    )
                    .map_err(|error| (error, args.json))?;
                    render_deleted_comment(&result, args.json);
                }
            },
        },
    }
    Ok(())
}

fn command_uses_json(command: &Command) -> bool {
    match command {
        Command::Init(args) => args.json,
        Command::Uninstall(args) => args.json,
        Command::Repo(RepoArgs {
            command: RepoCommand::Status(args),
        }) => args.json,
        Command::Agent(AgentArgs { command }) => match command {
            AgentCommand::Register(args) => args.json,
            AgentCommand::List(args) => args.json,
        },
        Command::Status(StatusArgs { command }) => match command {
            StatusCommand::List(args) => args.json,
            StatusCommand::Create(args) => args.json,
            StatusCommand::Rename(args) => args.json,
            StatusCommand::Move(args) => args.json,
            StatusCommand::Delete(args) => args.json,
            StatusCommand::SetCompleted(args) => args.json,
        },
        Command::Task(args) => match &args.command {
            TaskCommand::Create(args) => args.json,
            TaskCommand::View(args) => args.json,
            TaskCommand::List(args) => args.json,
            TaskCommand::Available(args) => args.json,
            TaskCommand::Blockers(args) => args.json,
            TaskCommand::Update(args) => args.json,
            TaskCommand::Dependency(args) => match &args.command {
                DependencyCommand::Add(args) | DependencyCommand::Remove(args) => args.json,
            },
            TaskCommand::Parent(args) => match &args.command {
                ParentCommand::Set(args) => args.json,
                ParentCommand::Remove(args) => args.json,
            },
            TaskCommand::Hierarchy(args) => args.json,
            TaskCommand::Map(args) => args.json,
            TaskCommand::Claim(args) => args.json,
            TaskCommand::ClaimNext(args) => args.json,
            TaskCommand::Unclaim(args) => args.json,
            TaskCommand::Archive(args) => args.json,
            TaskCommand::Unarchive(args) => args.json,
            TaskCommand::Comment(args) => match &args.command {
                TaskCommentCommand::Add(args) => args.json,
                TaskCommentCommand::List(args) => args.json,
                TaskCommentCommand::Delete(args) => args.json,
            },
        },
    }
}

fn placement<'a>(before: Option<&'a str>, after: Option<&'a str>) -> Option<Placement<'a>> {
    before
        .map(Placement::Before)
        .or_else(|| after.map(Placement::After))
}

fn render_status_line(status: &Status) {
    println!(
        "{} | {} | {} | {} | position {}",
        status.code,
        status.name,
        if status.completed {
            "completed"
        } else {
            "incomplete"
        },
        if status.is_default {
            "default"
        } else {
            "custom"
        },
        status.display_order
    );
}

fn render_status_list(statuses: &[Status], json: bool) {
    if json {
        render_success(&statuses, true);
    } else if statuses.is_empty() {
        println!("No statuses.");
    } else {
        for status in statuses {
            render_status_line(status);
        }
    }
}

fn render_status_mutation(operation: &str, status: &Status, json: bool) {
    if json {
        render_success(status, true);
    } else {
        println!("{operation} status:");
        render_status_line(status);
    }
}

fn render_status_move(result: &MoveStatusResult, json: bool) {
    if json {
        render_success(result, true);
    } else {
        println!("Moved status:");
        render_status_line(&result.status);
        println!("Board order:");
        for status in &result.statuses {
            render_status_line(status);
        }
    }
}

fn render_status_delete(status: &Status, json: bool) {
    if json {
        render_success(status, true);
    } else {
        println!("Deleted status: {} | {}", status.code, status.name);
    }
}

fn render_status_completion(result: &SetCompletedResult, before: bool, json: bool) {
    if json {
        render_success(result, true);
    } else {
        render_status_completion_impact(
            &result.status,
            result.status.completed,
            &result.impact,
            false,
        );
        println!(
            "Completion: {} -> {}",
            if before { "completed" } else { "incomplete" },
            if result.status.completed {
                "completed"
            } else {
                "incomplete"
            }
        );
    }
}

fn render_status_completion_impact(
    status: &Status,
    proposed: bool,
    impact: &StatusCompletionImpact,
    warning: bool,
) {
    println!("Status: {} ({})", status.name, status.code);
    if warning {
        println!(
            "Proposed completion: {} -> {}",
            if status.completed {
                "completed"
            } else {
                "incomplete"
            },
            if proposed { "completed" } else { "incomplete" }
        );
    }
    println!("Status tasks:");
    if impact.status_tasks.is_empty() {
        println!("  —");
    }
    for item in &impact.status_tasks {
        println!(
            "- {} {}; claim: {}; available: {} -> {}",
            item.task_id,
            item.title,
            display_claim(item.claim.as_ref()),
            item.available_before,
            item.available_after
        );
    }
    println!("Direct downstream tasks:");
    if impact.downstream_tasks.is_empty() {
        println!("  —");
    }
    for item in &impact.downstream_tasks {
        println!(
            "- {} {}; claim: {}; available: {} -> {}; unresolved upstreams: {} -> {}",
            item.task_id,
            item.title,
            display_claim(item.claim.as_ref()),
            item.available_before,
            item.available_after,
            display_values(&item.unresolved_upstream_task_ids_before),
            display_values(&item.unresolved_upstream_task_ids_after)
        );
    }
}

fn display_claim(claim: Option<&tbtm_core::task::TaskClaim>) -> String {
    claim.map_or_else(
        || "none".to_owned(),
        |claim| {
            format!(
                "{} ({}) since {}",
                claim.agent.display_name, claim.agent.id, claim.claimed_at
            )
        },
    )
}

fn render_comment(comment: &TaskComment, json: bool) {
    if json {
        render_success(comment, true);
        return;
    }
    println!("Comment ID: {}", comment.id);
    println!("Task ID: {}", comment.task_id);
    println!("Author: {}", comment_author(comment));
    println!("Created: {}", comment.created_at);
    println!("Content:\n{}", comment.content);
}

fn render_comment_list(comments: &[TaskComment], json: bool) {
    if json {
        render_success(&comments, true);
    } else if comments.is_empty() {
        println!("No comments found.");
    } else {
        for (index, comment) in comments.iter().enumerate() {
            if index > 0 {
                println!();
            }
            render_comment(comment, false);
        }
    }
}

fn render_deleted_comment(comment: &TaskComment, json: bool) {
    if json {
        render_success(comment, true);
    } else {
        println!(
            "Deleted comment {} from task {}.",
            comment.id, comment.task_id
        );
    }
}

fn comment_author(comment: &TaskComment) -> String {
    comment.author_display_name.as_ref().map_or_else(
        || "user".to_owned(),
        |display_name| format!("{display_name} ({})", comment.author),
    )
}

fn render_claim(result: &FullTask, json: bool) {
    if json {
        render_success(result, true);
    } else {
        let claim = result
            .claim
            .as_ref()
            .expect("successful claim is populated");
        println!("Claimed task: {}", result.id);
        println!("Agent: {}", claim.agent.display_name);
        println!("Claimed at: {}", claim.claimed_at);
    }
}

fn render_claim_next(result: Option<&FullTask>, json: bool) {
    match result {
        Some(task) => render_claim(task, json),
        None => render_empty_claim_next(json),
    }
}

fn render_empty_claim_next(json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string(&Envelope::<()> {
                ok: true,
                data: None,
                error: None,
            })
            .expect("serializable response")
        );
    } else {
        println!("No available task to claim.");
    }
}

fn render_unclaim(result: &UnclaimTaskResult, json: bool) {
    if json {
        render_success(&result.task, true);
    } else {
        println!("Unclaimed task: {}", result.task.id);
        println!("Agent: {}", result.released_agent_display_name);
    }
}

fn render_force_unclaim(result: &ForceUnclaimTaskResult, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    println!("Force-unclaimed task: {}", result.task.id);
    println!(
        "Released claim: {} ({}) at {}",
        result.released_claim.agent.display_name,
        result.released_claim.agent.id,
        result.released_claim.claimed_at
    );
    match result.availability.reason {
        None => println!("Available: yes"),
        Some(AvailabilityReason::Archived) => println!("Available: no (archived)"),
        Some(AvailabilityReason::Completed) => println!("Available: no (completed)"),
        Some(AvailabilityReason::DependenciesBlocked) => println!(
            "Available: no (dependencies blocked: {})",
            result
                .availability
                .unresolved_upstream_ids
                .as_ref()
                .expect("dependency IDs")
                .join(", ")
        ),
        Some(AvailabilityReason::Claimed) => unreachable!("claim was released"),
    }
}

fn render_unarchive(result: &UnarchiveTaskResult, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    println!("Unarchived task: {}", result.task.id);
    println!("Title: {}", result.task.title);
    println!(
        "Status: {} ({})",
        result.task.status.name, result.task.status.code
    );
    println!("Archived: {}", result.task.archived);
    println!(
        "Archive reason: {}",
        result.task.archive_reason.as_deref().unwrap_or("—")
    );
    render_unarchive_impact(&result.impact, false);
}

fn render_unarchive_impact(impact: &UnarchiveImpact, warning: bool) {
    if warning {
        println!("Unarchiving will restore a blocker for direct downstream tasks:");
    } else if impact.is_empty() {
        println!("Downstream impact: none");
        return;
    } else {
        println!("Downstream impact:");
    }
    for (heading, items) in [
        ("Claimed", &impact.claimed),
        ("Otherwise available", &impact.otherwise_available),
        (
            "Already blocked elsewhere",
            &impact.already_blocked_elsewhere,
        ),
    ] {
        println!("{heading}:");
        if items.is_empty() {
            println!("  —");
            continue;
        }
        for item in items {
            let claim = item.claim.as_ref().map_or(String::new(), |claim| {
                format!(
                    "; claimed by {} ({}) since {}",
                    claim.agent.display_name, claim.agent.id, claim.claimed_at
                )
            });
            let blockers = if item.other_unresolved_upstream_task_ids.is_empty() {
                "none".to_owned()
            } else {
                item.other_unresolved_upstream_task_ids.join(", ")
            };
            println!(
                "- {} {}{}; other blockers: {}",
                item.task_id, item.title, claim, blockers
            );
        }
    }
}

fn render_dependency(result: &DependencyResult, added: bool, json: bool) {
    if json {
        render_success(result, true);
    } else if added {
        println!(
            "Added dependency: {} depends on {}",
            result.task_id, result.depends_on
        );
    } else {
        println!(
            "Removed dependency: {} no longer depends on {}",
            result.task_id, result.depends_on
        );
    }
}

fn render_parent_mutation(result: &ParentMutationResult, set: bool, json: bool) {
    if json {
        render_success(result, true);
    } else if set {
        println!(
            "Set parent: {} -> {}",
            result.task_id,
            result.parent_id.as_deref().expect("set has parent")
        );
    } else {
        println!("Removed parent: {}", result.task_id);
    }
}

fn render_hierarchy(result: &HierarchyResult, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    println!("Task: {} {}", result.task.id, result.task.title);
    println!("Parent:");
    match &result.parent {
        Some(task) => println!("- {} {}", task.id, task.title),
        None => println!("  —"),
    }
    println!("Direct children:");
    if result.children.is_empty() {
        println!("  —");
    } else {
        for task in &result.children {
            println!("- {} {}", task.id, task.title);
        }
    }
    if let Some(descendants) = &result.descendants {
        println!("Descendants:");
        if descendants.is_empty() {
            println!("  —");
        } else {
            for item in descendants {
                println!(
                    "- depth {}: {} {}",
                    item.depth, item.task.id, item.task.title
                );
            }
        }
    }
}

fn render_relationship_map(result: &RelationshipMap, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    let nodes: BTreeMap<_, _> = result
        .nodes
        .iter()
        .map(|node| (node.task.id.as_str(), node))
        .collect();
    let target = nodes[result.root_task_id.as_str()];
    println!(
        "Relationship map for {} ({})",
        result.root_task_id,
        result.direction.as_str()
    );
    println!("Target: {}", relationship_node_text(target));
    let sections = [
        (RelationshipDirection::Parent, "Parent hierarchy"),
        (RelationshipDirection::Child, "Child hierarchy"),
        (RelationshipDirection::Upstream, "Upstream dependencies"),
        (RelationshipDirection::Downstream, "Downstream dependencies"),
    ];
    let mut rendered = false;
    for (direction, heading) in sections {
        if result.direction != RelationshipDirection::All && result.direction != direction {
            continue;
        }
        let section_edges: Vec<_> = result
            .edges
            .iter()
            .filter(|edge| edge.direction == direction)
            .collect();
        if section_edges.is_empty() {
            continue;
        }
        rendered = true;
        println!("\n{heading}:");
        render_relationship_section(result, direction, &section_edges, &nodes);
    }
    if !rendered {
        if result.direction == RelationshipDirection::All {
            println!("\nNo relationships found.");
        } else {
            println!("\nNo {} relationships found.", result.direction.as_str());
        }
    }
}

#[derive(Debug)]
struct MapOccurrence<'a> {
    task_id: &'a str,
    full: bool,
}

fn render_relationship_section<'a>(
    result: &'a RelationshipMap,
    direction: RelationshipDirection,
    edges: &[&'a RelationshipEdge],
    nodes: &BTreeMap<&'a str, &'a RelationshipNode>,
) {
    let depths: BTreeMap<_, _> = result
        .nodes
        .iter()
        .filter_map(|node| {
            node.reached_by
                .iter()
                .find(|item| item.direction == direction)
                .map(|item| (node.task.id.as_str(), item.depth))
        })
        .chain([(result.root_task_id.as_str(), 0)])
        .collect();
    let endpoints = |edge: &'a RelationshipEdge| match direction {
        RelationshipDirection::Parent => (edge.to_task_id.as_str(), edge.from_task_id.as_str()),
        RelationshipDirection::Child => (edge.from_task_id.as_str(), edge.to_task_id.as_str()),
        RelationshipDirection::Upstream => (edge.to_task_id.as_str(), edge.from_task_id.as_str()),
        RelationshipDirection::Downstream => (edge.from_task_id.as_str(), edge.to_task_id.as_str()),
        RelationshipDirection::All => unreachable!("sections have concrete directions"),
    };
    let mut chosen: BTreeMap<&str, &str> = BTreeMap::new();
    for edge in edges {
        let (predecessor, reached) = endpoints(edge);
        if depths
            .get(predecessor)
            .zip(depths.get(reached))
            .is_some_and(|(predecessor_depth, reached_depth)| {
                predecessor_depth.checked_add(1) == Some(*reached_depth)
            })
        {
            chosen
                .entry(reached)
                .and_modify(|current| *current = (*current).min(predecessor))
                .or_insert(predecessor);
        }
    }
    let chosen_edges: BTreeSet<_> = chosen
        .iter()
        .map(|(reached, predecessor)| (*predecessor, *reached))
        .collect();
    let mut occurrences: BTreeMap<&str, Vec<MapOccurrence<'_>>> = BTreeMap::new();
    for edge in edges {
        let (predecessor, reached) = endpoints(edge);
        occurrences
            .entry(predecessor)
            .or_default()
            .push(MapOccurrence {
                task_id: reached,
                full: chosen_edges.contains(&(predecessor, reached)),
            });
    }
    for items in occurrences.values_mut() {
        items.sort_by(|left, right| {
            left.task_id
                .cmp(right.task_id)
                .then_with(|| right.full.cmp(&left.full))
        });
    }
    render_map_children(result.root_task_id.as_str(), "", &occurrences, nodes);
}

fn render_map_children(
    predecessor: &str,
    prefix: &str,
    occurrences: &BTreeMap<&str, Vec<MapOccurrence<'_>>>,
    nodes: &BTreeMap<&str, &RelationshipNode>,
) {
    let Some(items) = occurrences.get(predecessor) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let last = index + 1 == items.len();
        let connector = if last { "└─ " } else { "├─ " };
        if item.full {
            println!(
                "{prefix}{connector}{}",
                relationship_node_text(nodes[item.task_id])
            );
            let child_prefix = format!("{prefix}{}", if last { "   " } else { "│  " });
            render_map_children(item.task_id, &child_prefix, occurrences, nodes);
        } else {
            println!("{prefix}{connector}↩ {} (already shown)", item.task_id);
        }
    }
}

fn relationship_node_text(node: &RelationshipNode) -> String {
    let mut badges = String::new();
    if node.archived {
        badges.push_str(" [archived]");
    }
    if node.task.status.completed {
        badges.push_str(" [completed]");
    }
    if let Some(claim) = &node.claim {
        badges.push_str(&format!(
            " [claimed: {} ({})]",
            claim.agent.display_name, claim.agent.id
        ));
    }
    if node.blocked {
        badges.push_str(" [blocked]");
    }
    if node.ready {
        badges.push_str(" [ready]");
    }
    format!(
        "{} [{}, {}]{} {}",
        node.task.id,
        node.task.task_type.as_str(),
        node.task.status.code,
        badges,
        node.task.title
    )
}

fn patch<T>(value: Option<T>, clear: bool) -> PatchValue<T> {
    if clear {
        PatchValue::Clear
    } else {
        value.map_or(PatchValue::Omitted, PatchValue::Set)
    }
}

fn collection_patch<T>(values: Vec<T>, clear: bool) -> PatchValue<Vec<T>> {
    if clear {
        PatchValue::Clear
    } else if values.is_empty() {
        PatchValue::Omitted
    } else {
        PatchValue::Set(values)
    }
}

fn render_task_detail(result: &FullTask, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    println!("Task ID: {}", result.id);
    println!("Title: {}", result.title);
    println!("Type: {}", result.task_type.as_str());
    println!("Status: {} ({})", result.status.name, result.status.code);
    println!("Priority: {}", result.priority);
    println!(
        "Estimate: {}",
        result
            .estimate
            .map_or_else(|| "—".to_owned(), |value| value.to_string())
    );
    println!("Archived: {}", result.archived);
    println!(
        "Archive reason: {}",
        result.archive_reason.as_deref().unwrap_or("—")
    );
    println!("Description:\n{}", result.description);
    println!("Goal:\n{}", result.goal);
    println!("Acceptance criteria:\n{}", result.acceptance_criteria);
    println!("Tags: {}", display_values(&result.tags));
    println!("External URLs: {}", display_values(&result.external_urls));
    println!("Code references:");
    if result.code_references.is_empty() {
        println!("  —");
    } else {
        for reference in &result.code_references {
            let lines = match (reference.start_line, reference.end_line) {
                (Some(start), Some(end)) => format!(":{start}-{end}"),
                (Some(start), None) => format!(":{start}"),
                _ => String::new(),
            };
            let description = reference
                .description
                .as_ref()
                .map_or(String::new(), |value| format!(" — {value}"));
            println!("  {}{}{}", reference.path, lines, description);
        }
    }
    println!("Hierarchy:");
    println!(
        "  Parent: {}",
        result
            .hierarchy
            .parent
            .as_ref()
            .map_or("—", |task| task.id.as_str())
    );
    println!(
        "  Children: {}",
        if result.hierarchy.children.is_empty() {
            "—".to_owned()
        } else {
            result
                .hierarchy
                .children
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    println!("Dependencies: upstream —; downstream —");
    if let Some(claim) = &result.claim {
        println!(
            "Claim: {} ({}) at {}",
            claim.agent.display_name, claim.agent.id, claim.claimed_at
        );
    } else {
        println!("Claim: —");
    }
    println!("Created: {} by {}", result.created_at, result.created_by);
    println!("Updated: {} by {}", result.updated_at, result.updated_by);
}

fn display_values(values: &[String]) -> String {
    if values.is_empty() {
        "—".to_owned()
    } else {
        values.join(", ")
    }
}

fn render_task_list(result: &[TaskListItem], json: bool) {
    if json {
        render_success(&result, true);
        return;
    }
    if result.is_empty() {
        println!("No tasks found.");
        return;
    }
    println!(
        "{:<28} {:<12} {:<14} {:>8}  TITLE",
        "ID", "TYPE", "STATUS", "PRIORITY"
    );
    for task in result {
        println!(
            "{:<28} {:<12} {:<14} {:>8}  {}",
            task.id,
            task.task_type.as_str(),
            task.status.code,
            task.priority,
            task.title
        );
    }
}

fn render_available_tasks(result: &[TaskListItem], json: bool) {
    if result.is_empty() && !json {
        println!("No available tasks found.");
    } else {
        render_task_list(result, json);
    }
}

fn render_task_blockers(result: &TaskBlockingExplanation, json: bool) {
    if json {
        render_success(result, true);
        return;
    }
    if result.available {
        println!("{} is available.", result.task_id);
        return;
    }
    println!("{} is unavailable.", result.task_id);
    println!(
        "Reasons: {}",
        result
            .reasons
            .iter()
            .map(|reason| reason.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(claim) = &result.claim {
        println!(
            "Claim: {} ({}) since {}",
            claim.agent.display_name, claim.agent.id, claim.claimed_at
        );
    }
    for (heading, dependencies) in [
        (
            "Direct unresolved dependencies:",
            &result.unresolved_dependencies.direct,
        ),
        (
            "Recursive unresolved dependencies:",
            &result.unresolved_dependencies.recursive,
        ),
    ] {
        if dependencies.is_empty() {
            continue;
        }
        println!("{heading}");
        for dependency in dependencies {
            let blocked_by = if dependency.blocked_by_task_ids.is_empty() {
                "none".to_owned()
            } else {
                dependency.blocked_by_task_ids.join(", ")
            };
            println!(
                "- {} [{}, {}] {}; blocked by: {}",
                dependency.id,
                dependency.task_type.as_str(),
                dependency.status.code,
                dependency.title,
                blocked_by
            );
        }
    }
}

fn render_created_task(result: &CreatedTask, json: bool) {
    if json {
        render_success(result, true);
    } else {
        println!("Task ID: {}", result.id);
        println!("Title: {}", result.title);
        println!("Type: {}", result.task_type.as_str());
        println!("Status: {}", result.status.name);
        println!("Priority: {}", result.priority);
        println!("Actor: {}", result.created_by);
    }
}

fn confirm(prompt: &str) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn render_success<T: Serialize>(data: &T, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string(&Envelope {
                ok: true,
                data: Some(data),
                error: None
            })
            .expect("serializable response")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(data).expect("serializable response")
        );
    }
}

fn render_repository_health(result: &RepositoryHealth, json: bool) {
    if json {
        render_success(result, true);
    } else {
        println!("Repository root: {}", result.repository_root.display());
        println!("Worktree root: {}", result.worktree_root.display());
        println!("Config: {}", result.config_path.display());
        println!("Database: {}", result.database_path.display());
        println!("Repository ID: {}", result.repository_id);
        println!("Prefix: {}", result.prefix);
        println!("Schema version: {}", result.schema_version);
        println!("Health: {}", result.health);
    }
}

fn render_agent_registration(result: &AgentRegistration, json: bool) {
    if json {
        render_success(result, true);
    } else {
        println!("Agent ID: {}", result.id);
        println!("Base name: {}", result.base_name);
        println!("Display name: {}", result.display_name);
        println!("Created at: {}", result.created_at);
    }
}

fn render_agent_list(result: &AgentList, json: bool) {
    if json {
        render_success(result, true);
    } else if result.agents.is_empty() {
        println!("No registered agents.");
    } else {
        for agent in &result.agents {
            println!("{} ({})", agent.display_name, agent.id);
            println!("  Base name: {}", agent.base_name);
            println!("  Created at: {}", agent.created_at);
            if agent.claims.is_empty() {
                println!("  Claims: none");
            } else {
                println!("  Claims:");
                for claim in &agent.claims {
                    println!(
                        "    {} | {} | {} | {}",
                        claim.task_id, claim.title, claim.status, claim.claimed_at
                    );
                }
            }
        }
    }
}

fn render_error(error: &Error, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string(&Envelope::<serde_json::Value> {
                ok: false,
                data: None,
                error: Some(ApiError {
                    code: error.code().to_owned(),
                    message: error.to_string(),
                    details: error.details()
                })
            })
            .expect("serializable response")
        );
    } else {
        eprintln!("{}: {error}", error.code());
        if let Error::ConfirmationRequired { impact } = error {
            eprintln!("Downstream impact:");
            for (heading, items) in [
                ("Claimed", &impact.claimed),
                ("Otherwise available", &impact.otherwise_available),
                (
                    "Already blocked elsewhere",
                    &impact.already_blocked_elsewhere,
                ),
            ] {
                eprintln!("{heading}:");
                for item in items {
                    let blockers = if item.other_unresolved_upstream_task_ids.is_empty() {
                        "none".to_owned()
                    } else {
                        item.other_unresolved_upstream_task_ids.join(", ")
                    };
                    eprintln!(
                        "- {} {}; other blockers: {}",
                        item.task_id, item.title, blockers
                    );
                }
            }
        }
        if let Error::StatusCompletionConfirmationRequired { impact } = error {
            eprintln!("Status tasks:");
            for item in &impact.status_tasks {
                eprintln!(
                    "- {} {}; available: {} -> {}",
                    item.task_id, item.title, item.available_before, item.available_after
                );
            }
            eprintln!("Direct downstream tasks:");
            for item in &impact.downstream_tasks {
                eprintln!(
                    "- {} {}; available: {} -> {}; unresolved upstreams: {} -> {}",
                    item.task_id,
                    item.title,
                    item.available_before,
                    item.available_after,
                    display_values(&item.unresolved_upstream_task_ids_before),
                    display_values(&item.unresolved_upstream_task_ids_after)
                );
            }
        }
        if let Some(suggestion) = error.suggestion() {
            eprintln!("Next step: {suggestion}");
        }
    }
}
