use deunicode::deunicode;
use rusqlite::{Connection, OpenFlags, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::OpenOptions,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use unicode_normalization::UnicodeNormalization;
use unicode_script::{Script, UnicodeScript};
use uuid::Uuid;

pub mod status;
pub mod task;

const CONFIG_FILE: &str = "config.json";
const DATABASE_FILE: &str = "tbtm.db";
const STEALTH_RULE: &str = "/.tbtm/";
const LATEST_MIGRATION: i64 = 3;
const IDENTITY_RETRY_LIMIT: usize = 8;

#[derive(Debug, Error)]
pub enum Error {
    #[error("INVALID_PREFIX: prefix must normalize to 1–48 ASCII characters")]
    InvalidPrefix,
    #[error("agent name must normalize to 1–48 ASCII characters")]
    InvalidAgentName { normalized: String },
    #[error("task title must not be empty")]
    InvalidTaskTitle,
    #[error("invalid task type: {value}")]
    InvalidTaskType { value: String },
    #[error("priority must be between 0 and 1000000")]
    InvalidPriority,
    #[error("estimate must be a finite non-negative number of hours")]
    InvalidEstimate,
    #[error("invalid external URL: {value}")]
    InvalidExternalUrl { value: String },
    #[error("invalid code reference: {value}")]
    InvalidCodeReference { value: String },
    #[error("duplicate or empty task context: {value}")]
    DuplicateTaskContext { value: String },
    #[error("agent not found: {id}")]
    AgentNotFound { id: Uuid },
    #[error("status not found: {code}")]
    StatusNotFound { code: String },
    #[error("task not found: {id}")]
    TaskNotFound { id: String },
    #[error("task is archived: {id}")]
    TaskArchived { id: String },
    #[error("at least one update field must be supplied")]
    NoUpdateFields,
    #[error("--archived and --all cannot be used together")]
    ConflictingArguments,
    #[error("ALREADY_INITIALIZED: valid TBTM workspace already exists")]
    AlreadyInitialized,
    #[error("INVALID_INITIALIZATION: existing .tbtm is missing, corrupt, or mismatched")]
    InvalidInitialization,
    #[error("GITIGNORE_UPDATE_FAILED: {0}")]
    GitignoreUpdate(#[source] io::Error),
    #[error("repository is not initialized at {path}; run `tbtm init`")]
    RepositoryNotInitialized { path: PathBuf },
    #[error("invalid repository configuration ({check}) at {path}: {message}")]
    InvalidConfiguration {
        check: &'static str,
        path: PathBuf,
        message: String,
        mismatched_fields: Vec<String>,
    },
    #[error("database unavailable ({check}) at {path}: {message}")]
    DatabaseUnavailable {
        check: &'static str,
        path: PathBuf,
        message: String,
    },
    #[error("permission denied during {check} at {path}: {message}")]
    PermissionDenied {
        check: &'static str,
        path: PathBuf,
        message: String,
    },
    #[error("repository unavailable during {phase}: {message}")]
    RepositoryUnavailable {
        phase: &'static str,
        message: String,
    },
    #[error("{phase}: {source}")]
    Phase {
        phase: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    pub fn phase(
        phase: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Phase {
            phase,
            source: Box::new(source),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidPrefix => "INVALID_PREFIX",
            Self::InvalidAgentName { .. } => "INVALID_AGENT_NAME",
            Self::InvalidTaskTitle => "INVALID_TASK_TITLE",
            Self::InvalidTaskType { .. } => "INVALID_TASK_TYPE",
            Self::InvalidPriority => "INVALID_PRIORITY",
            Self::InvalidEstimate => "INVALID_ESTIMATE",
            Self::InvalidExternalUrl { .. } => "INVALID_EXTERNAL_URL",
            Self::InvalidCodeReference { .. } => "INVALID_CODE_REFERENCE",
            Self::DuplicateTaskContext { .. } => "DUPLICATE_TASK_CONTEXT",
            Self::AgentNotFound { .. } => "AGENT_NOT_FOUND",
            Self::StatusNotFound { .. } => "STATUS_NOT_FOUND",
            Self::TaskNotFound { .. } => "TASK_NOT_FOUND",
            Self::TaskArchived { .. } => "TASK_ARCHIVED",
            Self::NoUpdateFields => "NO_UPDATE_FIELDS",
            Self::ConflictingArguments => "CONFLICTING_ARGUMENTS",
            Self::AlreadyInitialized => "ALREADY_INITIALIZED",
            Self::InvalidInitialization => "INVALID_INITIALIZATION",
            Self::GitignoreUpdate(_) => "GITIGNORE_UPDATE_FAILED",
            Self::RepositoryNotInitialized { .. } => "REPOSITORY_NOT_INITIALIZED",
            Self::InvalidConfiguration { .. } => "INVALID_CONFIGURATION",
            Self::DatabaseUnavailable { .. } => "DATABASE_UNAVAILABLE",
            Self::PermissionDenied { .. } => "PERMISSION_DENIED",
            Self::RepositoryUnavailable { .. } => "REPOSITORY_UNAVAILABLE",
            Self::Phase { phase, .. } => phase,
        }
    }

    pub fn details(&self) -> serde_json::Value {
        match self {
            Self::InvalidAgentName { normalized } => serde_json::json!({
                "normalized": normalized,
                "minimumLength": 1,
                "maximumLength": 48
            }),
            Self::RepositoryNotInitialized { path } => serde_json::json!({
                "check": "workspace",
                "path": path,
                "suggestion": "Run `tbtm init` from the repository root."
            }),
            Self::InvalidConfiguration {
                check,
                path,
                mismatched_fields,
                ..
            } => serde_json::json!({
                "check": check,
                "path": path,
                "mismatchedFields": mismatched_fields,
                "suggestion": "Restore a matching config and database from backup, or reinitialize the repository."
            }),
            Self::DatabaseUnavailable { check, path, .. } => serde_json::json!({
                "check": check,
                "path": path,
                "suggestion": "Restore a valid database from backup or reinitialize the repository."
            }),
            Self::PermissionDenied { check, path, .. } => serde_json::json!({
                "check": check,
                "path": path,
                "suggestion": "Grant access to the path and retry."
            }),
            Self::RepositoryUnavailable { phase, .. } => serde_json::json!({
                "phase": phase,
                "suggestion": "Repair the Git worktree metadata or run the command from a usable non-bare worktree."
            }),
            _ => serde_json::json!({}),
        }
    }

    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::RepositoryNotInitialized { .. } => {
                Some("Run `tbtm init` from the repository root.")
            }
            Self::InvalidConfiguration { .. } => Some(
                "Restore a matching config and database from backup, or reinitialize the repository.",
            ),
            Self::DatabaseUnavailable { .. } => {
                Some("Restore a valid database from backup or reinitialize the repository.")
            }
            Self::PermissionDenied { .. } => Some("Grant access to the path and retry."),
            Self::RepositoryUnavailable { .. } => Some(
                "Repair the Git worktree metadata or run the command from a usable non-bare worktree.",
            ),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRoot(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeRoot(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryDiscovery {
    pub repository_root: RepositoryRoot,
    pub worktree_root: WorktreeRoot,
    pub common_git_dir: Option<PathBuf>,
}

impl RepositoryRoot {
    pub fn discover(current: &Path) -> Result<RepositoryDiscovery, Error> {
        let canonical = current.canonicalize().map_err(|error| {
            if error.kind() == io::ErrorKind::PermissionDenied {
                Error::PermissionDenied {
                    check: "repository discovery",
                    path: current.to_path_buf(),
                    message: error.to_string(),
                }
            } else {
                Error::phase("REPOSITORY_DISCOVERY_FAILED", error)
            }
        })?;
        check_git_marker_access(&canonical).map_err(|error| {
            if error.kind() == io::ErrorKind::PermissionDenied {
                Error::PermissionDenied {
                    check: "Git repository discovery",
                    path: canonical.clone(),
                    message: error.to_string(),
                }
            } else {
                Error::phase("REPOSITORY_DISCOVERY_FAILED", error)
            }
        })?;
        let repository = match git2::Repository::discover(&canonical) {
            Ok(repository) => repository,
            Err(error) if error.code() == git2::ErrorCode::NotFound => {
                return Ok(RepositoryDiscovery {
                    repository_root: Self(canonical.clone()),
                    worktree_root: WorktreeRoot(canonical),
                    common_git_dir: None,
                });
            }
            Err(error) => return Err(repository_unavailable("Git repository discovery", error)),
        };
        let worktree = repository
            .workdir()
            .ok_or_else(|| Error::RepositoryUnavailable {
                phase: "current worktree resolution",
                message: "bare Git repositories are not supported".to_owned(),
            })?;
        let worktree_root = canonicalize_git_path("current worktree canonicalization", worktree)?;
        let common_git_dir = canonicalize_git_path(
            "common Git directory canonicalization",
            repository.commondir(),
        )?;
        let main_repository = git2::Repository::open(&common_git_dir)
            .map_err(|error| repository_unavailable("main worktree resolution", error))?;
        let main_worktree =
            main_repository
                .workdir()
                .ok_or_else(|| Error::RepositoryUnavailable {
                    phase: "main worktree resolution",
                    message: "Git common metadata does not identify a usable main worktree"
                        .to_owned(),
                })?;
        let repository_root =
            canonicalize_git_path("main worktree canonicalization", main_worktree)?;
        let validated_common = canonicalize_git_path(
            "common Git metadata validation",
            main_repository.commondir(),
        )?;
        if validated_common != common_git_dir {
            return Err(Error::RepositoryUnavailable {
                phase: "common Git metadata validation",
                message: "current and main worktrees do not share one common Git directory"
                    .to_owned(),
            });
        }
        Ok(RepositoryDiscovery {
            repository_root: Self(repository_root),
            worktree_root: WorktreeRoot(worktree_root),
            common_git_dir: Some(common_git_dir),
        })
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
    fn workspace(&self) -> PathBuf {
        self.0.join(".tbtm")
    }
}

impl WorktreeRoot {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

fn canonicalize_git_path(phase: &'static str, path: &Path) -> Result<PathBuf, Error> {
    path.canonicalize().map_err(|error| {
        if error.kind() == io::ErrorKind::PermissionDenied {
            Error::PermissionDenied {
                check: phase,
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        } else {
            Error::RepositoryUnavailable {
                phase,
                message: error.to_string(),
            }
        }
    })
}

fn repository_unavailable(phase: &'static str, error: impl std::fmt::Display) -> Error {
    Error::RepositoryUnavailable {
        phase,
        message: error.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub repository_id: Uuid,
    pub prefix: String,
    pub database: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessIntent {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug)]
pub struct ResolvedRepository {
    pub repository_root: PathBuf,
    pub worktree_root: PathBuf,
    pub config_path: PathBuf,
    pub database_path: PathBuf,
    pub config: Config,
    connection: Connection,
}

impl ResolvedRepository {
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryHealth {
    pub repository_root: PathBuf,
    pub worktree_root: PathBuf,
    pub config_path: PathBuf,
    pub database_path: PathBuf,
    pub repository_id: Uuid,
    pub prefix: String,
    pub schema_version: u32,
    pub health: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitResult {
    pub repository_root: PathBuf,
    pub prefix: String,
    pub repository_id: Uuid,
    pub config_path: PathBuf,
    pub database_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub stealth_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallResult {
    pub dry_run: bool,
    pub planned: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub failed: Vec<String>,
    pub stealth_lines_planned: usize,
    pub stealth_lines_removed: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistration {
    pub id: Uuid,
    pub base_name: String,
    pub display_name: String,
    pub created_at: String,
}

pub fn resolve_repository(
    current: &Path,
    access: AccessIntent,
) -> Result<ResolvedRepository, Error> {
    let discovery = RepositoryRoot::discover(current)?;
    resolve_workspace(discovery.repository_root, discovery.worktree_root, access)
}

fn resolve_workspace(
    root: RepositoryRoot,
    worktree_root: WorktreeRoot,
    access: AccessIntent,
) -> Result<ResolvedRepository, Error> {
    let workspace = root.workspace();
    let workspace_metadata = match fs::symlink_metadata(&workspace) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Error::RepositoryNotInitialized { path: workspace });
        }
        Err(error) => return Err(path_access_error("workspace", &workspace, error)),
    };
    if workspace_metadata.file_type().is_symlink() || !workspace_metadata.is_dir() {
        return Err(invalid_configuration(
            "workspace",
            &workspace,
            ".tbtm must be a real directory owned by the repository",
        ));
    }

    let config_path = workspace.join(CONFIG_FILE);
    let config = load_config(&config_path)?;

    let database_path = workspace.join(DATABASE_FILE);
    let database_metadata = fs::symlink_metadata(&database_path)
        .map_err(|error| path_database_error("database access", &database_path, error))?;
    if database_metadata.file_type().is_symlink() || !database_metadata.is_file() {
        return Err(Error::DatabaseUnavailable {
            check: "database access",
            path: database_path,
            message: "database must be an existing regular file, not a symlink".to_owned(),
        });
    }

    let flags = match access {
        AccessIntent::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
        AccessIntent::ReadWrite => OpenFlags::SQLITE_OPEN_READ_WRITE,
    } | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_NOFOLLOW;
    let connection = Connection::open_with_flags(&database_path, flags)
        .map_err(|error| sqlite_open_error(&database_path, access, error))?;
    if access == AccessIntent::ReadWrite {
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| sqlite_access_error("foreign key setup", &database_path, error))?;
    }
    connection
        .query_row("PRAGMA schema_version", [], |row| row.get::<_, i64>(0))
        .map_err(|error| sqlite_access_error("database validation", &database_path, error))?;

    Ok(ResolvedRepository {
        repository_root: root.0,
        worktree_root: worktree_root.0,
        config_path,
        database_path,
        config,
        connection,
    })
}

fn load_config(path: &Path) -> Result<Config, Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| path_config_error("config access", path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(invalid_configuration(
            "config access",
            path,
            "config must be an existing regular file, not a symlink",
        ));
    }
    let bytes = fs::read(path).map_err(|error| path_config_error("config read", path, error))?;
    let config: Config = serde_json::from_slice(&bytes)
        .map_err(|error| invalid_configuration("config parse", path, error.to_string()))?;
    validate_config(&config, path)?;
    Ok(config)
}

pub fn inspect_repository_health(current: &Path) -> Result<RepositoryHealth, Error> {
    let resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let metadata: (String, String, String) = resolved
        .connection
        .query_row(
            "SELECT repository_id, prefix, created_at FROM repository_metadata WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| {
            sqlite_access_error("repository metadata", &resolved.database_path, error)
        })?;
    validate_migration_ledger(&resolved.connection, &resolved.database_path)?;

    let mut mismatched_fields = Vec::new();
    if metadata.0 != resolved.config.repository_id.to_string() {
        mismatched_fields.push("repositoryId".to_owned());
    }
    if metadata.1 != resolved.config.prefix {
        mismatched_fields.push("prefix".to_owned());
    }
    if metadata.2 != resolved.config.created_at {
        mismatched_fields.push("createdAt".to_owned());
    }
    if !mismatched_fields.is_empty() {
        return Err(Error::InvalidConfiguration {
            check: "config/database metadata agreement",
            path: resolved.database_path,
            message: format!("mismatched fields: {}", mismatched_fields.join(", ")),
            mismatched_fields,
        });
    }

    let mut statement = resolved
        .connection
        .prepare("PRAGMA quick_check")
        .map_err(|error| sqlite_access_error("quick_check", &resolved.database_path, error))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| sqlite_access_error("quick_check", &resolved.database_path, error))?;
    let results: Result<Vec<_>, _> = rows.collect();
    let results = results
        .map_err(|error| sqlite_access_error("quick_check", &resolved.database_path, error))?;
    drop(statement);
    if results.as_slice() != ["ok"] {
        return Err(Error::DatabaseUnavailable {
            check: "quick_check",
            path: resolved.database_path,
            message: results.join("; "),
        });
    }

    Ok(RepositoryHealth {
        repository_root: resolved.repository_root,
        worktree_root: resolved.worktree_root,
        config_path: resolved.config_path,
        database_path: resolved.database_path,
        repository_id: resolved.config.repository_id,
        prefix: resolved.config.prefix,
        schema_version: resolved.config.schema_version,
        health: "healthy",
    })
}

fn validate_config(config: &Config, path: &Path) -> Result<(), Error> {
    if config.schema_version != 1 {
        return Err(invalid_configuration(
            "schemaVersion",
            path,
            format!("unsupported schema version {}", config.schema_version),
        ));
    }
    if normalize_prefix(&config.prefix).ok().as_deref() != Some(config.prefix.as_str()) {
        return Err(invalid_configuration(
            "prefix",
            path,
            "prefix must already be normalized",
        ));
    }
    if config.database != DATABASE_FILE {
        return Err(invalid_configuration(
            "database",
            path,
            "schema version 1 requires the exact value `tbtm.db`",
        ));
    }
    let created_at = OffsetDateTime::parse(&config.created_at, &Rfc3339).map_err(|error| {
        invalid_configuration(
            "createdAt",
            path,
            format!("invalid RFC 3339 value: {error}"),
        )
    })?;
    if created_at.offset() != time::UtcOffset::UTC {
        return Err(invalid_configuration(
            "createdAt",
            path,
            "timestamp must use a UTC offset",
        ));
    }
    Ok(())
}

fn invalid_configuration(check: &'static str, path: &Path, message: impl Into<String>) -> Error {
    Error::InvalidConfiguration {
        check,
        path: path.to_path_buf(),
        message: message.into(),
        mismatched_fields: Vec::new(),
    }
}

fn path_access_error(check: &'static str, path: &Path, error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::PermissionDenied {
        Error::PermissionDenied {
            check,
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    } else {
        invalid_configuration(check, path, error.to_string())
    }
}

fn path_config_error(check: &'static str, path: &Path, error: io::Error) -> Error {
    path_access_error(check, path, error)
}

fn path_database_error(check: &'static str, path: &Path, error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::PermissionDenied {
        Error::PermissionDenied {
            check,
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    } else {
        Error::DatabaseUnavailable {
            check,
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    }
}

fn sqlite_access_error(check: &'static str, path: &Path, error: rusqlite::Error) -> Error {
    let permission_denied = matches!(
        error,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::PermissionDenied,
                ..
            },
            _
        )
    );
    if permission_denied {
        Error::PermissionDenied {
            check,
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    } else {
        Error::DatabaseUnavailable {
            check,
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    }
}

fn sqlite_open_error(path: &Path, access: AccessIntent, error: rusqlite::Error) -> Error {
    let cannot_open = matches!(
        error,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::CannotOpen,
                ..
            },
            _
        )
    );
    if cannot_open {
        let filesystem_result = match access {
            AccessIntent::ReadOnly => fs::File::open(path).map(|_| ()),
            AccessIntent::ReadWrite => OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map(|_| ()),
        };
        if let Err(filesystem_error) = filesystem_result
            && filesystem_error.kind() == io::ErrorKind::PermissionDenied
        {
            return Error::PermissionDenied {
                check: "database open",
                path: path.to_path_buf(),
                message: filesystem_error.to_string(),
            };
        }
    }
    sqlite_access_error("database open", path, error)
}

fn check_git_marker_access(current: &Path) -> io::Result<()> {
    for ancestor in current.ancestors() {
        let marker = ancestor.join(".git");
        let metadata = match fs::symlink_metadata(&marker) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if metadata.is_dir() {
            fs::read_dir(marker)?.next().transpose()?;
        } else if metadata.is_file() {
            fs::File::open(marker)?;
        }
        break;
    }
    Ok(())
}

pub fn normalize_prefix(input: &str) -> Result<String, Error> {
    normalize_name(input).ok_or(Error::InvalidPrefix)
}

fn normalize_name(input: &str) -> Option<String> {
    if input
        .chars()
        .any(|character| character.is_alphabetic() && character.script() != Script::Latin)
    {
        return None;
    }
    let ascii = deunicode(&input.nfc().collect::<String>()).to_ascii_lowercase();
    let mut normalized = String::new();
    let mut separator = false;
    for character in ascii.chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            normalized.push(character);
            separator = false;
        } else if !normalized.is_empty() {
            separator = true;
        }
        if separator && !normalized.ends_with('-') {
            normalized.push('-');
        }
    }
    let value = normalized.trim_matches('-').to_owned();
    if value.is_empty() || value.len() > 48 {
        return None;
    }
    Some(value)
}

pub fn normalize_agent_name(input: &str) -> Result<String, Error> {
    normalize_name(input).ok_or_else(|| Error::InvalidAgentName {
        normalized: normalize_without_validation(input),
    })
}

fn normalize_without_validation(input: &str) -> String {
    if input
        .chars()
        .any(|character| character.is_alphabetic() && character.script() != Script::Latin)
    {
        return String::new();
    }
    let ascii = deunicode(&input.nfc().collect::<String>()).to_ascii_lowercase();
    ascii
        .split(|character: char| !character.is_ascii_lowercase() && !character.is_ascii_digit())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn register_agent(current: &Path, input: &str) -> Result<AgentRegistration, Error> {
    let base_name = normalize_agent_name(input)?;
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    register_agent_with(&mut resolved, base_name, Uuid::new_v4)
}

fn register_agent_with(
    resolved: &mut ResolvedRepository,
    base_name: String,
    mut generate_id: impl FnMut() -> Uuid,
) -> Result<AgentRegistration, Error> {
    for _ in 0..IDENTITY_RETRY_LIMIT {
        let id = generate_id();
        let display_name = format!("{}-{}", base_name, &id.simple().to_string()[..8]);
        let created_at = utc_now();
        let transaction = resolved
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                sqlite_access_error("agent registration", &resolved.database_path, error)
            })?;
        match transaction.execute(
            "INSERT INTO agents (id, base_name, display_name, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), base_name, display_name, created_at],
        ) {
            Ok(1) => {
                transaction.commit().map_err(|error| {
                    sqlite_access_error("agent registration", &resolved.database_path, error)
                })?;
                return Ok(AgentRegistration {
                    id,
                    base_name,
                    display_name,
                    created_at,
                });
            }
            Err(error) if is_identity_collision(&error) => continue,
            Err(error) => {
                return Err(sqlite_access_error(
                    "agent registration",
                    &resolved.database_path,
                    error,
                ));
            }
            Ok(_) => unreachable!("one agent row is inserted"),
        }
    }
    Err(Error::phase(
        "AGENT_REGISTRATION_FAILED",
        io::Error::other("unique identity retry limit exhausted"),
    ))
}

fn is_identity_collision(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(failure, message)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation
                && message.as_deref().is_some_and(|message| {
                    message.contains("agents.id") || message.contains("agents.display_name")
                })
    )
}

fn validate_migration_ledger(connection: &Connection, path: &Path) -> Result<Vec<i64>, Error> {
    let mut statement = connection
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .map_err(|error| sqlite_access_error("schema metadata", path, error))?;
    let versions = statement
        .query_map([], |row| row.get::<_, i64>(0))
        .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
        .map_err(|error| sqlite_access_error("schema metadata", path, error))?;
    if versions.is_empty()
        || versions[0] != 1
        || versions.windows(2).any(|pair| pair[1] != pair[0] + 1)
        || versions
            .last()
            .is_some_and(|version| *version > LATEST_MIGRATION)
    {
        return Err(Error::DatabaseUnavailable {
            check: "schema metadata",
            path: path.to_path_buf(),
            message: "migration ledger is missing, non-sequential, or unsupported".to_owned(),
        });
    }
    Ok(versions)
}

fn apply_pending_migrations(resolved: &mut ResolvedRepository) -> Result<(), Error> {
    let transaction = resolved
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| {
            sqlite_access_error("database migration", &resolved.database_path, error)
        })?;
    let versions = validate_migration_ledger(&transaction, &resolved.database_path)?;
    let current = *versions.last().expect("validated non-empty ledger");
    for (version, sql) in [
        (2, include_str!("../migrations/0002_agent_registry.sql")),
        (3, include_str!("../migrations/0003_task_core.sql")),
    ] {
        if version > current {
            transaction
                .execute_batch(sql)
                .and_then(|_| {
                    transaction.execute(
                        "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                        params![version, utc_now()],
                    )
                })
                .map_err(|error| {
                    sqlite_access_error("database migration", &resolved.database_path, error)
                })?;
        }
    }
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("database migration", &resolved.database_path, error))
}

