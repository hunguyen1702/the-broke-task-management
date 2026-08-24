use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::{
    io::{self, IsTerminal, Write},
    process::ExitCode,
};
use tbtm_core::{
    AgentRegistration, Error, RepositoryHealth, exit_code, initialize, inspect_repository_health,
    register_agent, uninstall,
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
