use deunicode::deunicode;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use unicode_normalization::UnicodeNormalization;
use unicode_script::{Script, UnicodeScript};
use uuid::Uuid;

const CONFIG_FILE: &str = "config.json";
const DATABASE_FILE: &str = "tbtm.db";
const STEALTH_RULE: &str = "/.tbtm/";

#[derive(Debug, Error)]
pub enum Error {
    #[error("INVALID_PREFIX: prefix must normalize to 1–48 ASCII characters")]
    InvalidPrefix,
    #[error("ALREADY_INITIALIZED: valid TBTM workspace already exists")]
    AlreadyInitialized,
    #[error("INVALID_INITIALIZATION: existing .tbtm is missing, corrupt, or mismatched")]
    InvalidInitialization,
    #[error("GITIGNORE_UPDATE_FAILED: {0}")]
    GitignoreUpdate(#[source] io::Error),
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
            Self::AlreadyInitialized => "ALREADY_INITIALIZED",
            Self::InvalidInitialization => "INVALID_INITIALIZATION",
            Self::GitignoreUpdate(_) => "GITIGNORE_UPDATE_FAILED",
            Self::Phase { phase, .. } => phase,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryRoot(PathBuf);

impl RepositoryRoot {
    pub fn discover(current: &Path) -> Result<Self, Error> {
        let canonical = current
            .canonicalize()
            .map_err(|e| Error::phase("REPOSITORY_DISCOVERY_FAILED", e))?;
        let root = git2::Repository::discover(&canonical)
            .ok()
            .and_then(|repo| repo.workdir().map(Path::to_path_buf))
            .unwrap_or(canonical);
        Ok(Self(root))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
    fn workspace(&self) -> PathBuf {
        self.0.join(".tbtm")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub schema_version: u32,
    pub repository_id: Uuid,
    pub prefix: String,
    pub database: String,
    pub created_at: String,
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

pub fn normalize_prefix(input: &str) -> Result<String, Error> {
    if input
        .chars()
        .any(|character| character.is_alphabetic() && character.script() != Script::Latin)
    {
        return Err(Error::InvalidPrefix);
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
        return Err(Error::InvalidPrefix);
    }
    Ok(value)
}

pub fn initialize(
    current: &Path,
    prefix_input: Option<&str>,
    stealth: bool,
    force: bool,
    force_confirmed: bool,
) -> Result<InitResult, Error> {
    let root = RepositoryRoot::discover(current)?;
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
    transaction.execute("INSERT INTO repository_metadata (singleton, repository_id, prefix, created_at) VALUES (1, ?1, ?2, ?3)", params![repository_id.to_string(), prefix, created_at]).map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
    for (name, completed, order) in [
        ("Todo", false, 0),
        ("In progress", false, 1),
        ("Done", true, 2),
    ] {
        transaction.execute("INSERT INTO statuses (id, name, completed, display_order, is_default) VALUES (?1, ?2, ?3, ?4, 1)", params![Uuid::new_v4().to_string(), name, completed, order]).map_err(|e| Error::phase("DATABASE_MIGRATION_FAILED", e))?;
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
    if fs::symlink_metadata(workspace)
        .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?
        .file_type()
        .is_symlink()
    {
        return Ok(false);
    }
    let config: Config = serde_json::from_slice(
        &fs::read(workspace.join(CONFIG_FILE))
            .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?,
    )
    .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?;
    if config.schema_version != 1 || config.database != DATABASE_FILE {
        return Ok(false);
    }
    let connection = Connection::open(workspace.join(DATABASE_FILE))
        .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?;
    let (id, prefix, created_at): (String, String, String) = connection
        .query_row(
            "SELECT repository_id, prefix, created_at FROM repository_metadata WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| Error::phase("WORKSPACE_INSPECTION_FAILED", e))?;
    Ok(id == config.repository_id.to_string()
        && prefix == config.prefix
        && created_at == config.created_at)
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
    let root = RepositoryRoot::discover(current)?;
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
        Error::InvalidPrefix | Error::AlreadyInitialized | Error::InvalidInitialization => 2,
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
    fn uninstall_dry_run_does_not_write() {
        let temp = tempdir().unwrap();
        initialize(temp.path(), None, true, false, false).unwrap();
        let before = fs::read(temp.path().join(".gitignore")).unwrap();
        let result = uninstall(temp.path(), true).unwrap();
        assert_eq!(result.planned.len(), 1);
        assert_eq!(fs::read(temp.path().join(".gitignore")).unwrap(), before);
    }
}