pub fn initialize(
    current: &Path,
    prefix_input: Option<&str>,
    stealth: bool,
    force: bool,
    force_confirmed: bool,
) -> Result<InitResult, Error> {
    let discovery = RepositoryRoot::discover(current)?;
    let root = discovery.repository_root;
    let default_prefix = root
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(Error::InvalidPrefix)?;
    let prefix = normalize_prefix(prefix_input.unwrap_or(default_prefix))?;
    let workspace = root.workspace();
    let mut backup_path = None;

    match fs::symlink_metadata(&workspace) {
        Ok(_) if !force => {
            return match valid_workspace(&workspace) {
                Ok(true) => Err(Error::AlreadyInitialized),
                Ok(false) | Err(_) => Err(Error::InvalidInitialization),
            };
        }
        Ok(_) if !force_confirmed => {
            return Err(Error::phase(
                "FORCE_CONFIRMATION_REQUIRED",
                io::Error::other("pass --yes to confirm force initialization"),
            ));
        }
        Ok(_) => {
            let backup = unique_backup_path(root.path());
            fs::rename(&workspace, &backup).map_err(|e| Error::phase("BACKUP_FAILED", e))?;
            backup_path = Some(backup);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::phase("WORKSPACE_INSPECTION_FAILED", error)),
    }

    fs::create_dir(&workspace).map_err(|e| Error::phase("WORKSPACE_CREATE_FAILED", e))?;
    let repository_id = Uuid::new_v4();
    let created_at = utc_now();
    let database_path = workspace.join(DATABASE_FILE);
    create_database(&database_path, repository_id, &prefix, &created_at)?;
    let config_path = workspace.join(CONFIG_FILE);
    let config = Config {
        schema_version: 1,
        repository_id,
        prefix: prefix.clone(),
        database: DATABASE_FILE.to_owned(),
        created_at,
    };
    write_config(&config_path, &config)?;
    let stealth_changed = if stealth {
        add_stealth_rule(root.path())?
    } else {
        false
    };
    Ok(InitResult {
        repository_root: root.0,
        prefix,
        repository_id,
        config_path,
        database_path,
        backup_path,
        stealth_changed,
    })
}

