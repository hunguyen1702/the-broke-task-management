use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::{
    io::{self, IsTerminal, Write},
    process::ExitCode,
};
use tbtm_core::{Error, exit_code, initialize, uninstall};

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
        long_about = "Create repository-local .tbtm/config.json and .tbtm/tbtm.db. If initialization partially fails, inspect reported artifacts and run `tbtm uninstall` before retrying."
    )]
    Init(InitArgs),
    #[command(
        about = "Remove repository-local TBTM artifacts",
        long_about = "Remove .tbtm, direct-root TBTM backups and staging directories, plus exact /.tbtm/ rules. This is destructive; use --dry-run to inspect targets first."
    )]
    Uninstall(UninstallArgs),
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
    let current = std::env::current_dir()
        .map_err(|error| (Error::phase("REPOSITORY_DISCOVERY_FAILED", error), false))?;
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
                        confirm("Force initialization moves existing .tbtm to a backup. Continue?")
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
                && !confirm("Uninstall removes TBTM files from this repository. Continue?")
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
    }
    Ok(())
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
                    details: serde_json::json!({})
                })
            })
            .expect("serializable response")
        );
    } else {
        eprintln!("{}: {error}", error.code());
    }
}
