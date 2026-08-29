use crate::{
    AccessIntent, Error, apply_pending_migrations, require_latest_migration, resolve_repository,
    sqlite_access_error, status, utc_now,
};
use rusqlite::{
    OptionalExtension, Transaction, TransactionBehavior, params, params_from_iter, types::Value,
};
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
pub struct TaskHierarchy {
    pub parent: Option<RelatedTask>,
    pub children: Vec<RelatedTask>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependencies {
    pub upstream: Vec<RelatedTask>,
    pub downstream: Vec<RelatedTask>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTask {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: RelatedTaskStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTaskStatus {
    pub code: String,
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskClaim {
    pub agent: ClaimAgent,
    pub claimed_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimAgent {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullTask {
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
    pub hierarchy: TaskHierarchy,
    pub dependencies: TaskDependencies,
    pub claim: Option<TaskClaim>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub updated_by: String,
}

pub type CreatedTask = FullTask;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchValue<T> {
    Omitted,
    Set(T),
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTaskInput {
    pub estimate: PatchValue<f64>,
    pub tags: PatchValue<Vec<String>>,
    pub external_urls: PatchValue<Vec<String>>,
    pub code_references: PatchValue<Vec<CodeReference>>,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct DependencyInput {
    pub task_id: String,
    pub depends_on: String,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ClaimTaskInput {
    pub task_id: String,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyResult {
    pub task_id: String,
    pub depends_on: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveScope {
    Active,
    Archived,
    All,
}

#[derive(Debug, Clone)]
pub struct ListTasksInput {
    pub archive_scope: ArchiveScope,
    pub status_codes: Vec<String>,
    pub task_types: Vec<TaskType>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListItem {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: i64,
    pub estimate: Option<f64>,
    pub tags: Vec<String>,
    pub archived: bool,
    pub claim: Option<TaskClaim>,
    pub created_at: String,
    pub updated_at: String,
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
        hierarchy: empty_hierarchy(),
        dependencies: empty_dependencies(),
        claim: None,
        archived: false,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        created_by: actor.clone(),
        updated_by: actor,
    })
}

fn empty_hierarchy() -> TaskHierarchy {
    TaskHierarchy {
        parent: None,
        children: vec![],
    }
}

fn empty_dependencies() -> TaskDependencies {
    TaskDependencies {
        upstream: vec![],
        downstream: vec![],
    }
}

pub fn view_task(current: &Path, id: &str) -> Result<FullTask, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("task detail snapshot", &database_path, error))?;
    let mut task = load_full_task(&transaction, id, &database_path)?
        .ok_or_else(|| Error::TaskNotFound { id: id.to_owned() })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task detail snapshot", &database_path, error))?;
    Ok(task)
}

pub fn claim_task(current: &Path, input: ClaimTaskInput) -> Result<FullTask, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection_mut()
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("task claim", &database_path, error))?;
    let agent_id = input.agent_id.to_string();
    let agent_exists = transaction
        .query_row(
            "SELECT 1 FROM agents WHERE id = ?1",
            [&agent_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("claim agent lookup", &database_path, error))?
        .is_some();
    if !agent_exists {
        return Err(Error::AgentNotFound { id: input.agent_id });
    }
    let (archived, completed) = transaction
        .query_row(
            "SELECT t.archived, s.completed FROM tasks t JOIN statuses s ON s.id = t.status_id WHERE t.id = ?1",
            [&input.task_id],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("claim task lookup", &database_path, error))?
        .ok_or_else(|| Error::TaskNotFound { id: input.task_id.clone() })?;
    if let Some((owner_id, owner_name, claimed_at)) = transaction
        .query_row(
            "SELECT a.id, a.display_name, c.claimed_at FROM task_claims c JOIN agents a ON a.id = c.agent_id WHERE c.task_id = ?1",
            [&input.task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("claim owner lookup", &database_path, error))?
    {
        return Err(Error::ClaimConflict {
            task_id: input.task_id,
            agent_id: owner_id,
            agent_display_name: owner_name,
            claimed_at,
        });
    }
    if archived {
        return Err(task_not_available(&input.task_id, "archived", vec![]));
    }
    if completed {
        return Err(task_not_available(&input.task_id, "completed", vec![]));
    }
    let unresolved = load_strings(
        &transaction,
        "SELECT upstream.id FROM task_dependencies d JOIN tasks upstream ON upstream.id = d.upstream_task_id JOIN statuses s ON s.id = upstream.status_id WHERE d.downstream_task_id = ?1 AND upstream.archived = 0 AND s.completed = 0 ORDER BY upstream.id",
        &input.task_id,
        &database_path,
    )?;
    if !unresolved.is_empty() {
        return Err(task_not_available(
            &input.task_id,
            "dependencies_blocked",
            unresolved,
        ));
    }
    let claimed_at = utc_now();
    transaction
        .execute(
            "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, ?2, ?3)",
            params![input.task_id, agent_id, claimed_at],
        )
        .map_err(|error| sqlite_access_error("task claim", &database_path, error))?;
    let mut task =
        load_full_task(&transaction, &input.task_id, &database_path)?.ok_or_else(|| {
            Error::TaskNotFound {
                id: input.task_id.clone(),
            }
        })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task claim", &database_path, error))?;
    Ok(task)
}

fn task_not_available(task_id: &str, reason: &str, unresolved_upstream_ids: Vec<String>) -> Error {
    Error::TaskNotAvailable {
        task_id: task_id.to_owned(),
        reason: reason.to_owned(),
        unresolved_upstream_ids,
    }
}

pub fn add_dependency(current: &Path, input: DependencyInput) -> Result<DependencyResult, Error> {
    mutate_dependency(current, input, true)
}

pub fn remove_dependency(
    current: &Path,
    input: DependencyInput,
) -> Result<DependencyResult, Error> {
    mutate_dependency(current, input, false)
}

fn mutate_dependency(
    current: &Path,
    input: DependencyInput,
    adding: bool,
) -> Result<DependencyResult, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection_mut()
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("dependency mutation", &database_path, error))?;
    if let Some(agent_id) = input.agent_id {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM agents WHERE id = ?1",
                [agent_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| sqlite_access_error("dependency actor lookup", &database_path, error))?
            .is_some();
        if !exists {
            return Err(Error::AgentNotFound { id: agent_id });
        }
    }
    let downstream_archived = transaction
        .query_row(
            "SELECT archived FROM tasks WHERE id = ?1",
            [&input.task_id],
            |row| row.get::<_, bool>(0),
        )
        .optional()
        .map_err(|error| sqlite_access_error("dependency task lookup", &database_path, error))?
        .ok_or_else(|| Error::TaskNotFound {
            id: input.task_id.clone(),
        })?;
    if downstream_archived {
        return Err(Error::TaskArchived {
            id: input.task_id.clone(),
        });
    }
    let upstream_exists = transaction
        .query_row(
            "SELECT 1 FROM tasks WHERE id = ?1",
            [&input.depends_on],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("dependency task lookup", &database_path, error))?
        .is_some();
    if !upstream_exists {
        return Err(Error::TaskNotFound {
            id: input.depends_on.clone(),
        });
    }
    if input.task_id == input.depends_on {
        return Err(Error::SelfDependency {
            task_id: input.task_id,
        });
    }
    let edge_exists = transaction
        .query_row(
            "SELECT 1 FROM task_dependencies WHERE downstream_task_id = ?1 AND upstream_task_id = ?2",
            params![input.task_id, input.depends_on],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("dependency edge lookup", &database_path, error))?
        .is_some();
    if adding && edge_exists {
        return Err(Error::DependencyExists {
            task_id: input.task_id,
            depends_on: input.depends_on,
        });
    }
    if !adding && !edge_exists {
        return Err(Error::DependencyNotFound {
            task_id: input.task_id,
            depends_on: input.depends_on,
        });
    }
    if adding
        && would_create_cycle(
            &transaction,
            &input.task_id,
            &input.depends_on,
            &database_path,
        )?
    {
        return Err(Error::DependencyCycle {
            task_id: input.task_id,
            depends_on: input.depends_on,
        });
    }
    let (sql, phase) = if adding {
        (
            "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
            "dependency addition",
        )
    } else {
        (
            "DELETE FROM task_dependencies WHERE downstream_task_id = ?1 AND upstream_task_id = ?2",
            "dependency removal",
        )
    };
    transaction
        .execute(sql, params![input.task_id, input.depends_on])
        .map_err(|error| sqlite_access_error(phase, &database_path, error))?;
    let actor_type = if input.agent_id.is_some() {
        "agent"
    } else {
        "user"
    };
    let actor_id = input.agent_id.map(|value| value.to_string());
    transaction.execute(
        "UPDATE tasks SET updated_at = ?1, updated_actor_type = ?2, updated_agent_id = ?3 WHERE id = ?4",
        params![utc_now(), actor_type, actor_id, input.task_id],
    ).map_err(|error| sqlite_access_error("dependency metadata update", &database_path, error))?;
    let result = DependencyResult {
        task_id: input.task_id,
        depends_on: input.depends_on,
    };
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("dependency mutation", &database_path, error))?;
    Ok(result)
}

fn would_create_cycle(
    transaction: &Transaction<'_>,
    task_id: &str,
    depends_on: &str,
    database_path: &Path,
) -> Result<bool, Error> {
    transaction.query_row(
        "WITH RECURSIVE ancestors(id) AS (
            SELECT upstream_task_id FROM task_dependencies WHERE downstream_task_id = ?1
            UNION
            SELECT d.upstream_task_id FROM task_dependencies d JOIN ancestors a ON d.downstream_task_id = a.id
         ) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = ?2)",
        params![depends_on, task_id],
        |row| row.get(0),
    ).map_err(|error| sqlite_access_error("dependency cycle validation", database_path, error))
}

pub fn dependencies_satisfied(
    connection: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<bool> {
    connection.query_row(
        "SELECT NOT EXISTS(
            SELECT 1 FROM task_dependencies d
            JOIN tasks upstream ON upstream.id = d.upstream_task_id
            JOIN statuses s ON s.id = upstream.status_id
            WHERE d.downstream_task_id = ?1 AND upstream.archived = 0 AND s.completed = 0
        )",
        [task_id],
        |row| row.get(0),
    )
}

pub fn update_task(
    current: &Path,
    id: &str,
    mut input: UpdateTaskInput,
) -> Result<FullTask, Error> {
    if matches!(input.estimate, PatchValue::Omitted)
        && matches!(input.tags, PatchValue::Omitted)
        && matches!(input.external_urls, PatchValue::Omitted)
        && matches!(input.code_references, PatchValue::Omitted)
    {
        return Err(Error::NoUpdateFields);
    }
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection_mut()
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("task update", &database_path, error))?;
    let mut task = load_full_task(&transaction, id, &database_path)?
        .ok_or_else(|| Error::TaskNotFound { id: id.to_owned() })?;
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
    normalize_update_input(&mut input)?;
    if task.archived {
        return Err(Error::TaskArchived { id: id.to_owned() });
    }
    load_full_context(&transaction, &mut task, &database_path)?;
    let estimate = patch_target(&input.estimate, task.estimate);
    let tags = collection_target(&input.tags);
    let urls = collection_target(&input.external_urls);
    let references = collection_target(&input.code_references);
    let changed = estimate != task.estimate
        || tags.as_ref().is_some_and(|value| value != &task.tags)
        || urls
            .as_ref()
            .is_some_and(|value| value != &task.external_urls)
        || references
            .as_ref()
            .is_some_and(|value| value != &task.code_references);
    if changed {
        if estimate != task.estimate {
            transaction
                .execute(
                    "UPDATE tasks SET estimate_hours = ?1 WHERE id = ?2",
                    params![estimate, id],
                )
                .map_err(|error| {
                    sqlite_access_error("task estimate update", &database_path, error)
                })?;
        }
        replace_strings(
            &transaction,
            id,
            "task_tags",
            "value",
            tags.as_ref().filter(|v| *v != &task.tags),
            &database_path,
        )?;
        replace_strings(
            &transaction,
            id,
            "task_external_urls",
            "url",
            urls.as_ref().filter(|v| *v != &task.external_urls),
            &database_path,
        )?;
        if let Some(values) = references.as_ref().filter(|v| *v != &task.code_references) {
            transaction
                .execute("DELETE FROM task_code_references WHERE task_id = ?1", [id])
                .map_err(|error| {
                    sqlite_access_error("task code reference replacement", &database_path, error)
                })?;
            for (ordinal, value) in values.iter().enumerate() {
                transaction.execute("INSERT INTO task_code_references (task_id, ordinal, path, start_line, end_line, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![id, ordinal as i64, value.path, value.start_line, value.end_line, value.description])
                    .map_err(|error| sqlite_access_error("task code reference replacement", &database_path, error))?;
            }
        }
        let actor_type = if input.agent_id.is_some() {
            "agent"
        } else {
            "user"
        };
        let actor_id = input.agent_id.map(|value| value.to_string());
        transaction.execute("UPDATE tasks SET updated_actor_type = ?1, updated_agent_id = ?2, updated_at = ?3 WHERE id = ?4", params![actor_type, actor_id, utc_now(), id])
            .map_err(|error| sqlite_access_error("task update metadata", &database_path, error))?;
        task = load_full_task(&transaction, id, &database_path)?.expect("updated task exists");
        load_full_context(&transaction, &mut task, &database_path)?;
    }
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task update", &database_path, error))?;
    Ok(task)
}

fn patch_target<T: Copy>(patch: &PatchValue<T>, current: Option<T>) -> Option<T> {
    match patch {
        PatchValue::Omitted => current,
        PatchValue::Set(value) => Some(*value),
        PatchValue::Clear => None,
    }
}

fn collection_target<T: Clone>(patch: &PatchValue<Vec<T>>) -> Option<Vec<T>> {
    match patch {
        PatchValue::Omitted => None,
        PatchValue::Set(value) => Some(value.clone()),
        PatchValue::Clear => Some(vec![]),
    }
}

fn replace_strings(
    transaction: &Transaction<'_>,
    id: &str,
    table: &str,
    column: &str,
    values: Option<&Vec<String>>,
    database_path: &Path,
) -> Result<(), Error> {
    let Some(values) = values else {
        return Ok(());
    };
    transaction
        .execute(&format!("DELETE FROM {table} WHERE task_id = ?1"), [id])
        .map_err(|error| sqlite_access_error("task context replacement", database_path, error))?;
    let sql = format!("INSERT INTO {table} (task_id, ordinal, {column}) VALUES (?1, ?2, ?3)");
    for (ordinal, value) in values.iter().enumerate() {
        transaction
            .execute(&sql, params![id, ordinal as i64, value])
            .map_err(|error| {
                sqlite_access_error("task context replacement", database_path, error)
            })?;
    }
    Ok(())
}

fn normalize_update_input(input: &mut UpdateTaskInput) -> Result<(), Error> {
    if let PatchValue::Set(value) = input.estimate
        && (!value.is_finite() || value < 0.0)
    {
        return Err(Error::InvalidEstimate);
    }
    if let PatchValue::Set(tags) = &mut input.tags {
        let mut seen = HashSet::new();
        for tag in tags {
            *tag = tag.trim().to_owned();
            if tag.is_empty() || !seen.insert(tag.clone()) {
                return Err(Error::DuplicateTaskContext { value: tag.clone() });
            }
        }
    }
    if let PatchValue::Set(urls) = &input.external_urls {
        let mut seen = HashSet::new();
        for value in urls {
            let parsed = Url::parse(value).map_err(|_| Error::InvalidExternalUrl {
                value: value.clone(),
            })?;
            if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
                return Err(Error::InvalidExternalUrl {
                    value: value.clone(),
                });
            }
            if !seen.insert(value.clone()) {
                return Err(Error::DuplicateTaskContext {
                    value: value.clone(),
                });
            }
        }
    }
    if let PatchValue::Set(references) = &input.code_references {
        let mut seen = HashSet::new();
        for value in references {
            validate_relative_path(&value.path, &value.path)?;
            if value.start_line == Some(0)
                || value.end_line == Some(0)
                || value
                    .end_line
                    .zip(value.start_line)
                    .is_some_and(|(end, start)| end < start)
            {
                return Err(Error::InvalidCodeReference {
                    value: value.path.clone(),
                });
            }
            if !seen.insert(value.clone()) {
                return Err(Error::DuplicateTaskContext {
                    value: value.path.clone(),
                });
            }
        }
    }
    Ok(())
}

fn load_full_task(
    transaction: &Transaction<'_>,
    id: &str,
    database_path: &Path,
) -> Result<Option<FullTask>, Error> {
    transaction
        .query_row(
            "SELECT t.id, t.title, t.description, t.goal, t.acceptance_criteria,
                t.task_type, s.id, s.code, s.name, s.completed, t.priority,
                t.estimate_hours, t.archived, t.created_at, t.updated_at,
                CASE WHEN t.created_actor_type = 'user' THEN 'user' ELSE t.created_agent_id END,
                CASE WHEN t.updated_actor_type = 'user' THEN 'user' ELSE t.updated_agent_id END,
                c.agent_id, a.display_name, c.claimed_at
         FROM tasks t JOIN statuses s ON s.id = t.status_id
         LEFT JOIN task_claims c ON c.task_id = t.id
         LEFT JOIN agents a ON a.id = c.agent_id
         WHERE t.id = ?1",
            [id],
            |row| {
                Ok(FullTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    goal: row.get(3)?,
                    acceptance_criteria: row.get(4)?,
                    task_type: task_type_from_row(row.get::<_, String>(5)?)?,
                    status: TaskStatus {
                        id: row.get(6)?,
                        code: row.get(7)?,
                        name: row.get(8)?,
                        completed: row.get(9)?,
                    },
                    priority: row.get(10)?,
                    estimate: row.get(11)?,
                    tags: vec![],
                    external_urls: vec![],
                    code_references: vec![],
                    hierarchy: empty_hierarchy(),
                    dependencies: empty_dependencies(),
                    claim: row.get::<_, Option<String>>(17)?.map(|id| TaskClaim {
                        agent: ClaimAgent {
                            id,
                            display_name: row.get(18).expect("joined claim agent"),
                        },
                        claimed_at: row.get(19).expect("joined claim timestamp"),
                    }),
                    archived: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                    created_by: row.get(15)?,
                    updated_by: row.get(16)?,
                })
            },
        )
        .optional()
        .map_err(|error| sqlite_access_error("task detail", database_path, error))
}

fn task_type_from_row(value: String) -> rusqlite::Result<TaskType> {
    TaskType::parse(&value).map_err(|_| {
        rusqlite::Error::InvalidColumnType(0, "task_type".to_owned(), rusqlite::types::Type::Text)
    })
}

fn load_full_context(
    transaction: &Transaction<'_>,
    task: &mut FullTask,
    database_path: &Path,
) -> Result<(), Error> {
    task.tags = load_strings(
        transaction,
        "SELECT value FROM task_tags WHERE task_id = ?1 ORDER BY ordinal",
        &task.id,
        database_path,
    )?;
    task.external_urls = load_strings(
        transaction,
        "SELECT url FROM task_external_urls WHERE task_id = ?1 ORDER BY ordinal",
        &task.id,
        database_path,
    )?;
    let mut statement = transaction.prepare("SELECT path, start_line, end_line, description FROM task_code_references WHERE task_id = ?1 ORDER BY ordinal")
        .map_err(|error| sqlite_access_error("task code references", database_path, error))?;
    task.code_references = statement
        .query_map([&task.id], |row| {
            Ok(CodeReference {
                path: row.get(0)?,
                start_line: row.get(1)?,
                end_line: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("task code references", database_path, error))?;
    task.dependencies.upstream = load_related_dependencies(
        transaction,
        "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed
         FROM task_dependencies d
         JOIN tasks t ON t.id = d.upstream_task_id
         JOIN statuses s ON s.id = t.status_id
         WHERE d.downstream_task_id = ?1 ORDER BY t.id",
        &task.id,
        database_path,
    )?;
    task.dependencies.downstream = load_related_dependencies(
        transaction,
        "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed
         FROM task_dependencies d
         JOIN tasks t ON t.id = d.downstream_task_id
         JOIN statuses s ON s.id = t.status_id
         WHERE d.upstream_task_id = ?1 ORDER BY t.id",
        &task.id,
        database_path,
    )?;
    Ok(())
}

fn load_related_dependencies(
    transaction: &Transaction<'_>,
    sql: &str,
    id: &str,
    database_path: &Path,
) -> Result<Vec<RelatedTask>, Error> {
    let mut statement = transaction
        .prepare(sql)
        .map_err(|error| sqlite_access_error("task dependencies", database_path, error))?;
    statement
        .query_map([id], |row| {
            Ok(RelatedTask {
                id: row.get(0)?,
                title: row.get(1)?,
                task_type: task_type_from_row(row.get::<_, String>(2)?)?,
                status: RelatedTaskStatus {
                    code: row.get(3)?,
                    name: row.get(4)?,
                    completed: row.get(5)?,
                },
            })
        })
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("task dependencies", database_path, error))
}

fn load_strings(
    transaction: &Transaction<'_>,
    sql: &str,
    id: &str,
    database_path: &Path,
) -> Result<Vec<String>, Error> {
    let mut statement = transaction
        .prepare(sql)
        .map_err(|error| sqlite_access_error("task context", database_path, error))?;
    statement
        .query_map([id], |row| row.get(0))
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("task context", database_path, error))
}

pub fn list_tasks(current: &Path, input: &ListTasksInput) -> Result<Vec<TaskListItem>, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("task list snapshot", &database_path, error))?;
    for code in input.status_codes.iter().collect::<HashSet<_>>() {
        if status::find_by_code(&transaction, code)
            .map_err(|error| sqlite_access_error("task status filter", &database_path, error))?
            .is_none()
        {
            return Err(Error::StatusNotFound { code: code.clone() });
        }
    }
    let (sql, values) = list_sql(input);
    let mut statement = transaction
        .prepare(&sql)
        .map_err(|error| sqlite_access_error("task list", &database_path, error))?;
    let rows = statement
        .query_map(params_from_iter(values), |row| {
            Ok(TaskListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                task_type: task_type_from_row(row.get::<_, String>(2)?)?,
                status: TaskStatus {
                    id: row.get(3)?,
                    code: row.get(4)?,
                    name: row.get(5)?,
                    completed: row.get(6)?,
                },
                priority: row.get(7)?,
                estimate: row.get(8)?,
                tags: vec![],
                archived: row.get(9)?,
                claim: row.get::<_, Option<String>>(12)?.map(|id| TaskClaim {
                    agent: ClaimAgent {
                        id,
                        display_name: row.get(13).expect("joined claim agent"),
                    },
                    claimed_at: row.get(14).expect("joined claim timestamp"),
                }),
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|error| sqlite_access_error("task list", &database_path, error))?;
    let mut tasks: Vec<_> = rows
        .collect::<rusqlite::Result<_>>()
        .map_err(|error| sqlite_access_error("task list", &database_path, error))?;
    drop(statement);
    load_list_tags(&transaction, &mut tasks, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task list snapshot", &database_path, error))?;
    Ok(tasks)
}

fn list_sql(input: &ListTasksInput) -> (String, Vec<Value>) {
    let mut sql = String::from(
        "SELECT t.id, t.title, t.task_type, s.id, s.code, s.name, s.completed, t.priority, t.estimate_hours, t.archived, t.created_at, t.updated_at, c.agent_id, a.display_name, c.claimed_at FROM tasks t JOIN statuses s ON s.id = t.status_id LEFT JOIN task_claims c ON c.task_id = t.id LEFT JOIN agents a ON a.id = c.agent_id WHERE 1=1",
    );
    let mut values = Vec::new();
    match input.archive_scope {
        ArchiveScope::Active => sql.push_str(" AND t.archived = 0"),
        ArchiveScope::Archived => sql.push_str(" AND t.archived = 1"),
        ArchiveScope::All => {}
    }
    push_in_filter(
        &mut sql,
        &mut values,
        "s.code",
        input.status_codes.iter().cloned(),
    );
    push_in_filter(
        &mut sql,
        &mut values,
        "t.task_type",
        input
            .task_types
            .iter()
            .map(|value| value.as_str().to_owned()),
    );
    if !input.tags.is_empty() {
        sql.push_str(
            " AND EXISTS (SELECT 1 FROM task_tags tf WHERE tf.task_id = t.id AND tf.value IN (",
        );
        push_placeholders(&mut sql, input.tags.len());
        sql.push_str("))");
        values.extend(input.tags.iter().cloned().map(Value::Text));
    }
    sql.push_str(" ORDER BY t.priority DESC, t.created_at ASC, t.id ASC");
    (sql, values)
}

fn push_in_filter(
    sql: &mut String,
    values: &mut Vec<Value>,
    column: &str,
    items: impl Iterator<Item = String>,
) {
    let items: Vec<_> = items.collect();
    if items.is_empty() {
        return;
    }
    sql.push_str(" AND ");
    sql.push_str(column);
    sql.push_str(" IN (");
    push_placeholders(sql, items.len());
    sql.push(')');
    values.extend(items.into_iter().map(Value::Text));
}

fn push_placeholders(sql: &mut String, count: usize) {
    sql.push_str(
        &std::iter::repeat_n("?", count)
            .collect::<Vec<_>>()
            .join(","),
    );
}

fn load_list_tags(
    transaction: &Transaction<'_>,
    tasks: &mut [TaskListItem],
    database_path: &Path,
) -> Result<(), Error> {
    if tasks.is_empty() {
        return Ok(());
    }
    let mut sql = String::from("SELECT task_id, value FROM task_tags WHERE task_id IN (");
    push_placeholders(&mut sql, tasks.len());
    sql.push_str(") ORDER BY task_id, ordinal");
    let ids: Vec<_> = tasks.iter().map(|task| task.id.clone()).collect();
    let mut statement = transaction
        .prepare(&sql)
        .map_err(|error| sqlite_access_error("task list tags", database_path, error))?;
    let tags = statement
        .query_map(params_from_iter(ids), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .and_then(Iterator::collect::<rusqlite::Result<Vec<_>>>)
        .map_err(|error| sqlite_access_error("task list tags", database_path, error))?;
    let indexes: std::collections::HashMap<_, _> = tasks
        .iter()
        .enumerate()
        .map(|(index, task)| (task.id.clone(), index))
        .collect();
    for (id, tag) in tags {
        tasks[*indexes.get(&id).expect("selected task")]
            .tags
            .push(tag);
    }
    Ok(())
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
