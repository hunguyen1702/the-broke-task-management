use rusqlite::Connection;
use serde_json::Value;
use std::process::Command;
use tempfile::tempdir;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn initialize(current: &std::path::Path) {
    assert!(
        tbtm(current, &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
}

fn create(current: &std::path::Path, title: &str, task_type: &str, extra: &[&str]) -> Value {
    let mut arguments = vec!["task", "create", "--title", title, "--type", task_type];
    arguments.extend_from_slice(extra);
    arguments.extend_from_slice(&["--json"]);
    let output = tbtm(current, &arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

#[test]
fn detail_returns_full_normalized_contract_for_archived_task() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let created = create(
        temp.path(),
        "Fix parser",
        "bug",
        &[
            "--description",
            "Long details",
            "--goal",
            "Reliable parser",
            "--acceptance-criteria",
            "All tests pass",
            "--tag",
            "Rust",
            "--tag",
            "CLI",
            "--url",
            "https://example.com/1",
            "--code-ref",
            "src/lib.rs:2-4::parser",
        ],
    );
    let id = created["id"].as_str().unwrap();
    Connection::open(temp.path().join(".tbtm/tbtm.db"))
        .unwrap()
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [id],
        )
        .unwrap();

    let output = tbtm(temp.path(), &["task", "view", id, "--json"]);
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let task = &response["data"];
    assert_eq!(task["description"], "Long details");
    assert_eq!(task["tags"], serde_json::json!(["Rust", "CLI"]));
    assert_eq!(
        task["externalUrls"],
        serde_json::json!(["https://example.com/1"])
    );
    assert_eq!(task["codeReferences"][0]["description"], "parser");
    assert_eq!(
        task["hierarchy"],
        serde_json::json!({"parent": null, "children": []})
    );
    assert_eq!(
        task["dependencies"],
        serde_json::json!({"upstream": [], "downstream": []})
    );
    assert_eq!(task["archived"], true);
}

#[test]
fn list_filters_and_orders_with_compact_projection() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let low = create(
        temp.path(),
        "Low",
        "task",
        &["--priority", "10", "--tag", "Rust"],
    );
    let high_b = create(
        temp.path(),
        "High B",
        "bug",
        &[
            "--priority",
            "90",
            "--status",
            "in_progress",
            "--tag",
            "Rust",
        ],
    );
    let high_a = create(
        temp.path(),
        "High A",
        "task",
        &[
            "--priority",
            "90",
            "--status",
            "in_progress",
            "--tag",
            "CLI",
        ],
    );
    let db = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(db).unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-02T00:00:00Z' WHERE id = ?1",
            [high_b["id"].as_str().unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-01T00:00:00Z' WHERE id = ?1",
            [high_a["id"].as_str().unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [low["id"].as_str().unwrap()],
        )
        .unwrap();

    let output = tbtm(
        temp.path(),
        &[
            "task",
            "list",
            "--status",
            "in_progress",
            "--type",
            "task",
            "--type",
            "bug",
            "--json",
        ],
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tasks = response["data"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["title"], "High A");
    assert_eq!(tasks[1]["title"], "High B");
    for task in tasks {
        for excluded in [
            "description",
            "goal",
            "acceptanceCriteria",
            "externalUrls",
            "codeReferences",
            "createdBy",
            "hierarchy",
            "dependencies",
        ] {
            assert!(task.get(excluded).is_none(), "unexpected {excluded}");
        }
        assert!(task["claim"].is_null());
    }

    let archived = tbtm(
        temp.path(),
        &["task", "list", "--archived", "--tag", "Rust", "--json"],
    );
    let archived: Value = serde_json::from_slice(&archived.stdout).unwrap();
    assert_eq!(archived["data"].as_array().unwrap().len(), 1);
    let case_sensitive = tbtm(
        temp.path(),
        &["task", "list", "--all", "--tag", "rust", "--json"],
    );
    let case_sensitive: Value = serde_json::from_slice(&case_sensitive.stdout).unwrap();
    assert_eq!(case_sensitive["data"], serde_json::json!([]));
}

#[test]
fn empty_and_selector_errors_use_stable_contracts() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let empty = tbtm(temp.path(), &["task", "list"]);
    assert!(empty.status.success());
    assert_eq!(
        String::from_utf8(empty.stdout).unwrap(),
        "No tasks found.\n"
    );

    for (arguments, code, exit) in [
        (
            vec!["task", "view", "missing", "--json"],
            "TASK_NOT_FOUND",
            3,
        ),
        (
            vec!["task", "list", "--status", "missing", "--json"],
            "STATUS_NOT_FOUND",
            3,
        ),
        (
            vec!["task", "list", "--type", "feature", "--json"],
            "INVALID_TASK_TYPE",
            2,
        ),
        (
            vec!["task", "list", "--archived", "--all", "--json"],
            "CONFLICTING_ARGUMENTS",
            2,
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit));
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["error"]["code"], code);
    }
}

#[test]
fn list_uses_or_within_groups_and_and_across_groups() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    create(
        temp.path(),
        "Matching bug",
        "bug",
        &["--status", "to_do", "--tag", "backend"],
    );
    create(
        temp.path(),
        "Wrong tag",
        "task",
        &["--status", "in_progress", "--tag", "frontend"],
    );
    create(
        temp.path(),
        "Wrong status",
        "task",
        &["--status", "done", "--tag", "backend"],
    );
    let output = tbtm(
        temp.path(),
        &[
            "task",
            "list",
            "--status",
            "to_do",
            "--status",
            "in_progress",
            "--type",
            "bug",
            "--type",
            "task",
            "--tag",
            "backend",
            "--json",
        ],
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["data"].as_array().unwrap().len(), 1);
    assert_eq!(response["data"][0]["title"], "Matching bug");
}

#[test]
fn read_commands_do_not_change_database_bytes() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Read only", "task", &[]);
    let database = temp.path().join(".tbtm/tbtm.db");
    let before = std::fs::read(&database).unwrap();
    assert!(
        tbtm(
            temp.path(),
            &["task", "view", task["id"].as_str().unwrap(), "--json"]
        )
        .status
        .success()
    );
    assert!(
        tbtm(temp.path(), &["task", "list", "--json"])
            .status
            .success()
    );
    assert_eq!(std::fs::read(&database).unwrap(), before);
    assert!(!database.with_extension("db-wal").exists());
    assert!(!database.with_extension("db-shm").exists());
}
