use crate::{
    AccessIntent, Error, apply_pending_migrations, require_latest_migration, resolve_repository,
    sqlite_access_error,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;
use std::{path::Path, time::Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub id: String,
    pub code: String,
    pub name: String,
    pub completed: bool,
    pub display_order: i64,
    pub is_default: bool,
}

pub fn find_by_code(connection: &Connection, code: &str) -> rusqlite::Result<Option<Status>> {
    connection
        .query_row(
            "SELECT id, code, name, completed, display_order, is_default
             FROM statuses
             WHERE code = ?1",
            [code],
            |row| {
                Ok(Status {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    completed: row.get(3)?,
                    display_order: row.get(4)?,
                    is_default: row.get(5)?,
                })
            },
        )
        .optional()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement<'a> {
    Before(&'a str),
    After(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStatusInput<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub placement: Option<Placement<'a>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MoveStatusResult {
    pub status: Status,
    pub statuses: Vec<Status>,
}

pub fn list_statuses(current: &Path) -> Result<Vec<Status>, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    require_latest_migration(resolved.connection(), &resolved.database_path)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("status listing", &database_path, error))?;
    let statuses = list_with(&transaction).map_err(status_operation_error)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("status listing", &database_path, error))?;
    Ok(statuses)
}

pub fn create_status(current: &Path, input: CreateStatusInput<'_>) -> Result<Status, Error> {
    validate_code(input.code)?;
    let name = normalize_name(input.name)?;
    mutate(current, |transaction| {
        let mut statuses = list_with(transaction).map_err(status_operation_error)?;
        let insertion = match input.placement {
            None => statuses.len(),
            Some(Placement::Before(target)) => position(&statuses, target, "target")?,
            Some(Placement::After(target)) => position(&statuses, target, "target")? + 1,
        };
        if find_by_code(transaction, input.code)
            .map_err(status_operation_error)?
            .is_some()
        {
            return Err(Error::StatusCodeConflict {
                code: input.code.to_owned(),
            });
        }
        if name_exists(transaction, &name, None).map_err(status_operation_error)? {
            return Err(Error::StatusNameConflict { name: name.clone() });
        }
        let status = Status {
            id: Uuid::new_v4().to_string(),
            code: input.code.to_owned(),
            name,
            completed: false,
            display_order: insertion as i64,
            is_default: false,
        };
        transaction
            .execute(
                "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
             VALUES (?1, ?2, ?3, 0, -1, 0)",
                params![status.id, status.code, status.name],
            )
            .map_err(status_operation_error)?;
        statuses.insert(insertion, status.clone());
        write_order(transaction, &statuses).map_err(status_operation_error)?;
        find_by_code(transaction, &status.code)
            .map_err(status_operation_error)?
            .ok_or_else(|| Error::StatusLookupNotFound {
                code: status.code,
                role: "source",
            })
    })
}

pub fn rename_status(current: &Path, code: &str, requested_name: &str) -> Result<Status, Error> {
    let name = normalize_name(requested_name)?;
    mutate(current, |transaction| {
        let status = find_required(transaction, code, "source")?;
        if status.is_default {
            return Err(Error::StatusDefaultImmutable {
                code: code.to_owned(),
            });
        }
        if status.name == name {
            return Ok(status);
        }
        if name_exists(transaction, &name, Some(&status.id)).map_err(status_operation_error)? {
            return Err(Error::StatusNameConflict { name: name.clone() });
        }
        transaction
            .execute(
                "UPDATE statuses SET name = ?1 WHERE id = ?2",
                params![name, status.id],
            )
            .map_err(status_operation_error)?;
        find_required(transaction, code, "source")
    })
}

