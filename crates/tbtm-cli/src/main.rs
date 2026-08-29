use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::{
    io::{self, IsTerminal, Write},
    process::ExitCode,
};
use tbtm_core::{
    AgentRegistration, Error, RepositoryHealth, exit_code, initialize, inspect_repository_health,
    register_agent,
    task::{
        ArchiveScope, CreateTaskInput, CreatedTask, DependencyInput, DependencyResult, FullTask,
        ListTasksInput, PatchValue, TaskListItem, TaskType, UpdateTaskInput, add_dependency,
        create_task, list_tasks, parse_code_reference, remove_dependency, update_task, view_task,
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
    #[command(about = "Update a task")]
    Update(TaskUpdateArgs),
    #[command(about = "Manage task dependencies")]
    Dependency(DependencyArgs),
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
struct TaskUpdateArgs {
    id: String,
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
}

#[derive(Args)]
struct AgentRegisterArgs {
    base_name: String,
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
                let result = update_task(
                    &current,
                    &args.id,
                    UpdateTaskInput {
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
        Command::Agent(AgentArgs {
            command: AgentCommand::Register(args),
        }) => args.json,
        Command::Task(args) => match &args.command {
            TaskCommand::Create(args) => args.json,
            TaskCommand::View(args) => args.json,
            TaskCommand::List(args) => args.json,
            TaskCommand::Update(args) => args.json,
            TaskCommand::Dependency(args) => match &args.command {
                DependencyCommand::Add(args) | DependencyCommand::Remove(args) => args.json,
            },
        },
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
    println!("Hierarchy: parent —; children —");
    println!("Dependencies: upstream —; downstream —");
    println!("Claim: —");
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
        if let Some(suggestion) = error.suggestion() {
            eprintln!("Next step: {suggestion}");
        }
    }
}