fn unique_backup_path(root: &Path) -> PathBuf {
    loop {
        let stamp = OffsetDateTime::now_utc()
            .format(&time::macros::format_description!(
                "[year][month][day]T[hour][minute][second]Z"
            ))
            .expect("valid timestamp format");
        let candidate = root.join(format!(
            ".tbtm.backup-{stamp}-{}",
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
}

fn utc_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC 3339 formatting succeeds")
}

fn create_database(
    path: &Path,
    repository_id: Uuid,
    prefix: &str,
    created_at: &str,
) -> Result<(), Error> {
    let mut connection =
        Connection::open(path).map_err(|e| Error::phase("DATABASE_CREATE_FAILED", e))?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE; PRAGMA synchronous = FULL;",
        )
        .map_err(|e| Error::phase("DATABASE_CREATE_FAILED", e))?;
    let transaction = connection
        .transaction()
        .map_err(|e| Error::phase("DATABASE_CREATE_FAILED", e))?;
    transaction
        .execute_batch(include_str!("../migrations/0001_repository_foundation.sql"))
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction
        .execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
            [created_at],
        )
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction
        .execute_batch(include_str!("../migrations/0002_agent_registry.sql"))
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction
        .execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (2, ?1)",
            [created_at],
        )
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction
        .execute_batch(include_str!("../migrations/0003_task_core.sql"))
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction
        .execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (3, ?1)",
            [created_at],
        )
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    transaction.execute("INSERT INTO repository_metadata (singleton, repository_id, prefix, created_at) VALUES (1, ?1, ?2, ?3)", params![repository_id.to_string(), prefix, created_at]).map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    for (code, name, completed, order) in [
        ("to_do", "Todo", false, 0),
        ("in_progress", "In progress", false, 1),
        ("done", "Done", true, 2),
    ] {
        transaction.execute("INSERT INTO statuses (id, code, name, completed, display_order, is_default) VALUES (?1, ?2, ?3, ?4, ?5, 1)", params![Uuid::new_v4().to_string(), code, name, completed, order]).map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    }
    transaction
        .commit()
        .map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| Error::phase("DATABASE_INTEGRITY_FAILED", e))?;
    if integrity != "ok" {
        return Err(Error::phase(
            "DATABASE_INTEGRITY_FAILED",
            io::Error::other(integrity),
        ));
    }
    Ok(())
}

