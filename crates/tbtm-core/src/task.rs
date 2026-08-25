use crate::{
    AccessIntent, Error, apply_pending_migrations, resolve_repository, sqlite_access_error, status,
    utc_now,
};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::Serialize;
use std::{collections::HashSet, path::Path};
use url::Url;
use uuid::Uuid;

const TASK_ID_RETRY_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Epic,
    Story,
    Task,
    Improvement,
    Refactor,
    Bug,
    Spike,
    Testing,
    Poc,
}

impl TaskType {
    pub fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "epic" => Ok(Self::Epic),
            "story" => Ok(Self::Story),
            "task" => Ok(Self::Task),
            "improvement" => Ok(Self::Improvement),
            "refactor" => Ok(Self::Refactor),
            "bug" => Ok(Self::Bug),
            "spike" => Ok(Self::Spike),
            "testing" => Ok(Self::Testing),
            "poc" => Ok(Self::Poc),
            _ => Err(Error::InvalidTaskType {
                value: value.to_owned(),
            }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Epic => "epic",
            Self::Story => "story",
            Self::Task => "task",
            Self::Improvement => "improvement",
            Self::Refactor => "refactor",
            Self::Bug => "bug",
            Self::Spike => "spike",
            Self::Testing => "testing",
            Self::Poc => "poc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct CodeReference {
    pub path: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub title: String,
    pub task_type: TaskType,
    pub description: String,
    pub goal: String,
    pub acceptance_criteria: String,
    pub status_code: String,
    pub priority: i64,
    pub estimate: Option<f64>,
    pub tags: Vec<String>,
    pub external_urls: Vec<String>,
    pub code_references: Vec<CodeReference>,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub id: String,
    pub code: String,
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub goal: String,
    pub acceptance_criteria: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: i64,
    pub estimate: Option<f64>,
    pub tags: Vec<String>,
    pub external_urls: Vec<String>,
    pub code_references: Vec<CodeReference>,
    pub parent_id: Option<String>,
    pub dependencies: Vec<String>,
    pub claim: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub updated_by: String,
}

pub fn parse_code_reference(value: &str) -> Result<CodeReference, Error> {
    let (location, description) = match value.split_once("::") {
        Some((location, description)) if !description.is_empty() && !description.contains("::") => {
            (location, Some(description.to_owned()))
        }
        Some(_) => {
            return Err(Error::InvalidCodeReference {
                value: value.to_owned(),
            });
        }
        None => (value, None),
    };
    let (path, start_line, end_line) = match location.rsplit_once(':') {
        Some((path, range))
            if range
                .chars()
                .all(|character| character.is_ascii_digit() || character == '-') =>
        {
            let (start, end) = parse_range(range).ok_or_else(|| Error::InvalidCodeReference {
                value: value.to_owned(),
            })?;
            (path, Some(start), end)
        }
        _ => (location, None, None),
    };
    validate_relative_path(path, value)?;
    Ok(CodeReference {
        path: path.to_owned(),
        start_line,
        end_line,
        description,
    })
}

fn parse_range(value: &str) -> Option<(u32, Option<u32>)> {
    let (start, end) = match value.split_once('-') {
        Some((start, end)) => (start.parse().ok()?, Some(end.parse().ok()?)),
        None => (value.parse().ok()?, None),
    };
    if start == 0 || end.is_some_and(|end| end == 0 || end < start) {
        return None;
    }
    Some((start, end))
}

fn validate_relative_path(path: &str, original: &str) -> Result<(), Error> {
    let unsafe_path = path.trim().is_empty()
        || path.starts_with(['/', '\\'])
        || path.as_bytes().get(1) == Some(&b':')
        || path
            .split(['/', '\\'])
            .any(|part| part == ".." || part.is_empty());
    if unsafe_path {
        Err(Error::InvalidCodeReference {
            value: original.to_owned(),
        })
    } else {
        Ok(())
    }
}

pub fn create_task(current: &Path, mut input: CreateTaskInput) -> Result<CreatedTask, Error> {
    validate_input(&mut input)?;
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection_mut()
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    for _ in 0..TASK_ID_RETRY_LIMIT {
        let suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
        let id = format!(
            "{}-{}-{suffix}",
            resolved.config.prefix,
            input.task_type.as_str()
        );
        match insert_task(&mut resolved, &input, id, suffix) {
            Err(error) if is_task_id_collision(&error) => continue,
            result => return result,
        }
    }
    Err(Error::phase(
        "TASK_CREATION_FAILED",
        std::io::Error::other("unique task ID retry limit exhausted"),
    ))
}

fn validate_input(input: &mut CreateTaskInput) -> Result<(), Error> {
    input.title = input.title.trim().to_owned();
    if input.title.is_empty() {
        return Err(Error::InvalidTaskTitle);
    }
    if !(0..=1_000_000).contains(&input.priority) {
        return Err(Error::InvalidPriority);
    }
    if input
        .estimate
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(Error::InvalidEstimate);
    }
    let mut tags = HashSet::new();
    for tag in &mut input.tags {
        *tag = tag.trim().to_owned();
        if tag.is_empty() || !tags.insert(tag.clone()) {
            return Err(Error::DuplicateTaskContext { value: tag.clone() });
        }
    }
    let mut urls = HashSet::new();
    for value in &input.external_urls {
        let parsed = Url::parse(value).map_err(|_| Error::InvalidExternalUrl {
            value: value.clone(),
        })?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(Error::InvalidExternalUrl {
                value: value.clone(),
            });
        }
        if !urls.insert(value.clone()) {
            return Err(Error::DuplicateTaskContext {
                value: value.clone(),
            });
        }
    }
    let mut references = HashSet::new();
    for reference in &input.code_references {
        validate_relative_path(&reference.path, &reference.path)?;
        if reference.start_line == Some(0)
            || reference.end_line == Some(0)
            || reference
                .end_line
                .zip(reference.start_line)
                .is_some_and(|(end, start)| end < start)
            || !references.insert(reference.clone())
        {
            return Err(Error::DuplicateTaskContext {
                value: reference.path.clone(),
            });
        }
    }
    Ok(())
}

