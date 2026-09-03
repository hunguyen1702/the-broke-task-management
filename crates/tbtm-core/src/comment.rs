use crate::{
    AccessIntent, Error, apply_pending_migrations, require_latest_migration, resolve_repository,
    sqlite_access_error, utc_now,
};
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

const COMMENT_ID_RETRY_LIMIT: usize = 8;

#[derive(Debug, Clone)]
pub struct AddCommentInput {
    pub task_id: String,
    pub content: String,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskComment {
    pub id: String,
    pub task_id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
    #[serde(skip)]
    pub author_display_name: Option<String>,
}

pub fn add_comment(current: &Path, input: AddCommentInput) -> Result<TaskComment, Error> {
    add_comment_with(current, input, Uuid::new_v4)
}

fn add_comment_with(
    current: &Path,
    input: AddCommentInput,
    mut generate_id: impl FnMut() -> Uuid,
) -> Result<TaskComment, Error> {
    if input.content.trim().is_empty() {
        return Err(Error::InvalidCommentContent);
    }
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection_mut()
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    for _ in 0..COMMENT_ID_RETRY_LIMIT {
        let id = generate_id().to_string();
        match insert_comment(&mut resolved, &input, id) {
            Err(error) if is_comment_id_collision(&error) => continue,
            result => return result,
        }
    }
    Err(Error::phase(
        "COMMENT_CREATION_FAILED",
        std::io::Error::other("unique comment ID retry limit exhausted"),
    ))
}

fn insert_comment(
    resolved: &mut crate::ResolvedRepository,
    input: &AddCommentInput,
    id: String,
) -> Result<TaskComment, Error> {
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("comment creation", &database_path, error))?;
    require_task(&transaction, &input.task_id, &database_path)?;
    let author_display_name = match input.agent_id {
        Some(agent_id) => Some(
            transaction
                .query_row(
                    "SELECT display_name FROM agents WHERE id = ?1",
                    [agent_id.to_string()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| {
                    sqlite_access_error("comment actor lookup", &database_path, error)
                })?
                .ok_or(Error::AgentNotFound { id: agent_id })?,
        ),
        None => None,
    };
    let author = input
        .agent_id
        .map_or_else(|| "user".to_owned(), |id| id.to_string());
    let actor_type = if input.agent_id.is_some() {
        "agent"
    } else {
        "user"
    };
    let created_at = utc_now();
    transaction
        .execute(
            "INSERT INTO task_comments (id, task_id, content, author_actor_type, author_agent_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, input.task_id, input.content, actor_type, input.agent_id.map(|value| value.to_string()), created_at],
        )
        .map_err(|error| sqlite_access_error("comment creation", &database_path, error))?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("comment creation", &database_path, error))?;
    Ok(TaskComment {
        id,
        task_id: input.task_id.clone(),
        content: input.content.clone(),
        author,
        created_at,
        author_display_name,
    })
}

pub fn list_comments(current: &Path, task_id: &str) -> Result<Vec<TaskComment>, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    require_latest_migration(resolved.connection(), &resolved.database_path)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("comment listing", &database_path, error))?;
    require_task(&transaction, task_id, &database_path)?;
    let comments = select_comments(&transaction, task_id, &database_path)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("comment listing", &database_path, error))?;
    Ok(comments)
}

fn require_task(
    transaction: &Transaction<'_>,
    task_id: &str,
    database_path: &Path,
) -> Result<(), Error> {
    let exists = transaction
        .query_row("SELECT 1 FROM tasks WHERE id = ?1", [task_id], |_| Ok(()))
        .optional()
        .map_err(|error| sqlite_access_error("comment task lookup", database_path, error))?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(Error::TaskNotFound {
            id: task_id.to_owned(),
        })
    }
}

fn select_comments(
    transaction: &Transaction<'_>,
    task_id: &str,
    database_path: &Path,
) -> Result<Vec<TaskComment>, Error> {
    let mut statement = transaction
        .prepare(
            "SELECT c.id, c.task_id, c.content, c.author_actor_type, c.author_agent_id, c.created_at, a.display_name FROM task_comments c LEFT JOIN agents a ON a.id = c.author_agent_id WHERE c.task_id = ?1 ORDER BY c.created_at ASC, c.id ASC",
        )
        .map_err(|error| sqlite_access_error("comment listing", database_path, error))?;
    statement
        .query_map([task_id], |row| {
            let actor_type: String = row.get(3)?;
            let agent_id: Option<String> = row.get(4)?;
            Ok(TaskComment {
                id: row.get(0)?,
                task_id: row.get(1)?,
                content: row.get(2)?,
                author: if actor_type == "user" {
                    "user".to_owned()
                } else {
                    agent_id.expect("database constraint requires agent ID")
                },
                created_at: row.get(5)?,
                author_display_name: row.get(6)?,
            })
        })
        .and_then(Iterator::collect)
        .map_err(|error| sqlite_access_error("comment listing", database_path, error))
}

fn is_comment_id_collision(error: &Error) -> bool {
    matches!(
        error,
        Error::DatabaseUnavailable { message, .. } if message.contains("task_comments.id")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{CreateTaskInput, TaskType, create_task};
    use tempfile::tempdir;

    fn task_input() -> CreateTaskInput {
        CreateTaskInput {
            title: "Comment target".to_owned(),
            task_type: TaskType::Task,
            description: String::new(),
            goal: String::new(),
            acceptance_criteria: String::new(),
            status_code: "to_do".to_owned(),
            priority: 50,
            estimate: None,
            tags: vec![],
            external_urls: vec![],
            code_references: vec![],
            agent_id: None,
        }
    }

    #[test]
    fn retries_only_comment_id_collisions_and_exhausts_without_partial_rows() {
        let temp = tempdir().unwrap();
        crate::initialize(temp.path(), Some("project"), false, false, false).unwrap();
        let task = create_task(temp.path(), task_input()).unwrap();
        let collision = Uuid::new_v4();
        let first = add_comment_with(
            temp.path(),
            AddCommentInput {
                task_id: task.id.clone(),
                content: "first".to_owned(),
                agent_id: None,
            },
            || collision,
        )
        .unwrap();
        let replacement = Uuid::new_v4();
        let mut ids = [collision, replacement].into_iter();
        let second = add_comment_with(
            temp.path(),
            AddCommentInput {
                task_id: task.id.clone(),
                content: "second".to_owned(),
                agent_id: None,
            },
            || ids.next().unwrap(),
        )
        .unwrap();
        assert_eq!(second.id, replacement.to_string());
        let exhausted = add_comment_with(
            temp.path(),
            AddCommentInput {
                task_id: task.id.clone(),
                content: "never inserted".to_owned(),
                agent_id: None,
            },
            || collision,
        )
        .unwrap_err();
        assert_eq!(exhausted.code(), "COMMENT_CREATION_FAILED");
        let comments = list_comments(temp.path(), &task.id).unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, first.id);
        assert!(!comments.iter().any(|item| item.content == "never inserted"));
    }
}
