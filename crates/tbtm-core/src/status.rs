use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

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