fn write_config(path: &Path, config: &Config) -> Result<(), Error> {
    let content =
        serde_json::to_vec_pretty(config).map_err(|e| Error::phase("CONFIG_WRITE_FAILED", e))?;
    fs::write(path, content).map_err(|e| Error::phase("CONFIG_WRITE_FAILED", e))
}

fn valid_workspace(workspace: &Path) -> Result<bool, Error> {
    let root_path = workspace.parent().ok_or_else(|| {
        Error::phase(
            "WORKSPACE_INSPECTION_FAILED",
            io::Error::other("workspace has no repository parent"),
        )
    })?;
    let root_path = root_path
        .canonicalize()
        .map_err(|error| Error::phase("WORKSPACE_INSPECTION_FAILED", error))?;
    let resolved = resolve_workspace(
        RepositoryRoot(root_path.clone()),
        WorktreeRoot(root_path),
        AccessIntent::ReadOnly,
    )?;
    let (id, prefix, created_at): (String, String, String) = resolved
        .connection
        .query_row(
            "SELECT repository_id, prefix, created_at FROM repository_metadata WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?;
    Ok(id == resolved.config.repository_id.to_string()
        && prefix == resolved.config.prefix
        && created_at == resolved.config.created_at)
}

pub fn add_stealth_rule(root: &Path) -> Result<bool, Error> {
    let path = root.join(".gitignore");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(Error::GitignoreUpdate(io::Error::other(
                ".gitignore must be a regular file",
            )));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::write(&path, format!("{STEALTH_RULE}\n")).map_err(Error::GitignoreUpdate)?;
            return Ok(true);
        }
        Err(error) => return Err(Error::GitignoreUpdate(error)),
    }
    let existing = fs::read_to_string(&path).map_err(Error::GitignoreUpdate)?;
    let ending = if existing.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    if existing
        .split(['\n', '\r'])
        .any(|line| line == STEALTH_RULE)
    {
        return Ok(false);
    }
    let separator = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        ending
    };
    fs::write(path, format!("{existing}{separator}{STEALTH_RULE}{ending}"))
        .map_err(Error::GitignoreUpdate)?;
    Ok(true)
}