fn insert_task(
    resolved: &mut crate::ResolvedRepository,
    input: &CreateTaskInput,
    id: String,
    suffix: String,
) -> Result<CreatedTask, Error> {
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("task creation", &database_path, error))?;
    let found_status = status::find_by_code(&transaction, &input.status_code)
        .map_err(|error| sqlite_access_error("task status lookup", &database_path, error))?
        .ok_or_else(|| Error::StatusNotFound {
            code: input.status_code.clone(),
        })?;
    if let Some(agent_id) = input.agent_id {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM agents WHERE id = ?1",
                [agent_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| sqlite_access_error("task actor lookup", &database_path, error))?
            .is_some();
        if !exists {
            return Err(Error::AgentNotFound { id: agent_id });
        }
    }
    let actor_type = if input.agent_id.is_some() {
        "agent"
    } else {
        "user"
    };
    let actor_id = input.agent_id.map(|id| id.to_string());
    let actor = actor_id.clone().unwrap_or_else(|| "user".to_owned());
    let timestamp = utc_now();
    transaction.execute(
        "INSERT INTO tasks (id, short_suffix, task_type, title, description, goal, acceptance_criteria, status_id, priority, estimate_hours, archived, created_actor_type, created_agent_id, updated_actor_type, updated_agent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11, ?12, ?11, ?12, ?13, ?13)",
        params![id, suffix, input.task_type.as_str(), input.title, input.description, input.goal, input.acceptance_criteria, found_status.id, input.priority, input.estimate, actor_type, actor_id, timestamp],
    ).map_err(|error| sqlite_access_error("task creation", &database_path, error))?;
    for (ordinal, tag) in input.tags.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO task_tags (task_id, ordinal, value) VALUES (?1, ?2, ?3)",
                params![id, ordinal as i64, tag],
            )
            .map_err(|error| sqlite_access_error("task tag creation", &database_path, error))?;
    }
    for (ordinal, url) in input.external_urls.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO task_external_urls (task_id, ordinal, url) VALUES (?1, ?2, ?3)",
                params![id, ordinal as i64, url],
            )
            .map_err(|error| sqlite_access_error("task URL creation", &database_path, error))?;
    }
    for (ordinal, reference) in input.code_references.iter().enumerate() {
        transaction.execute("INSERT INTO task_code_references (task_id, ordinal, path, start_line, end_line, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![id, ordinal as i64, reference.path, reference.start_line, reference.end_line, reference.description])
            .map_err(|error| sqlite_access_error("task code reference creation", &database_path, error))?;
    }
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task creation", &database_path, error))?;
    Ok(CreatedTask {
        id,
        title: input.title.clone(),
        description: input.description.clone(),
        goal: input.goal.clone(),
        acceptance_criteria: input.acceptance_criteria.clone(),
        task_type: input.task_type,
        status: TaskStatus {
            id: found_status.id,
            code: found_status.code,
            name: found_status.name,
            completed: found_status.completed,
        },
        priority: input.priority,
        estimate: input.estimate,
        tags: input.tags.clone(),
        external_urls: input.external_urls.clone(),
        code_references: input.code_references.clone(),
        parent_id: None,
        dependencies: vec![],
        claim: None,
        archived: false,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        created_by: actor.clone(),
        updated_by: actor,
    })
}

fn is_task_id_collision(error: &Error) -> bool {
    matches!(error, Error::DatabaseUnavailable { message, .. } if message.contains("tasks.id") || message.contains("tasks.short_suffix"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_types_and_code_references() {
        assert_eq!(TaskType::parse("poc").unwrap(), TaskType::Poc);
        assert!(TaskType::parse("feature").is_err());
        assert_eq!(
            parse_code_reference("src/lib.rs:10-12::parser").unwrap(),
            CodeReference {
                path: "src/lib.rs".to_owned(),
                start_line: Some(10),
                end_line: Some(12),
                description: Some("parser".to_owned())
            }
        );
        assert_eq!(
            parse_code_reference("docs/a:b.md").unwrap().path,
            "docs/a:b.md"
        );
        assert!(parse_code_reference("../secret:1").is_err());
        assert!(parse_code_reference("src/lib.rs:0").is_err());
    }
}