pub fn move_status(
    current: &Path,
    code: &str,
    placement: Placement<'_>,
) -> Result<MoveStatusResult, Error> {
    let target = match placement {
        Placement::Before(value) | Placement::After(value) => value,
    };
    if code == target {
        return Err(Error::InvalidStatusPosition {
            code: code.to_owned(),
            target_code: target.to_owned(),
        });
    }
    mutate(current, |transaction| {
        let mut statuses = list_with(transaction).map_err(status_operation_error)?;
        let source_index = position(&statuses, code, "source")?;
        position(&statuses, target, "target")?;
        let moved = statuses.remove(source_index);
        let target_index = statuses
            .iter()
            .position(|status| status.code == target)
            .expect("validated target remains after removing distinct source");
        let insertion = match placement {
            Placement::Before(_) => target_index,
            Placement::After(_) => target_index + 1,
        };
        statuses.insert(insertion, moved);
        let changed = statuses
            .iter()
            .enumerate()
            .any(|(index, status)| status.display_order != index as i64);
        if changed {
            write_order(transaction, &statuses).map_err(status_operation_error)?;
        }
        let statuses = list_with(transaction).map_err(status_operation_error)?;
        let status = statuses
            .iter()
            .find(|status| status.code == code)
            .cloned()
            .expect("moved status exists");
        Ok(MoveStatusResult { status, statuses })
    })
}

pub fn validate_code(code: &str) -> Result<(), Error> {
    let mut chars = code.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return Err(Error::InvalidStatusCode {
            code: code.to_owned(),
        });
    }
    Ok(())
}

fn normalize_name(name: &str) -> Result<String, Error> {
    let normalized = name.trim().to_owned();
    if normalized.is_empty() {
        Err(Error::InvalidStatusName { name: normalized })
    } else {
        Ok(normalized)
    }
}

fn mutate<T>(
    current: &Path,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, Error>,
) -> Result<T, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadWrite)?;
    resolved
        .connection()
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| {
            sqlite_access_error("database busy timeout", &resolved.database_path, error)
        })?;
    apply_pending_migrations(&mut resolved)?;
    let database_path = resolved.database_path.clone();
    let transaction = resolved
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| sqlite_access_error("status mutation", &database_path, error))?;
    let result = operation(&transaction)?;
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("status mutation", &database_path, error))?;
    Ok(result)
}

fn list_with(connection: &Connection) -> rusqlite::Result<Vec<Status>> {
    let mut statement = connection.prepare("SELECT id, code, name, completed, display_order, is_default FROM statuses ORDER BY display_order, code")?;
    statement
        .query_map([], |row| {
            Ok(Status {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                completed: row.get(3)?,
                display_order: row.get(4)?,
                is_default: row.get(5)?,
            })
        })?
        .collect()
}

fn find_required(connection: &Connection, code: &str, role: &'static str) -> Result<Status, Error> {
    find_by_code(connection, code)
        .map_err(status_operation_error)?
        .ok_or_else(|| Error::StatusLookupNotFound {
            code: code.to_owned(),
            role,
        })
}

fn position(statuses: &[Status], code: &str, role: &'static str) -> Result<usize, Error> {
    statuses
        .iter()
        .position(|status| status.code == code)
        .ok_or_else(|| Error::StatusLookupNotFound {
            code: code.to_owned(),
            role,
        })
}

fn name_exists(
    connection: &Connection,
    name: &str,
    except_id: Option<&str>,
) -> rusqlite::Result<bool> {
    connection.query_row("SELECT EXISTS(SELECT 1 FROM statuses WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id <> ?2))", params![name, except_id], |row| row.get(0))
}

fn write_order(transaction: &Transaction<'_>, statuses: &[Status]) -> rusqlite::Result<()> {
    transaction.execute(
        "UPDATE statuses SET display_order = display_order + 1000000",
        [],
    )?;
    for (index, status) in statuses.iter().enumerate() {
        transaction.execute(
            "UPDATE statuses SET display_order = ?1 WHERE id = ?2",
            params![index as i64, status.id],
        )?;
    }
    Ok(())
}

fn status_operation_error(error: rusqlite::Error) -> Error {
    Error::phase("STATUS_OPERATION_FAILED", error)
}
