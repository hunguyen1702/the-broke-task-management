use crate::{
    AccessIntent, Error, apply_pending_migrations, require_latest_migration, resolve_repository,
    sqlite_access_error, status, utc_now,
};
use rusqlite::{
    OptionalExtension, Transaction, TransactionBehavior, params, params_from_iter, types::Value,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    path::Path,
};
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
    pub archive_reason: Option<String>,
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
    pub title: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub task_type: Option<TaskType>,
    pub status_code: Option<String>,
    pub priority: Option<i64>,
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
pub struct ParentSetInput {
    pub task_id: String,
    pub parent_id: String,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ParentRemoveInput {
    pub task_id: String,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParentMutationResult {
    pub task_id: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyDescendant {
    #[serde(flatten)]
    pub task: RelatedTask,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyResult {
    pub task: RelatedTask,
    pub parent: Option<RelatedTask>,
    pub children: Vec<RelatedTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descendants: Option<Vec<HierarchyDescendant>>,
}

#[derive(Debug, Clone)]
pub struct ClaimTaskInput {
    pub task_id: String,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ClaimNextTaskInput {
    pub agent_id: Uuid,
    pub status_codes: Vec<String>,
    pub task_types: Vec<TaskType>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnclaimTaskInput {
    pub task_id: String,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UnclaimTaskResult {
    pub task: FullTask,
    pub released_agent_display_name: String,
}

#[derive(Debug, Clone)]
pub struct ForceUnclaimTaskInput {
    pub task_id: String,
    pub observed_claim: Option<ObservedClaim>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceUnclaimAvailability {
    pub available: bool,
    pub reason: Option<AvailabilityReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved_upstream_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceUnclaimTaskResult {
    pub task: FullTask,
    pub released_claim: TaskClaim,
    pub availability: ForceUnclaimAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedClaim {
    pub agent_id: String,
    pub claimed_at: String,
}

#[derive(Debug, Clone)]
pub struct ArchiveTaskInput {
    pub task_id: String,
    pub reason: String,
    pub agent_id: Option<Uuid>,
    pub force: bool,
    pub observed_claim: Option<ObservedClaim>,
}

#[derive(Debug, Clone)]
pub struct UnarchiveTaskInput {
    pub task_id: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnarchiveImpactItem {
    pub task_id: String,
    pub title: String,
    pub claim: Option<TaskClaim>,
    pub other_unresolved_upstream_task_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnarchiveImpact {
    pub claimed: Vec<UnarchiveImpactItem>,
    pub otherwise_available: Vec<UnarchiveImpactItem>,
    pub already_blocked_elsewhere: Vec<UnarchiveImpactItem>,
}

impl UnarchiveImpact {
    pub fn is_empty(&self) -> bool {
        self.claimed.is_empty()
            && self.otherwise_available.is_empty()
            && self.already_blocked_elsewhere.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnarchiveTaskResult {
    pub task: FullTask,
    pub impact: UnarchiveImpact,
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

#[derive(Debug, Clone, Default)]
pub struct AvailableTasksInput {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityReason {
    Archived,
    Completed,
    Claimed,
    DependenciesBlocked,
}

impl AvailabilityReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Archived => "archived",
            Self::Completed => "completed",
            Self::Claimed => "claimed",
            Self::DependenciesBlocked => "dependencies_blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyExplanationItem {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: RelatedTaskStatus,
    pub blocked_by_task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedDependencies {
    pub direct: Vec<DependencyExplanationItem>,
    pub recursive: Vec<DependencyExplanationItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBlockingExplanation {
    pub task_id: String,
    pub available: bool,
    pub reasons: Vec<AvailabilityReason>,
    pub claim: Option<TaskClaim>,
    pub unresolved_dependencies: UnresolvedDependencies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipDirection {
    Parent,
    Child,
    Upstream,
    Downstream,
    All,
}

impl RelationshipDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parent => "parent",
            Self::Child => "child",
            Self::Upstream => "upstream",
            Self::Downstream => "downstream",
            Self::All => "all",
        }
    }

    fn selected(self) -> &'static [Self] {
        match self {
            Self::Parent => &[Self::Parent],
            Self::Child => &[Self::Child],
            Self::Upstream => &[Self::Upstream],
            Self::Downstream => &[Self::Downstream],
            Self::All => &[Self::Parent, Self::Child, Self::Upstream, Self::Downstream],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Hierarchy,
    Dependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipReach {
    pub direction: RelationshipDirection,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipNode {
    #[serde(flatten)]
    pub task: RelatedTask,
    pub archived: bool,
    pub claim: Option<TaskClaim>,
    pub blocked: bool,
    pub ready: bool,
    pub reached_by: Vec<RelationshipReach>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipEdge {
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    pub direction: RelationshipDirection,
    pub from_task_id: String,
    pub to_task_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipMap {
    pub root_task_id: String,
    pub direction: RelationshipDirection,
    pub nodes: Vec<RelationshipNode>,
    pub edges: Vec<RelationshipEdge>,
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
        archive_reason: None,
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

pub fn set_parent(current: &Path, input: ParentSetInput) -> Result<ParentMutationResult, Error> {
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
        .map_err(|error| sqlite_access_error("parent mutation", &database_path, error))?;
    validate_actor(&transaction, input.agent_id, &database_path)?;
    let (child_type, child_archived) =
        load_type_and_archive(&transaction, &input.task_id, &database_path)?.ok_or_else(|| {
            Error::TaskNotFound {
                id: input.task_id.clone(),
            }
        })?;
    if child_archived {
        return Err(Error::TaskArchived { id: input.task_id });
    }
    let (parent_type, parent_archived) =
        load_type_and_archive(&transaction, &input.parent_id, &database_path)?.ok_or_else(
            || Error::TaskNotFound {
                id: input.parent_id.clone(),
            },
        )?;
    if parent_archived {
        return Err(Error::ParentArchived {
            task_id: input.task_id,
            parent_id: input.parent_id,
        });
    }
    if input.task_id == input.parent_id {
        return Err(Error::SelfParent {
            task_id: input.task_id,
        });
    }
    if !hierarchy_type_allowed(child_type, Some(parent_type)) {
        return Err(Error::InvalidTaskHierarchy {
            task_id: input.task_id,
            parent_id: input.parent_id,
        });
    }
    if hierarchy_would_cycle(
        &transaction,
        &input.task_id,
        &input.parent_id,
        &database_path,
    )? {
        return Err(Error::HierarchyCycle {
            task_id: input.task_id,
            parent_id: input.parent_id,
        });
    }
    let current_parent = select_parent_id(&transaction, &input.task_id)
        .map_err(|error| sqlite_access_error("parent lookup", &database_path, error))?;
    if current_parent.as_deref() != Some(&input.parent_id) {
        transaction.execute(
            "INSERT INTO task_hierarchy (child_task_id, parent_task_id) VALUES (?1, ?2) ON CONFLICT(child_task_id) DO UPDATE SET parent_task_id = excluded.parent_task_id",
            params![input.task_id, input.parent_id],
        ).map_err(|error| sqlite_access_error("parent mutation", &database_path, error))?;
        update_child_actor(&transaction, &input.task_id, input.agent_id, &database_path)?;
    }
    let result = ParentMutationResult {
        task_id: input.task_id,
        parent_id: Some(input.parent_id),
    };
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("parent mutation", &database_path, error))?;
    Ok(result)
}

pub fn remove_parent(
    current: &Path,
    input: ParentRemoveInput,
) -> Result<ParentMutationResult, Error> {
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
        .map_err(|error| sqlite_access_error("parent mutation", &database_path, error))?;
    validate_actor(&transaction, input.agent_id, &database_path)?;
    let (_, archived) = load_type_and_archive(&transaction, &input.task_id, &database_path)?
        .ok_or_else(|| Error::TaskNotFound {
            id: input.task_id.clone(),
        })?;
    if archived {
        return Err(Error::TaskArchived { id: input.task_id });
    }
    let parent_id = select_parent_id(&transaction, &input.task_id)
        .map_err(|error| sqlite_access_error("parent lookup", &database_path, error))?
        .ok_or_else(|| Error::ParentNotFound {
            task_id: input.task_id.clone(),
        })?;
    let (_, parent_archived) = load_type_and_archive(&transaction, &parent_id, &database_path)?
        .expect("hierarchy foreign key references a task");
    if parent_archived {
        return Err(Error::ParentArchived {
            task_id: input.task_id,
            parent_id,
        });
    }
    transaction
        .execute(
            "DELETE FROM task_hierarchy WHERE child_task_id = ?1",
            [&input.task_id],
        )
        .map_err(|error| sqlite_access_error("parent removal", &database_path, error))?;
    update_child_actor(&transaction, &input.task_id, input.agent_id, &database_path)?;
    let result = ParentMutationResult {
        task_id: input.task_id,
        parent_id: None,
    };
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("parent mutation", &database_path, error))?;
    Ok(result)
}

pub fn task_hierarchy(
    current: &Path,
    task_id: &str,
    recursive: bool,
) -> Result<HierarchyResult, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("hierarchy snapshot", &database_path, error))?;
    let task = select_related_task(&transaction, task_id, &database_path)?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_owned(),
        }
    })?;
    let parent = select_direct_parent(&transaction, task_id, &database_path)?;
    let children = select_direct_children(&transaction, task_id, &database_path)?;
    let descendants = recursive
        .then(|| select_descendants(&transaction, task_id, &database_path))
        .transpose()?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("hierarchy snapshot", &database_path, error))?;
    Ok(HierarchyResult {
        task,
        parent,
        children,
        descendants,
    })
}

pub fn hierarchy_type_allowed(child: TaskType, parent: Option<TaskType>) -> bool {
    match (child, parent) {
        (_, None) => true,
        (TaskType::Epic, Some(_)) => false,
        (TaskType::Story, Some(TaskType::Epic)) => true,
        (TaskType::Story, Some(_)) => false,
        (_, Some(TaskType::Epic | TaskType::Story)) => true,
        _ => false,
    }
}

pub fn validate_hierarchy_edges(
    connection: &rusqlite::Connection,
    task_id: &str,
    task_type: TaskType,
    database_path: &Path,
) -> Result<(), Error> {
    if let Some(parent) = select_direct_parent(connection, task_id, database_path)?
        && !hierarchy_type_allowed(task_type, Some(parent.task_type))
    {
        return Err(Error::InvalidTaskHierarchy {
            task_id: task_id.to_owned(),
            parent_id: parent.id,
        });
    }
    for child in select_direct_children(connection, task_id, database_path)? {
        if !hierarchy_type_allowed(child.task_type, Some(task_type)) {
            return Err(Error::InvalidTaskHierarchy {
                task_id: child.id,
                parent_id: task_id.to_owned(),
            });
        }
    }
    Ok(())
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
    let agent_id = validate_claim_agent(&transaction, input.agent_id, &database_path)?;
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
    insert_task_claim(&transaction, &input.task_id, &agent_id, &database_path)?;
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

/// Selects and claims the first currently available task in one immediate transaction.
pub fn claim_next_task(
    current: &Path,
    input: ClaimNextTaskInput,
) -> Result<Option<FullTask>, Error> {
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
        .map_err(|error| sqlite_access_error("task claim-next", &database_path, error))?;
    let agent_id = validate_claim_agent(&transaction, input.agent_id, &database_path)?;
    let filters = AvailableTasksInput {
        status_codes: input.status_codes,
        task_types: input.task_types,
        tags: input.tags,
    };
    let Some(candidate) = select_first_available_task(&transaction, &filters, &database_path)?
    else {
        transaction
            .commit()
            .map_err(|error| sqlite_access_error("task claim-next", &database_path, error))?;
        return Ok(None);
    };
    insert_task_claim(&transaction, &candidate.id, &agent_id, &database_path)?;
    let mut task =
        load_full_task(&transaction, &candidate.id, &database_path)?.ok_or_else(|| {
            Error::TaskNotFound {
                id: candidate.id.clone(),
            }
        })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task claim-next", &database_path, error))?;
    Ok(Some(task))
}

fn validate_claim_agent(
    connection: &rusqlite::Connection,
    agent_id: Uuid,
    database_path: &Path,
) -> Result<String, Error> {
    let agent_id_string = agent_id.to_string();
    let exists = connection
        .query_row(
            "SELECT 1 FROM agents WHERE id = ?1",
            [&agent_id_string],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("claim agent lookup", database_path, error))?
        .is_some();
    if !exists {
        return Err(Error::AgentNotFound { id: agent_id });
    }
    Ok(agent_id_string)
}

fn insert_task_claim(
    connection: &rusqlite::Connection,
    task_id: &str,
    agent_id: &str,
    database_path: &Path,
) -> Result<(), Error> {
    connection
        .execute(
            "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, ?2, ?3)",
            params![task_id, agent_id, utc_now()],
        )
        .map_err(|error| sqlite_access_error("task claim", database_path, error))?;
    Ok(())
}

pub fn unclaim_task(current: &Path, input: UnclaimTaskInput) -> Result<UnclaimTaskResult, Error> {
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
        .map_err(|error| sqlite_access_error("task unclaim", &database_path, error))?;
    let agent_name = release_owned_claim(&transaction, &input, &database_path)?;
    let mut task =
        load_full_task(&transaction, &input.task_id, &database_path)?.ok_or_else(|| {
            Error::TaskNotFound {
                id: input.task_id.clone(),
            }
        })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    debug_assert!(task.claim.is_none());
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task unclaim", &database_path, error))?;
    Ok(UnclaimTaskResult {
        task,
        released_agent_display_name: agent_name,
    })
}

pub fn force_unclaim_task(
    current: &Path,
    input: ForceUnclaimTaskInput,
) -> Result<ForceUnclaimTaskResult, Error> {
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
        .map_err(|error| sqlite_access_error("task force unclaim", &database_path, error))?;
    let task_exists = transaction
        .query_row(
            "SELECT 1 FROM tasks WHERE id = ?1",
            [&input.task_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("force unclaim task lookup", &database_path, error))?
        .is_some();
    if !task_exists {
        return Err(Error::TaskNotFound { id: input.task_id });
    }
    let (agent_id, display_name, claimed_at) = transaction
        .query_row(
            "SELECT a.id, a.display_name, c.claimed_at FROM task_claims c JOIN agents a ON a.id = c.agent_id WHERE c.task_id = ?1",
            [&input.task_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("force unclaim claim lookup", &database_path, error))?
        .ok_or_else(|| Error::ClaimNotFound { task_id: input.task_id.clone() })?;
    if input
        .observed_claim
        .as_ref()
        .is_some_and(|observed| observed.agent_id != agent_id || observed.claimed_at != claimed_at)
    {
        return Err(Error::ClaimChanged {
            task_id: input.task_id,
            agent_id,
            agent_display_name: display_name,
            claimed_at,
        });
    }
    let released_claim = TaskClaim {
        agent: ClaimAgent {
            id: agent_id.clone(),
            display_name,
        },
        claimed_at: claimed_at.clone(),
    };
    let affected = transaction
        .execute(
            "DELETE FROM task_claims WHERE task_id = ?1 AND agent_id = ?2 AND claimed_at = ?3",
            params![input.task_id, agent_id, claimed_at],
        )
        .map_err(|error| sqlite_access_error("task force unclaim", &database_path, error))?;
    if affected != 1 {
        return Err(sqlite_access_error(
            "task force unclaim integrity",
            &database_path,
            rusqlite::Error::ExecuteReturnedResults,
        ));
    }
    let mut task = load_full_task(&transaction, &input.task_id, &database_path)?
        .expect("force-unclaimed task exists");
    load_full_context(&transaction, &mut task, &database_path)?;
    let explanation =
        select_task_blocking_explanation(&transaction, &input.task_id, &database_path)?;
    let reason = explanation.reasons.first().copied();
    let unresolved_upstream_ids =
        (reason == Some(AvailabilityReason::DependenciesBlocked)).then(|| {
            explanation
                .unresolved_dependencies
                .direct
                .iter()
                .map(|item| item.id.clone())
                .collect()
        });
    let availability = ForceUnclaimAvailability {
        available: explanation.available,
        reason,
        unresolved_upstream_ids,
    };
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task force unclaim", &database_path, error))?;
    Ok(ForceUnclaimTaskResult {
        task,
        released_claim,
        availability,
    })
}

pub fn archive_task(current: &Path, mut input: ArchiveTaskInput) -> Result<FullTask, Error> {
    input.reason = input.reason.trim().to_owned();
    if input.reason.is_empty() {
        return Err(Error::InvalidArchiveReason);
    }
    if input.force && input.agent_id.is_some() {
        return Err(Error::ArchivePermissionDenied);
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
        .map_err(|error| sqlite_access_error("task archive", &database_path, error))?;
    if let Some(agent_id) = input.agent_id {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM agents WHERE id = ?1",
                [agent_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| sqlite_access_error("archive actor lookup", &database_path, error))?
            .is_some();
        if !exists {
            return Err(Error::AgentNotFound { id: agent_id });
        }
    }
    let (archived, existing_reason) = transaction
        .query_row(
            "SELECT archived, archive_reason FROM tasks WHERE id = ?1",
            [&input.task_id],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("archive task lookup", &database_path, error))?
        .ok_or_else(|| Error::TaskNotFound {
            id: input.task_id.clone(),
        })?;
    if archived {
        if existing_reason.as_deref() != Some(&input.reason) {
            return Err(Error::TaskArchived { id: input.task_id });
        }
        let mut task =
            load_full_task(&transaction, &input.task_id, &database_path)?.expect("task exists");
        load_full_context(&transaction, &mut task, &database_path)?;
        transaction
            .commit()
            .map_err(|error| sqlite_access_error("task archive", &database_path, error))?;
        return Ok(task);
    }
    let claim = transaction
        .query_row(
            "SELECT a.id, a.display_name, c.claimed_at FROM task_claims c JOIN agents a ON a.id = c.agent_id WHERE c.task_id = ?1",
            [&input.task_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("archive claim lookup", &database_path, error))?;
    if let Some((owner_id, owner_name, claimed_at)) = &claim {
        let own_claim = input.agent_id.is_some_and(|id| id.to_string() == *owner_id);
        let approved = input.force
            && input.agent_id.is_none()
            && input.observed_claim.as_ref().is_some_and(|observed| {
                observed.agent_id == *owner_id && observed.claimed_at == *claimed_at
            });
        if !own_claim && !approved {
            return Err(Error::TaskClaimed {
                task_id: input.task_id,
                agent_id: owner_id.clone(),
                agent_display_name: owner_name.clone(),
                claimed_at: claimed_at.clone(),
            });
        }
    }
    if claim.is_some() {
        transaction
            .execute(
                "DELETE FROM task_claims WHERE task_id = ?1",
                [&input.task_id],
            )
            .map_err(|error| sqlite_access_error("archive claim release", &database_path, error))?;
    }
    let actor_type = if input.agent_id.is_some() {
        "agent"
    } else {
        "user"
    };
    let actor_id = input.agent_id.map(|id| id.to_string());
    transaction.execute(
        "UPDATE tasks SET archived = 1, archive_reason = ?1, updated_actor_type = ?2, updated_agent_id = ?3, updated_at = ?4 WHERE id = ?5",
        params![input.reason, actor_type, actor_id, utc_now(), input.task_id],
    ).map_err(|error| sqlite_access_error("task archive", &database_path, error))?;
    let mut task =
        load_full_task(&transaction, &input.task_id, &database_path)?.expect("task exists");
    load_full_context(&transaction, &mut task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task archive", &database_path, error))?;
    Ok(task)
}

pub fn preview_unarchive(current: &Path, task_id: &str) -> Result<UnarchiveTaskResult, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    let database_path = resolved.database_path.clone();
    apply_pending_migrations(&mut resolved)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("task unarchive preview", &database_path, error))?;
    let mut task = load_full_task(&transaction, task_id, &database_path)?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_owned(),
        }
    })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    let impact = select_unarchive_impact(&transaction, &task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task unarchive preview", &database_path, error))?;
    Ok(UnarchiveTaskResult { task, impact })
}

pub fn unarchive_task(
    current: &Path,
    input: UnarchiveTaskInput,
) -> Result<UnarchiveTaskResult, Error> {
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
        .map_err(|error| sqlite_access_error("task unarchive", &database_path, error))?;
    let mut task =
        load_full_task(&transaction, &input.task_id, &database_path)?.ok_or_else(|| {
            Error::TaskNotFound {
                id: input.task_id.clone(),
            }
        })?;
    load_full_context(&transaction, &mut task, &database_path)?;
    if !task.archived {
        transaction
            .commit()
            .map_err(|error| sqlite_access_error("task unarchive", &database_path, error))?;
        return Ok(UnarchiveTaskResult {
            task,
            impact: UnarchiveImpact::default(),
        });
    }
    let impact = select_unarchive_impact(&transaction, &task, &database_path)?;
    if !impact.is_empty() && !input.confirmed {
        return Err(Error::ConfirmationRequired { impact });
    }
    transaction
        .execute(
            "UPDATE tasks SET archived = 0, archive_reason = NULL, updated_actor_type = 'user', updated_agent_id = NULL, updated_at = ?1 WHERE id = ?2",
            params![utc_now(), input.task_id],
        )
        .map_err(|error| sqlite_access_error("task unarchive", &database_path, error))?;
    let mut task = load_full_task(&transaction, &input.task_id, &database_path)?
        .expect("unarchived task exists");
    load_full_context(&transaction, &mut task, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("task unarchive", &database_path, error))?;
    Ok(UnarchiveTaskResult { task, impact })
}

fn select_unarchive_impact(
    connection: &rusqlite::Connection,
    task: &FullTask,
    database_path: &Path,
) -> Result<UnarchiveImpact, Error> {
    if !task.archived || task.status.completed {
        return Ok(UnarchiveImpact::default());
    }
    let mut statement = connection
        .prepare(
            "SELECT t.id, t.title FROM task_dependencies d JOIN tasks t ON t.id = d.downstream_task_id JOIN statuses s ON s.id = t.status_id WHERE d.upstream_task_id = ?1 AND NOT t.archived AND NOT s.completed ORDER BY t.id",
        )
        .map_err(|error| sqlite_access_error("unarchive direct impact", database_path, error))?;
    let downstream: Vec<(String, String)> = statement
        .query_map([&task.id], |row| Ok((row.get(0)?, row.get(1)?)))
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("unarchive direct impact", database_path, error))?;
    drop(statement);
    let mut impact = UnarchiveImpact::default();
    for (task_id, title) in downstream {
        let explanation = select_task_blocking_explanation(connection, &task_id, database_path)?;
        let item = UnarchiveImpactItem {
            task_id,
            title,
            claim: explanation.claim.clone(),
            other_unresolved_upstream_task_ids: explanation
                .unresolved_dependencies
                .direct
                .iter()
                .map(|dependency| dependency.id.clone())
                .filter(|id| id != &task.id)
                .collect(),
        };
        if item.claim.is_some() {
            impact.claimed.push(item);
        } else if explanation.available {
            impact.otherwise_available.push(item);
        } else {
            impact.already_blocked_elsewhere.push(item);
        }
    }
    Ok(impact)
}

pub(crate) fn release_owned_claim(
    transaction: &Transaction<'_>,
    input: &UnclaimTaskInput,
    database_path: &Path,
) -> Result<String, Error> {
    let agent_id = input.agent_id.to_string();
    let agent_name = transaction
        .query_row(
            "SELECT display_name FROM agents WHERE id = ?1",
            [&agent_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| sqlite_access_error("unclaim agent lookup", database_path, error))?
        .ok_or(Error::AgentNotFound { id: input.agent_id })?;
    let task_exists = transaction
        .query_row(
            "SELECT 1 FROM tasks WHERE id = ?1",
            [&input.task_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| sqlite_access_error("unclaim task lookup", database_path, error))?
        .is_some();
    if !task_exists {
        return Err(Error::TaskNotFound {
            id: input.task_id.clone(),
        });
    }
    let (owner_id, owner_name, claimed_at) = transaction
        .query_row(
            "SELECT a.id, a.display_name, c.claimed_at FROM task_claims c JOIN agents a ON a.id = c.agent_id WHERE c.task_id = ?1",
            [&input.task_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("unclaim owner lookup", database_path, error))?
        .ok_or_else(|| Error::ClaimNotFound {
            task_id: input.task_id.clone(),
        })?;
    if owner_id != agent_id {
        return Err(Error::ClaimNotOwned {
            task_id: input.task_id.clone(),
            agent_id: owner_id,
            agent_display_name: owner_name,
            claimed_at,
        });
    }
    let affected = transaction
        .execute(
            "DELETE FROM task_claims WHERE task_id = ?1 AND agent_id = ?2",
            params![input.task_id, agent_id],
        )
        .map_err(|error| sqlite_access_error("task unclaim", database_path, error))?;
    if affected != 1 {
        return Err(sqlite_access_error(
            "task unclaim integrity",
            database_path,
            rusqlite::Error::ExecuteReturnedResults,
        ));
    }
    Ok(agent_name)
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
        && input.title.is_none()
        && input.description.is_none()
        && input.goal.is_none()
        && input.acceptance_criteria.is_none()
        && input.task_type.is_none()
        && input.status_code.is_none()
        && input.priority.is_none()
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
    let status = input
        .status_code
        .as_ref()
        .map(|code| {
            status::find_by_code(&transaction, code)
                .map_err(|error| sqlite_access_error("task status lookup", &database_path, error))?
                .ok_or_else(|| Error::StatusNotFound { code: code.clone() })
        })
        .transpose()?;
    if task.archived {
        return Err(Error::TaskArchived { id: id.to_owned() });
    }
    load_full_context(&transaction, &mut task, &database_path)?;
    let title = input.title.as_ref().unwrap_or(&task.title);
    let description = input.description.as_ref().unwrap_or(&task.description);
    let goal = input.goal.as_ref().unwrap_or(&task.goal);
    let acceptance_criteria = input
        .acceptance_criteria
        .as_ref()
        .unwrap_or(&task.acceptance_criteria);
    let task_type = input.task_type.unwrap_or(task.task_type);
    let status_id = status
        .as_ref()
        .map_or(task.status.id.as_str(), |value| value.id.as_str());
    let priority = input.priority.unwrap_or(task.priority);
    if input.task_type.is_some() && task_type != task.task_type {
        validate_hierarchy_edges(&transaction, id, task_type, &database_path)?;
    }
    let estimate = patch_target(&input.estimate, task.estimate);
    let tags = collection_target(&input.tags);
    let urls = collection_target(&input.external_urls);
    let references = collection_target(&input.code_references);
    let scalar_changed = title != &task.title
        || description != &task.description
        || goal != &task.goal
        || acceptance_criteria != &task.acceptance_criteria
        || task_type != task.task_type
        || status_id != task.status.id
        || priority != task.priority;
    let changed = scalar_changed
        || estimate != task.estimate
        || tags.as_ref().is_some_and(|value| value != &task.tags)
        || urls
            .as_ref()
            .is_some_and(|value| value != &task.external_urls)
        || references
            .as_ref()
            .is_some_and(|value| value != &task.code_references);
    if changed {
        if scalar_changed {
            transaction
                .execute(
                    "UPDATE tasks SET title = ?1, description = ?2, goal = ?3, acceptance_criteria = ?4, task_type = ?5, status_id = ?6, priority = ?7 WHERE id = ?8",
                    params![title, description, goal, acceptance_criteria, task_type.as_str(), status_id, priority, id],
                )
                .map_err(|error| sqlite_access_error("task scalar update", &database_path, error))?;
        }
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
    if let Some(title) = &mut input.title {
        *title = title.trim().to_owned();
        if title.is_empty() {
            return Err(Error::InvalidTaskTitle);
        }
    }
    if input
        .priority
        .is_some_and(|priority| !(0..=1_000_000).contains(&priority))
    {
        return Err(Error::InvalidTaskUpdatePriority);
    }
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
                t.estimate_hours, t.archived, t.archive_reason, t.created_at, t.updated_at,
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
                    claim: row.get::<_, Option<String>>(18)?.map(|id| TaskClaim {
                        agent: ClaimAgent {
                            id,
                            display_name: row.get(19).expect("joined claim agent"),
                        },
                        claimed_at: row.get(20).expect("joined claim timestamp"),
                    }),
                    archived: row.get(12)?,
                    archive_reason: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                    created_by: row.get(16)?,
                    updated_by: row.get(17)?,
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
    task.hierarchy.parent = select_direct_parent(transaction, &task.id, database_path)?;
    task.hierarchy.children = select_direct_children(transaction, &task.id, database_path)?;
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

fn validate_actor(
    connection: &rusqlite::Connection,
    agent_id: Option<Uuid>,
    database_path: &Path,
) -> Result<(), Error> {
    if let Some(agent_id) = agent_id {
        let exists = connection
            .query_row(
                "SELECT 1 FROM agents WHERE id = ?1",
                [agent_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| sqlite_access_error("parent actor lookup", database_path, error))?
            .is_some();
        if !exists {
            return Err(Error::AgentNotFound { id: agent_id });
        }
    }
    Ok(())
}

fn load_type_and_archive(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Option<(TaskType, bool)>, Error> {
    connection
        .query_row(
            "SELECT task_type, archived FROM tasks WHERE id = ?1",
            [task_id],
            |row| Ok((task_type_from_row(row.get::<_, String>(0)?)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| sqlite_access_error("hierarchy task lookup", database_path, error))
}

fn update_child_actor(
    connection: &rusqlite::Connection,
    task_id: &str,
    agent_id: Option<Uuid>,
    database_path: &Path,
) -> Result<(), Error> {
    let actor_type = if agent_id.is_some() { "agent" } else { "user" };
    connection.execute(
        "UPDATE tasks SET updated_at = ?1, updated_actor_type = ?2, updated_agent_id = ?3 WHERE id = ?4",
        params![utc_now(), actor_type, agent_id.map(|id| id.to_string()), task_id],
    ).map_err(|error| sqlite_access_error("parent metadata update", database_path, error))?;
    Ok(())
}

fn select_parent_id(
    connection: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<Option<String>> {
    connection
        .query_row(
            "SELECT parent_task_id FROM task_hierarchy WHERE child_task_id = ?1",
            [task_id],
            |row| row.get(0),
        )
        .optional()
}

pub fn select_direct_parent(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Option<RelatedTask>, Error> {
    select_related_tasks(connection,
        "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed FROM task_hierarchy h JOIN tasks t ON t.id = h.parent_task_id JOIN statuses s ON s.id = t.status_id WHERE h.child_task_id = ?1",
        task_id, database_path, "task parent").map(|mut tasks| tasks.pop())
}

pub fn select_direct_children(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Vec<RelatedTask>, Error> {
    select_related_tasks(
        connection,
        "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed FROM task_hierarchy h JOIN tasks t ON t.id = h.child_task_id JOIN statuses s ON s.id = t.status_id WHERE h.parent_task_id = ?1 ORDER BY t.id",
        task_id,
        database_path,
        "task children",
    )
}

fn select_related_task(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Option<RelatedTask>, Error> {
    select_related_tasks(connection,
        "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed FROM tasks t JOIN statuses s ON s.id = t.status_id WHERE t.id = ?1",
        task_id, database_path, "hierarchy task").map(|mut tasks| tasks.pop())
}

fn select_related_tasks(
    connection: &rusqlite::Connection,
    sql: &str,
    task_id: &str,
    database_path: &Path,
    phase: &'static str,
) -> Result<Vec<RelatedTask>, Error> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| sqlite_access_error(phase, database_path, error))?;
    statement
        .query_map([task_id], |row| {
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
        .map_err(|error| sqlite_access_error(phase, database_path, error))
}

pub fn select_descendants(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Vec<HierarchyDescendant>, Error> {
    let mut statement = connection.prepare(
        "WITH RECURSIVE descendants(id, depth) AS (
           SELECT child_task_id, 1 FROM task_hierarchy WHERE parent_task_id = ?1
           UNION
           SELECT h.child_task_id, d.depth + 1 FROM task_hierarchy h JOIN descendants d ON h.parent_task_id = d.id
         ) SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed, MIN(d.depth)
         FROM descendants d JOIN tasks t ON t.id = d.id JOIN statuses s ON s.id = t.status_id
         GROUP BY t.id, t.title, t.task_type, s.code, s.name, s.completed ORDER BY MIN(d.depth), t.id"
    ).map_err(|error| sqlite_access_error("hierarchy descendants", database_path, error))?;
    statement
        .query_map([task_id], |row| {
            Ok(HierarchyDescendant {
                task: RelatedTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    task_type: task_type_from_row(row.get::<_, String>(2)?)?,
                    status: RelatedTaskStatus {
                        code: row.get(3)?,
                        name: row.get(4)?,
                        completed: row.get(5)?,
                    },
                },
                depth: row.get(6)?,
            })
        })
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("hierarchy descendants", database_path, error))
}

pub fn hierarchy_would_cycle(
    connection: &rusqlite::Connection,
    task_id: &str,
    parent_id: &str,
    database_path: &Path,
) -> Result<bool, Error> {
    connection.query_row(
        "WITH RECURSIVE descendants(id) AS (
           SELECT child_task_id FROM task_hierarchy WHERE parent_task_id = ?1
           UNION SELECT h.child_task_id FROM task_hierarchy h JOIN descendants d ON h.parent_task_id = d.id
         ) SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?2)",
        params![task_id, parent_id], |row| row.get(0)
    ).map_err(|error| sqlite_access_error("hierarchy cycle validation", database_path, error))
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

pub fn task_relationship_map(
    current: &Path,
    task_id: &str,
    direction: RelationshipDirection,
) -> Result<RelationshipMap, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("relationship map snapshot", &database_path, error))?;
    let result = select_relationship_map(&transaction, task_id, direction, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("relationship map snapshot", &database_path, error))?;
    Ok(result)
}

#[derive(Debug, Clone)]
struct MapNodeState {
    task: RelatedTask,
    archived: bool,
    claim: Option<TaskClaim>,
    blocked: bool,
    ready: bool,
}

/// Builds a relationship map inside a caller-owned snapshot.
pub fn select_relationship_map(
    connection: &rusqlite::Connection,
    task_id: &str,
    direction: RelationshipDirection,
    database_path: &Path,
) -> Result<RelationshipMap, Error> {
    let states = load_map_node_states(connection, database_path)?;
    if !states.contains_key(task_id) {
        return Err(Error::TaskNotFound {
            id: task_id.to_owned(),
        });
    }
    let hierarchy = load_map_pairs(
        connection,
        "SELECT parent_task_id, child_task_id FROM task_hierarchy ORDER BY parent_task_id, child_task_id",
        database_path,
        "relationship map hierarchy",
    )?;
    let dependencies = load_map_pairs(
        connection,
        "SELECT upstream_task_id, downstream_task_id FROM task_dependencies ORDER BY upstream_task_id, downstream_task_id",
        database_path,
        "relationship map dependencies",
    )?;

    let mut reached: BTreeMap<String, BTreeMap<RelationshipDirection, u32>> = BTreeMap::new();
    let mut edges = BTreeSet::new();
    for selected in direction.selected() {
        let (relationship_type, pairs, reverse) = match selected {
            RelationshipDirection::Parent => (RelationshipType::Hierarchy, &hierarchy, true),
            RelationshipDirection::Child => (RelationshipType::Hierarchy, &hierarchy, false),
            RelationshipDirection::Upstream => (RelationshipType::Dependency, &dependencies, true),
            RelationshipDirection::Downstream => {
                (RelationshipType::Dependency, &dependencies, false)
            }
            RelationshipDirection::All => unreachable!("all expands into concrete directions"),
        };
        traverse_map_direction(
            task_id,
            *selected,
            relationship_type,
            pairs,
            reverse,
            &mut reached,
            &mut edges,
        );
    }

    let mut nodes: Vec<_> = states
        .into_iter()
        .filter_map(|(id, state)| {
            if id != task_id && !reached.contains_key(&id) {
                return None;
            }
            let reached_by = reached
                .remove(&id)
                .unwrap_or_default()
                .into_iter()
                .map(|(direction, depth)| RelationshipReach { direction, depth })
                .collect();
            Some(RelationshipNode {
                task: state.task,
                archived: state.archived,
                claim: state.claim,
                blocked: state.blocked,
                ready: state.ready,
                reached_by,
            })
        })
        .collect();
    nodes.sort_by(|left, right| {
        if left.task.id == task_id {
            return std::cmp::Ordering::Less;
        }
        if right.task.id == task_id {
            return std::cmp::Ordering::Greater;
        }
        let left_depth = left
            .reached_by
            .iter()
            .map(|item| item.depth)
            .min()
            .unwrap_or(0);
        let right_depth = right
            .reached_by
            .iter()
            .map(|item| item.depth)
            .min()
            .unwrap_or(0);
        left_depth
            .cmp(&right_depth)
            .then_with(|| left.task.id.cmp(&right.task.id))
    });
    Ok(RelationshipMap {
        root_task_id: task_id.to_owned(),
        direction,
        nodes,
        edges: edges.into_iter().collect(),
    })
}

fn traverse_map_direction(
    root: &str,
    direction: RelationshipDirection,
    relationship_type: RelationshipType,
    pairs: &[(String, String)],
    reverse: bool,
    reached: &mut BTreeMap<String, BTreeMap<RelationshipDirection, u32>>,
    edges: &mut BTreeSet<RelationshipEdge>,
) {
    let mut adjacency: BTreeMap<&str, Vec<&(String, String)>> = BTreeMap::new();
    for pair in pairs {
        let predecessor = if reverse {
            pair.1.as_str()
        } else {
            pair.0.as_str()
        };
        adjacency.entry(predecessor).or_default().push(pair);
    }
    let mut distances = BTreeMap::from([(root.to_owned(), 0_u32)]);
    let mut queue = VecDeque::from([root.to_owned()]);
    while let Some(predecessor) = queue.pop_front() {
        let depth = distances[&predecessor];
        for pair in adjacency.get(predecessor.as_str()).into_iter().flatten() {
            let next = if reverse { &pair.0 } else { &pair.1 };
            edges.insert(RelationshipEdge {
                relationship_type,
                direction,
                from_task_id: pair.0.clone(),
                to_task_id: pair.1.clone(),
            });
            if next == root {
                continue;
            }
            let next_depth = depth + 1;
            if !distances.contains_key(next) {
                distances.insert(next.clone(), next_depth);
                queue.push_back(next.clone());
            }
            reached
                .entry(next.clone())
                .or_default()
                .entry(direction)
                .and_modify(|current| *current = (*current).min(next_depth))
                .or_insert(next_depth);
        }
    }
}

fn load_map_pairs(
    connection: &rusqlite::Connection,
    sql: &str,
    database_path: &Path,
    phase: &'static str,
) -> Result<Vec<(String, String)>, Error> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| sqlite_access_error(phase, database_path, error))?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error(phase, database_path, error))
}

fn load_map_node_states(
    connection: &rusqlite::Connection,
    database_path: &Path,
) -> Result<BTreeMap<String, MapNodeState>, Error> {
    let mut statement = connection
        .prepare(
            "SELECT t.id, t.title, t.task_type, s.code, s.name, s.completed, t.archived,
         c.agent_id, a.display_name, c.claimed_at,
         EXISTS (SELECT 1 FROM task_dependencies d JOIN tasks u ON u.id = d.upstream_task_id
           JOIN statuses us ON us.id = u.status_id WHERE d.downstream_task_id = t.id
           AND NOT u.archived AND NOT us.completed),
         NOT t.archived AND NOT s.completed AND c.task_id IS NULL AND NOT EXISTS (
           SELECT 1 FROM task_dependencies d JOIN tasks u ON u.id = d.upstream_task_id
           JOIN statuses us ON us.id = u.status_id WHERE d.downstream_task_id = t.id
           AND NOT u.archived AND NOT us.completed)
         FROM tasks t JOIN statuses s ON s.id = t.status_id
         LEFT JOIN task_claims c ON c.task_id = t.id LEFT JOIN agents a ON a.id = c.agent_id
         ORDER BY t.id",
        )
        .map_err(|error| sqlite_access_error("relationship map nodes", database_path, error))?;
    let rows = statement
        .query_map([], |row| {
            let id: String = row.get(0)?;
            Ok((
                id.clone(),
                MapNodeState {
                    task: RelatedTask {
                        id,
                        title: row.get(1)?,
                        task_type: task_type_from_row(row.get::<_, String>(2)?)?,
                        status: RelatedTaskStatus {
                            code: row.get(3)?,
                            name: row.get(4)?,
                            completed: row.get(5)?,
                        },
                    },
                    archived: row.get(6)?,
                    claim: row.get::<_, Option<String>>(7)?.map(|id| TaskClaim {
                        agent: ClaimAgent {
                            id,
                            display_name: row.get(8).expect("joined claim agent"),
                        },
                        claimed_at: row.get(9).expect("joined claim timestamp"),
                    }),
                    blocked: row.get(10)?,
                    ready: row.get(11)?,
                },
            ))
        })
        .map_err(|error| sqlite_access_error("relationship map nodes", database_path, error))?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(|error| sqlite_access_error("relationship map nodes", database_path, error))
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

pub fn available_tasks(
    current: &Path,
    input: &AvailableTasksInput,
) -> Result<Vec<TaskListItem>, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("available task snapshot", &database_path, error))?;
    let tasks = select_available_tasks(&transaction, input, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("available task snapshot", &database_path, error))?;
    Ok(tasks)
}

pub fn explain_task_blocking(
    current: &Path,
    task_id: &str,
) -> Result<TaskBlockingExplanation, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved.connection_mut().transaction().map_err(|error| {
        sqlite_access_error("blocking explanation snapshot", &database_path, error)
    })?;
    let explanation = select_task_blocking_explanation(&transaction, task_id, &database_path)?;
    transaction.commit().map_err(|error| {
        sqlite_access_error("blocking explanation snapshot", &database_path, error)
    })?;
    Ok(explanation)
}

/// Explains availability using a caller-owned connection or transaction.
pub fn select_task_blocking_explanation(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<TaskBlockingExplanation, Error> {
    let root = connection
        .query_row(
            "SELECT t.archived, s.completed, c.agent_id, a.display_name, c.claimed_at,
         NOT t.archived AND NOT s.completed AND c.task_id IS NULL AND NOT EXISTS (
           SELECT 1 FROM task_dependencies d JOIN tasks u ON u.id = d.upstream_task_id
           JOIN statuses us ON us.id = u.status_id WHERE d.downstream_task_id = t.id
           AND NOT u.archived AND NOT us.completed)
         FROM tasks t JOIN statuses s ON s.id = t.status_id
         LEFT JOIN task_claims c ON c.task_id = t.id LEFT JOIN agents a ON a.id = c.agent_id
         WHERE t.id = ?1",
            [task_id],
            |row| {
                Ok((
                    row.get::<_, bool>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, bool>(5)?,
                ))
            },
        )
        .optional()
        .map_err(|error| sqlite_access_error("blocking explanation root", database_path, error))?
        .ok_or_else(|| Error::TaskNotFound {
            id: task_id.to_owned(),
        })?;
    let (archived, completed, agent_id, display_name, claimed_at, available) = root;
    let claim = agent_id.map(|id| TaskClaim {
        agent: ClaimAgent {
            id,
            display_name: display_name.expect("joined claim agent"),
        },
        claimed_at: claimed_at.expect("joined claim timestamp"),
    });
    let items = load_unresolved_dependency_explanations(connection, task_id, database_path)?;
    let (direct, recursive): (Vec<_>, Vec<_>) = items.into_iter().partition(|(direct, _)| *direct);
    let direct: Vec<_> = direct.into_iter().map(|(_, item)| item).collect();
    let recursive: Vec<_> = recursive.into_iter().map(|(_, item)| item).collect();
    let mut reasons = Vec::new();
    if archived {
        reasons.push(AvailabilityReason::Archived);
    }
    if completed {
        reasons.push(AvailabilityReason::Completed);
    }
    if claim.is_some() {
        reasons.push(AvailabilityReason::Claimed);
    }
    if !direct.is_empty() {
        reasons.push(AvailabilityReason::DependenciesBlocked);
    }
    debug_assert_eq!(available, reasons.is_empty());
    Ok(TaskBlockingExplanation {
        task_id: task_id.to_owned(),
        available,
        reasons,
        claim,
        unresolved_dependencies: UnresolvedDependencies { direct, recursive },
    })
}

pub fn unresolved_upstream_task_ids(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Vec<String>, Error> {
    let mut statement = connection
        .prepare(
            "SELECT u.id FROM task_dependencies d JOIN tasks u ON u.id = d.upstream_task_id
         JOIN statuses s ON s.id = u.status_id WHERE d.downstream_task_id = ?1
         AND NOT u.archived AND NOT s.completed ORDER BY u.id",
        )
        .map_err(|error| sqlite_access_error("unresolved upstream tasks", database_path, error))?;
    statement
        .query_map([task_id], |row| row.get(0))
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("unresolved upstream tasks", database_path, error))
}

fn load_unresolved_dependency_explanations(
    connection: &rusqlite::Connection,
    task_id: &str,
    database_path: &Path,
) -> Result<Vec<(bool, DependencyExplanationItem)>, Error> {
    let sql = "WITH RECURSIVE reachable(id) AS (
      SELECT u.id FROM task_dependencies d JOIN tasks u ON u.id = d.upstream_task_id
      JOIN statuses s ON s.id = u.status_id WHERE d.downstream_task_id = ?1 AND NOT u.archived AND NOT s.completed
      UNION
      SELECT u.id FROM reachable r JOIN task_dependencies d ON d.downstream_task_id = r.id
      JOIN tasks u ON u.id = d.upstream_task_id JOIN statuses s ON s.id = u.status_id
      WHERE NOT u.archived AND NOT s.completed
    )
    SELECT EXISTS (SELECT 1 FROM task_dependencies direct WHERE direct.downstream_task_id = ?1 AND direct.upstream_task_id = t.id),
      t.id, t.title, t.task_type, s.code, s.name, s.completed, blocked.id
    FROM reachable n JOIN tasks t ON t.id = n.id JOIN statuses s ON s.id = t.status_id
    LEFT JOIN task_dependencies d ON d.downstream_task_id = t.id
    LEFT JOIN tasks blocked ON blocked.id = d.upstream_task_id AND NOT blocked.archived
      AND NOT (SELECT completed FROM statuses WHERE id = blocked.status_id)
      AND EXISTS (SELECT 1 FROM reachable included WHERE included.id = blocked.id)
    ORDER BY t.id, blocked.id";
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| sqlite_access_error("blocking dependency graph", database_path, error))?;
    let rows = statement
        .query_map([task_id], |row| {
            Ok((
                row.get::<_, bool>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                task_type_from_row(row.get::<_, String>(3)?)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, bool>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })
        .map_err(|error| sqlite_access_error("blocking dependency graph", database_path, error))?;
    let rows: Vec<_> = rows
        .collect::<rusqlite::Result<_>>()
        .map_err(|error| sqlite_access_error("blocking dependency graph", database_path, error))?;
    let mut result: Vec<(bool, DependencyExplanationItem)> = Vec::new();
    for (direct, id, title, task_type, code, name, completed, blocked) in rows {
        if let Some((_, item)) = result.iter_mut().find(|(_, item)| item.id == id) {
            if let Some(blocked) = blocked {
                item.blocked_by_task_ids.push(blocked);
            }
        } else {
            result.push((
                direct,
                DependencyExplanationItem {
                    id,
                    title,
                    task_type,
                    status: RelatedTaskStatus {
                        code,
                        name,
                        completed,
                    },
                    blocked_by_task_ids: blocked.into_iter().collect(),
                },
            ));
        }
    }
    Ok(result)
}

/// Selects available tasks using a caller-owned connection or transaction.
///
/// The caller controls the snapshot and transaction behavior, allowing the same
/// predicate and ordering to be reused by a future atomic select-and-claim flow.
pub fn select_available_tasks(
    connection: &rusqlite::Connection,
    input: &AvailableTasksInput,
    database_path: &Path,
) -> Result<Vec<TaskListItem>, Error> {
    select_available_tasks_limited(connection, input, database_path, None)
}

fn select_first_available_task(
    connection: &rusqlite::Connection,
    input: &AvailableTasksInput,
    database_path: &Path,
) -> Result<Option<TaskListItem>, Error> {
    Ok(
        select_available_tasks_limited(connection, input, database_path, Some(1))?
            .into_iter()
            .next(),
    )
}

fn select_available_tasks_limited(
    connection: &rusqlite::Connection,
    input: &AvailableTasksInput,
    database_path: &Path,
    limit: Option<usize>,
) -> Result<Vec<TaskListItem>, Error> {
    validate_status_filters(connection, &input.status_codes, database_path)?;
    let (mut sql, values) = available_sql(input);
    if let Some(limit) = limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| sqlite_access_error("available task query", database_path, error))?;
    let rows = statement
        .query_map(params_from_iter(values), task_list_item_from_row)
        .map_err(|error| sqlite_access_error("available task query", database_path, error))?;
    let mut tasks: Vec<_> = rows
        .collect::<rusqlite::Result<_>>()
        .map_err(|error| sqlite_access_error("available task query", database_path, error))?;
    drop(statement);
    load_list_tags(connection, &mut tasks, database_path)?;
    Ok(tasks)
}

fn validate_status_filters(
    connection: &rusqlite::Connection,
    status_codes: &[String],
    database_path: &Path,
) -> Result<(), Error> {
    for code in status_codes.iter().collect::<HashSet<_>>() {
        if status::find_by_code(connection, code)
            .map_err(|error| sqlite_access_error("task status filter", database_path, error))?
            .is_none()
        {
            return Err(Error::StatusNotFound { code: code.clone() });
        }
    }
    Ok(())
}

fn available_sql(input: &AvailableTasksInput) -> (String, Vec<Value>) {
    let mut sql = String::from(
        "SELECT DISTINCT t.id, t.title, t.task_type, s.id, s.code, s.name, s.completed, t.priority, t.estimate_hours, t.archived, t.created_at, t.updated_at, NULL, NULL, NULL FROM tasks t JOIN statuses s ON s.id = t.status_id WHERE t.archived = 0 AND s.completed = 0 AND NOT EXISTS (SELECT 1 FROM task_claims c WHERE c.task_id = t.id) AND NOT EXISTS (SELECT 1 FROM task_dependencies d JOIN tasks upstream ON upstream.id = d.upstream_task_id JOIN statuses upstream_status ON upstream_status.id = upstream.status_id WHERE d.downstream_task_id = t.id AND upstream.archived = 0 AND upstream_status.completed = 0)",
    );
    let mut values = Vec::new();
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

fn task_list_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskListItem> {
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
    transaction: &rusqlite::Connection,
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

    fn empty_update() -> UpdateTaskInput {
        UpdateTaskInput {
            title: None,
            description: None,
            goal: None,
            acceptance_criteria: None,
            task_type: None,
            status_code: None,
            priority: None,
            estimate: PatchValue::Omitted,
            tags: PatchValue::Omitted,
            external_urls: PatchValue::Omitted,
            code_references: PatchValue::Omitted,
            agent_id: None,
        }
    }

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

    #[test]
    fn scalar_update_normalization_trims_title_and_checks_priority() {
        let mut input = empty_update();
        input.title = Some("  New title  ".to_owned());
        input.priority = Some(1_000_000);
        normalize_update_input(&mut input).unwrap();
        assert_eq!(input.title.as_deref(), Some("New title"));

        input.title = Some("   ".to_owned());
        assert!(matches!(
            normalize_update_input(&mut input),
            Err(Error::InvalidTaskTitle)
        ));

        let mut input = empty_update();
        input.priority = Some(-1);
        assert!(matches!(
            normalize_update_input(&mut input),
            Err(Error::InvalidTaskUpdatePriority)
        ));
    }
}