pub fn uninstall(current: &Path, dry_run: bool) -> Result<UninstallResult, Error> {
    let root = RepositoryRoot::discover(current)?.repository_root;
    let mut planned = Vec::new();
    for entry in fs::read_dir(root.path()).map_err(|e| Error::phase("UNINSTALL_PLAN_FAILED", e))? {
        let entry = entry.map_err(|e| Error::phase("UNINSTALL_PLAN_FAILED", e))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".tbtm"
            || name.starts_with(".tbtm.backup-")
            || name.starts_with(".tbtm.staging-")
        {
            planned.push(entry.path());
        }
    }
    planned.sort();
    let gitignore = root.path().join(".gitignore");
    let stealth_lines_planned = count_stealth_lines(&gitignore)?;
    if dry_run {
        return Ok(UninstallResult {
            dry_run,
            planned,
            removed: vec![],
            failed: vec![],
            stealth_lines_planned,
            stealth_lines_removed: 0,
        });
    }
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    for path in &planned {
        match remove_entry(path) {
            Ok(()) => removed.push(path.clone()),
            Err(error) => failed.push(format!("{}: {error}", path.display())),
        }
    }
    let stealth_lines_removed = match remove_stealth_lines(&gitignore) {
        Ok(count) => count,
        Err(error) => {
            failed.push(format!("{}: {error}", gitignore.display()));
            0
        }
    };
    Ok(UninstallResult {
        dry_run,
        planned,
        removed,
        failed,
        stealth_lines_planned,
        stealth_lines_removed,
    })
}

