use rusqlite::Connection;
use serde_json::Value;
use std::{fs, process::Command};
use tempfile::tempdir;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn initialize(current: &std::path::Path) {
    let output = tbtm(current, &["init", "--prefix", "example", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn register_json_from_nested_directory_creates_distinct_identities() {
    let temp = tempdir().unwrap();
    assert!(
        Command::new("git")
            .current_dir(temp.path())
            .args(["init", "--quiet"])
            .status()
            .unwrap()
            .success()
    );
    initialize(temp.path());
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let first = tbtm(&nested, &["agent", "register", "Claude Agent", "--json"]);
    let second = tbtm(
        temp.path(),
        &["agent", "register", "Claude Agent", "--json"],
    );

    assert!(
        first.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(second.status.success());
    let first: Value = serde_json::from_slice(&first.stdout).unwrap();
    let second: Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(first["data"]["baseName"], "claude-agent");
    assert_eq!(second["data"]["baseName"], "claude-agent");
    assert_ne!(first["data"]["id"], second["data"]["id"]);
    assert_ne!(first["data"]["displayName"], second["data"]["displayName"]);
    assert_eq!(first["error"], Value::Null);
    let count: i64 = Connection::open(temp.path().join(".tbtm/tbtm.db"))
        .unwrap()
        .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn invalid_name_has_stable_json_error_and_does_not_write() {
    let temp = tempdir().unwrap();
    initialize(temp.path());

    let output = tbtm(temp.path(), &["agent", "register", "中文", "--json"]);

    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "INVALID_AGENT_NAME");
    assert_eq!(response["error"]["details"]["minimumLength"], 1);
    assert_eq!(response["error"]["details"]["maximumLength"], 48);
    let count: i64 = Connection::open(temp.path().join(".tbtm/tbtm.db"))
        .unwrap()
        .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn human_success_lists_identity_fields() {
    let temp = tempdir().unwrap();
    initialize(temp.path());

    let output = tbtm(temp.path(), &["agent", "register", "Quản Lý"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for field in [
        "Agent ID:",
        "Base name: quan-ly",
        "Display name: quan-ly-",
        "Created at:",
    ] {
        assert!(stdout.contains(field), "missing {field:?} in {stdout:?}");
    }
}

#[test]
fn registration_upgrades_an_older_schema_one_repository() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version >= 2", [])
        .unwrap();
    connection
        .execute("DROP TABLE task_code_references", [])
        .unwrap();
    connection
        .execute("DROP TABLE task_external_urls", [])
        .unwrap();
    connection.execute("DROP TABLE task_tags", []).unwrap();
    connection.execute("DROP TABLE tasks", []).unwrap();
    connection.execute("DROP TABLE agents", []).unwrap();
    drop(connection);
    let config_before = fs::read(temp.path().join(".tbtm/config.json")).unwrap();

    let output = tbtm(temp.path(), &["agent", "register", "agent", "--json"]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(temp.path().join(".tbtm/config.json")).unwrap(),
        config_before
    );
    let connection = Connection::open(database).unwrap();
    let versions: Vec<i64> = connection
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(versions, [1, 2, 3]);
}

#[test]
fn simultaneous_registrations_remain_unique() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = temp.path().to_path_buf();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let root = root.clone();
            std::thread::spawn(move || {
                let output = tbtm(&root, &["agent", "register", "worker", "--json"]);
                assert!(
                    output.status.success(),
                    "stdout={} stderr={}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                serde_json::from_slice::<Value>(&output.stdout).unwrap()
            })
        })
        .collect();
    let responses: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();

    let mut ids: Vec<_> = responses
        .iter()
        .map(|response| response["data"]["id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 8);
    let connection = Connection::open(root.join(".tbtm/tbtm.db")).unwrap();
    let (rows, distinct_ids, distinct_names): (i64, i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), COUNT(DISTINCT id), COUNT(DISTINCT display_name) FROM agents",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!((rows, distinct_ids, distinct_names), (8, 8, 8));
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}
