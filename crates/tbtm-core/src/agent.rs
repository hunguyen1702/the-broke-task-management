use crate::{
    AccessIntent, Error, require_latest_migration, resolve_repository, sqlite_access_error,
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentList {
    pub agents: Vec<AgentWithClaims>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentWithClaims {
    pub id: String,
    pub base_name: String,
    pub display_name: String,
    pub created_at: String,
    pub claims: Vec<CurrentClaimSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentClaimSummary {
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub claimed_at: String,
}

pub fn list_agents(current: &Path) -> Result<AgentList, Error> {
    let mut resolved = resolve_repository(current, AccessIntent::ReadOnly)?;
    let database_path = resolved.database_path.clone();
    require_latest_migration(resolved.connection_mut(), &database_path)?;
    let transaction = resolved
        .connection_mut()
        .transaction()
        .map_err(|error| sqlite_access_error("agent list snapshot", &database_path, error))?;
    let mut statement = transaction
        .prepare(
            "SELECT a.id, a.base_name, a.display_name, a.created_at,
                    c.task_id, t.title, s.name, c.claimed_at
             FROM agents a
             LEFT JOIN task_claims c ON c.agent_id = a.id
             LEFT JOIN tasks t ON t.id = c.task_id
             LEFT JOIN statuses s ON s.id = t.status_id
             ORDER BY a.created_at, a.id, c.claimed_at, c.task_id",
        )
        .map_err(|error| sqlite_access_error("agent list query", &database_path, error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })
        .map_err(|error| sqlite_access_error("agent list query", &database_path, error))?;
    let mut agents: Vec<AgentWithClaims> = Vec::new();
    for row in rows {
        let (id, base_name, display_name, created_at, task_id, title, status, claimed_at) =
            row.map_err(|error| sqlite_access_error("agent list row", &database_path, error))?;
        if agents.last().is_none_or(|agent| agent.id != id) {
            agents.push(AgentWithClaims {
                id,
                base_name,
                display_name,
                created_at,
                claims: vec![],
            });
        }
        match (task_id, title, status, claimed_at) {
            (None, None, None, None) => {}
            (Some(task_id), Some(title), Some(status), Some(claimed_at)) => {
                agents
                    .last_mut()
                    .expect("agent row exists")
                    .claims
                    .push(CurrentClaimSummary {
                        task_id,
                        title,
                        status,
                        claimed_at,
                    });
            }
            _ => {
                return Err(Error::phase(
                    "DATABASE_INTEGRITY_ERROR",
                    std::io::Error::other("active claim has incomplete task or status data"),
                ));
            }
        }
    }
    drop(statement);
    transaction
        .commit()
        .map_err(|error| sqlite_access_error("agent list snapshot", &database_path, error))?;
    Ok(AgentList { agents })
}