fn remove_entry(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.file_type().is_file() {
        fs::remove_file(path)
    } else if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        Err(io::Error::other("unsupported filesystem entry"))
    }
}

fn count_stealth_lines(path: &Path) -> Result<usize, Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(fs::read_to_string(path)
            .map_err(|e| Error::phase("UNINSTALL_PLAN_FAILED", e))?
            .lines()
            .filter(|line| *line == STEALTH_RULE)
            .count()),
        Ok(_) => Ok(0),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(Error::phase("UNINSTALL_PLAN_FAILED", error)),
    }
}

fn remove_stealth_lines(path: &Path) -> io::Result<usize> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Ok(0);
    }
    let content = fs::read_to_string(path)?;
    let count = content.lines().filter(|line| *line == STEALTH_RULE).count();
    if count == 0 {
        return Ok(0);
    }
    let ending = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let trailing = content.ends_with('\n');
    let filtered: Vec<_> = content
        .lines()
        .filter(|line| *line != STEALTH_RULE)
        .collect();
    let mut replacement = filtered.join(ending);
    if trailing && !replacement.is_empty() {
        replacement.push_str(ending);
    }
    fs::write(path, replacement)?;
    Ok(count)
}

pub fn exit_code(error: &Error) -> i32 {
    match error {
        Error::DatabaseUnavailable { .. } => 1,
        Error::InvalidConfiguration { .. } => 2,
        Error::RepositoryNotInitialized { .. } => 3,
        Error::PermissionDenied { .. } => 5,
        Error::InvalidPrefix
        | Error::InvalidAgentName { .. }
        | Error::InvalidTaskTitle
        | Error::InvalidTaskType { .. }
        | Error::InvalidPriority
        | Error::InvalidEstimate
        | Error::InvalidExternalUrl { .. }
        | Error::InvalidCodeReference { .. }
        | Error::DuplicateTaskContext { .. }
        | Error::ConflictingArguments
        | Error::NoUpdateFields
        | Error::TaskArchived { .. }
        | Error::AlreadyInitialized
        | Error::InvalidInitialization => 2,
        Error::AgentNotFound { .. } | Error::StatusNotFound { .. } | Error::TaskNotFound { .. } => {
            3
        }
        Error::Phase {
            phase: "FORCE_CONFIRMATION_REQUIRED" | "CONFIRMATION_REQUIRED",
            ..
        } => 2,
        Error::Phase {
            phase: "UNINSTALL_PERMISSION_FAILED",
            ..
        } => 5,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::tempdir;

    #[test]
    fn normalizes_prefixes() {
        assert_eq!(
            normalize_prefix("The Broke Task Management").unwrap(),
            "the-broke-task-management"
        );
        assert_eq!(
            normalize_prefix("Quản Lý Công Việc").unwrap(),
            "quan-ly-cong-viec"
        );
        assert_eq!(normalize_prefix("alpha___beta").unwrap(), "alpha-beta");
        assert!(matches!(
            normalize_prefix("中文"),
            Err(Error::InvalidPrefix)
        ));
    }

    #[test]
    fn creates_matching_workspace() {
        let temp = tempdir().unwrap();
        let result = initialize(temp.path(), Some("Quản Lý"), true, false, false).unwrap();
        let config: Config =
            serde_json::from_slice(&fs::read(&result.config_path).unwrap()).unwrap();
        assert_eq!(config.prefix, "quan-ly");
        assert!(valid_workspace(&temp.path().join(".tbtm")).unwrap());
        assert_eq!(
            fs::read_to_string(temp.path().join(".gitignore")).unwrap(),
            "/.tbtm/\n"
        );
    }

    #[test]
    fn initializes_canonical_statuses_and_finds_them_by_exact_code() {
        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let connection = Connection::open(initialized.database_path).unwrap();

        let statuses = [
            ("to_do", "Todo", false, 0),
            ("in_progress", "In progress", false, 1),
            ("done", "Done", true, 2),
        ];
        for (code, name, completed, display_order) in statuses {
            let status = status::find_by_code(&connection, code).unwrap().unwrap();
            assert!(Uuid::parse_str(&status.id).is_ok());
            assert_eq!(status.code, code);
            assert_eq!(status.name, name);
            assert_eq!(status.completed, completed);
            assert_eq!(status.display_order, display_order);
            assert!(status.is_default);
        }
        assert!(
            status::find_by_code(&connection, "TO_DO")
                .unwrap()
                .is_none()
        );
        assert!(
            status::find_by_code(&connection, "unknown")
                .unwrap()
                .is_none()
        );
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM statuses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn status_code_constraints_accept_only_the_documented_grammar() {
        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let connection = Connection::open(initialized.database_path).unwrap();

        for (index, code) in ["a", "custom_2", "z9"].into_iter().enumerate() {
            connection
                .execute(
                    "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
                     VALUES (?1, ?2, ?3, 0, ?4, 0)",
                    params![
                        Uuid::new_v4().to_string(),
                        code,
                        format!("Valid {index}"),
                        10 + index as i64
                    ],
                )
                .unwrap();
        }

        for (index, code) in ["", "Todo", "2do", "has space", "has-hyphen", "é"]
            .into_iter()
            .enumerate()
        {
            let result = connection.execute(
                "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
                 VALUES (?1, ?2, ?3, 0, ?4, 0)",
                params![
                    Uuid::new_v4().to_string(),
                    code,
                    format!("Invalid {index}"),
                    20 + index as i64
                ],
            );
            assert!(result.is_err(), "invalid code was accepted: {code:?}");
        }

        let null_result = connection.execute(
            "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
             VALUES (?1, NULL, 'Null code', 0, 30, 0)",
            [Uuid::new_v4().to_string()],
        );
        assert!(null_result.is_err());
        let duplicate_result = connection.execute(
            "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
             VALUES (?1, 'to_do', 'Different name', 0, 31, 0)",
            [Uuid::new_v4().to_string()],
        );
        assert!(duplicate_result.is_err());
    }

    #[test]
    fn status_lookup_obeys_caller_transaction_control() {
        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let mut connection = Connection::open(initialized.database_path).unwrap();
        let transaction = connection.transaction().unwrap();
        transaction
            .execute(
                "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
                 VALUES (?1, 'review', 'Review', 0, 10, 0)",
                [Uuid::new_v4().to_string()],
            )
            .unwrap();

        assert!(
            status::find_by_code(&transaction, "review")
                .unwrap()
                .is_some()
        );
        transaction.rollback().unwrap();
        assert!(
            status::find_by_code(&connection, "review")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn uninstall_dry_run_does_not_write() {
        let temp = tempdir().unwrap();
        initialize(temp.path(), None, true, false, false).unwrap();
        let before = fs::read(temp.path().join(".gitignore")).unwrap();
        let result = uninstall(temp.path(), true).unwrap();
        assert_eq!(result.planned.len(), 1);
        assert_eq!(fs::read(temp.path().join(".gitignore")).unwrap(), before);
    }

    #[test]
    fn validates_schema_one_config_fields() {
        let path = Path::new("config.json");
        let base = Config {
            schema_version: 1,
            repository_id: Uuid::new_v4(),
            prefix: "example".to_owned(),
            database: DATABASE_FILE.to_owned(),
            created_at: "2026-08-12T12:00:00Z".to_owned(),
        };
        validate_config(&base, path).unwrap();

        for database in ["../tbtm.db", "nested/tbtm.db", "/tmp/tbtm.db", "other.db"] {
            let mut config = base.clone();
            config.database = database.to_owned();
            assert!(matches!(
                validate_config(&config, path),
                Err(Error::InvalidConfiguration {
                    check: "database",
                    ..
                })
            ));
        }

        let mut config = base.clone();
        config.prefix = "Not Normalized".to_owned();
        assert!(validate_config(&config, path).is_err());
        config = base.clone();
        config.created_at = "2026-08-12T12:00:00+07:00".to_owned();
        assert!(validate_config(&config, path).is_err());
        config = base;
        config.schema_version = 2;
        assert!(validate_config(&config, path).is_err());

        let unknown_field = serde_json::json!({
            "schemaVersion": 1,
            "repositoryId": Uuid::new_v4(),
            "prefix": "example",
            "database": "tbtm.db",
            "createdAt": "2026-08-12T12:00:00Z",
            "redirect": "outside.db"
        });
        assert!(serde_json::from_value::<Config>(unknown_field).is_err());
        let invalid_uuid = serde_json::json!({
            "schemaVersion": 1,
            "repositoryId": "not-a-uuid",
            "prefix": "example",
            "database": "tbtm.db",
            "createdAt": "2026-08-12T12:00:00Z"
        });
        assert!(serde_json::from_value::<Config>(invalid_uuid).is_err());
    }

    #[test]
    fn missing_database_is_not_created() {
        let temp = tempdir().unwrap();
        let initialized =
            initialize(temp.path(), Some("stored-prefix"), false, false, false).unwrap();
        fs::remove_file(&initialized.database_path).unwrap();

        let error = resolve_repository(temp.path(), AccessIntent::ReadOnly).unwrap_err();
        assert!(matches!(error, Error::DatabaseUnavailable { .. }));
        let error = resolve_repository(temp.path(), AccessIntent::ReadWrite).unwrap_err();
        assert!(matches!(error, Error::DatabaseUnavailable { .. }));
        assert!(!initialized.database_path.exists());
    }

    #[test]
    fn invalid_sqlite_is_database_unavailable() {
        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        fs::write(&initialized.database_path, b"this is not sqlite").unwrap();

        let error = resolve_repository(temp.path(), AccessIntent::ReadOnly).unwrap_err();

        assert!(matches!(error, Error::DatabaseUnavailable { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn config_symlink_is_invalid_configuration() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let outside = temp.path().join("outside-config.json");
        fs::rename(&initialized.config_path, &outside).unwrap();
        symlink(&outside, &initialized.config_path).unwrap();

        let error = resolve_repository(temp.path(), AccessIntent::ReadOnly).unwrap_err();

        assert!(matches!(
            error,
            Error::InvalidConfiguration {
                check: "config access",
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_database_is_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let original_permissions = fs::metadata(&initialized.database_path)
            .unwrap()
            .permissions();
        fs::set_permissions(
            &initialized.database_path,
            fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let error = resolve_repository(temp.path(), AccessIntent::ReadOnly).unwrap_err();

        fs::set_permissions(&initialized.database_path, original_permissions).unwrap();
        assert!(matches!(error, Error::PermissionDenied { .. }));
        assert_eq!(exit_code(&error), 5);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_git_marker_is_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().unwrap();
        git2::Repository::init(temp.path()).unwrap();
        let marker = temp.path().join(".git");
        let original_permissions = fs::metadata(&marker).unwrap().permissions();
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o000)).unwrap();

        let error = RepositoryRoot::discover(temp.path()).unwrap_err();

        fs::set_permissions(&marker, original_permissions).unwrap();
        assert!(matches!(error, Error::PermissionDenied { .. }));
        assert_eq!(exit_code(&error), 5);
    }

    #[test]
    fn health_is_read_only_and_uses_stored_identity() {
        let temp = tempdir().unwrap();
        let initialized =
            initialize(temp.path(), Some("stored-prefix"), false, false, false).unwrap();
        let config_before = fs::read(&initialized.config_path).unwrap();
        let database_before = fs::read(&initialized.database_path).unwrap();

        let health = inspect_repository_health(temp.path()).unwrap();

        assert_eq!(health.repository_id, initialized.repository_id);
        assert_eq!(health.prefix, "stored-prefix");
        assert_eq!(health.health, "healthy");
        assert_eq!(fs::read(&initialized.config_path).unwrap(), config_before);
        assert_eq!(
            fs::read(&initialized.database_path).unwrap(),
            database_before
        );
        let workspace_entries: Vec<_> = fs::read_dir(temp.path().join(".tbtm"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(workspace_entries.len(), 2);
    }

    #[test]
    fn health_reports_metadata_mismatches() {
        for (statement, expected) in [
            (
                "UPDATE repository_metadata SET repository_id = '00000000-0000-0000-0000-000000000000'",
                "repositoryId",
            ),
            (
                "UPDATE repository_metadata SET prefix = 'different'",
                "prefix",
            ),
            (
                "UPDATE repository_metadata SET created_at = '2000-01-01T00:00:00Z'",
                "createdAt",
            ),
        ] {
            let temp = tempdir().unwrap();
            let initialized = initialize(temp.path(), None, false, false, false).unwrap();
            let connection = Connection::open(&initialized.database_path).unwrap();
            connection.execute(statement, []).unwrap();
            drop(connection);
            let database_before = fs::read(&initialized.database_path).unwrap();

            let error = inspect_repository_health(temp.path()).unwrap_err();
            match error {
                Error::InvalidConfiguration {
                    mismatched_fields, ..
                } => assert_eq!(mismatched_fields, [expected]),
                other => panic!("unexpected error: {other}"),
            }
            assert_eq!(
                fs::read(&initialized.database_path).unwrap(),
                database_before
            );
        }
    }

    #[test]
    fn health_rejects_invalid_migration_ledger_without_writing() {
        for statement in [
            "DELETE FROM schema_migrations",
            "UPDATE schema_migrations SET version = 4 WHERE version = 3",
        ] {
            let temp = tempdir().unwrap();
            let initialized = initialize(temp.path(), None, false, false, false).unwrap();
            let connection = Connection::open(&initialized.database_path).unwrap();
            connection.execute(statement, []).unwrap();
            drop(connection);
            let database_before = fs::read(&initialized.database_path).unwrap();

            let error = inspect_repository_health(temp.path()).unwrap_err();

            assert!(matches!(
                error,
                Error::DatabaseUnavailable {
                    check: "schema metadata",
                    ..
                }
            ));
            assert_eq!(
                fs::read(&initialized.database_path).unwrap(),
                database_before
            );
        }
    }

    #[test]
    fn registers_distinct_agents_and_retries_identity_collisions() {
        let temp = tempdir().unwrap();
        initialize(temp.path(), None, false, false, false).unwrap();
        let first = register_agent(temp.path(), "Claude Agent").unwrap();
        let mut resolved = resolve_repository(temp.path(), AccessIntent::ReadWrite).unwrap();
        let replacement = Uuid::new_v4();
        let mut ids = [first.id, replacement].into_iter();

        let second = register_agent_with(&mut resolved, "claude-agent".to_owned(), || {
            ids.next().unwrap()
        })
        .unwrap();

        assert_eq!(first.base_name, "claude-agent");
        assert_eq!(second.id, replacement);
        assert_ne!(first.display_name, second.display_name);
        let count: i64 = resolved
            .connection()
            .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn exhausted_identity_retries_leave_no_partial_agent() {
        let temp = tempdir().unwrap();
        initialize(temp.path(), None, false, false, false).unwrap();
        let first = register_agent(temp.path(), "agent").unwrap();
        let mut resolved = resolve_repository(temp.path(), AccessIntent::ReadWrite).unwrap();

        let error =
            register_agent_with(&mut resolved, "agent".to_owned(), || first.id).unwrap_err();

        assert_eq!(error.code(), "AGENT_REGISTRATION_FAILED");
        let count: i64 = resolved
            .connection()
            .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn invalid_agent_names_report_normalized_value() {
        assert!(matches!(
            normalize_agent_name("中文"),
            Err(Error::InvalidAgentName { normalized }) if normalized.is_empty()
        ));
        let too_long = "a".repeat(49);
        assert!(matches!(
            normalize_agent_name(&too_long),
            Err(Error::InvalidAgentName { normalized }) if normalized == too_long
        ));
    }

    #[test]
    fn failed_quick_check_is_read_only_and_database_unavailable() {
        let temp = tempdir().unwrap();
        let initialized = initialize(temp.path(), None, false, false, false).unwrap();
        let connection = Connection::open(&initialized.database_path).unwrap();
        let page_size: i64 = connection
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .unwrap();
        let statuses_root_page: i64 = connection
            .query_row(
                "SELECT rootpage FROM sqlite_schema WHERE name = 'statuses'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        drop(connection);
        let mut database = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&initialized.database_path)
            .unwrap();
        database
            .seek(SeekFrom::Start(
                ((statuses_root_page - 1) * page_size).try_into().unwrap(),
            ))
            .unwrap();
        database.write_all(&[0xff]).unwrap();
        database.sync_all().unwrap();
        drop(database);
        let config_before = fs::read(&initialized.config_path).unwrap();
        let database_before = fs::read(&initialized.database_path).unwrap();
        let mut entries_before: Vec<_> = fs::read_dir(temp.path().join(".tbtm"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        entries_before.sort();

        let error = inspect_repository_health(temp.path()).unwrap_err();

        assert!(matches!(
            error,
            Error::DatabaseUnavailable {
                check: "quick_check",
                ..
            }
        ));
        assert_eq!(fs::read(&initialized.config_path).unwrap(), config_before);
        assert_eq!(
            fs::read(&initialized.database_path).unwrap(),
            database_before
        );
        let mut entries_after: Vec<_> = fs::read_dir(temp.path().join(".tbtm"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        entries_after.sort();
        assert_eq!(entries_after, entries_before);
    }

    #[test]
    fn repository_rename_keeps_stored_identity_and_prefix() {
        let parent = tempdir().unwrap();
        let original = parent.path().join("original-name");
        let renamed = parent.path().join("renamed-directory");
        fs::create_dir(&original).unwrap();
        let initialized =
            initialize(&original, Some("stored-prefix"), false, false, false).unwrap();
        fs::rename(&original, &renamed).unwrap();

        let health = inspect_repository_health(&renamed).unwrap();

        assert_eq!(health.repository_id, initialized.repository_id);
        assert_eq!(health.prefix, "stored-prefix");
    }

    #[test]
    fn resolves_nearest_git_root_from_nested_directory() {
        let temp = tempdir().unwrap();
        git2::Repository::init(temp.path()).unwrap();
        let initialized = initialize(temp.path(), Some("git-prefix"), false, false, false).unwrap();
        let nested = temp.path().join("one/two");
        fs::create_dir_all(&nested).unwrap();

        let resolved = resolve_repository(&nested, AccessIntent::ReadOnly).unwrap();

        assert_eq!(
            resolved.repository_root,
            temp.path().canonicalize().unwrap()
        );
        assert_eq!(resolved.config.repository_id, initialized.repository_id);
        assert_eq!(resolved.config.prefix, "git-prefix");
    }
}
