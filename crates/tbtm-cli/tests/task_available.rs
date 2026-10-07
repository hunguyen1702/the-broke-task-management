use rusqlite::{Connection, params};
use serde_json::Value;
use std::{path::Path, process::Command};
use tbtm_core::task::{AvailableTasksInput, select_available_tasks};
use tempfile::tempdir;

fn tbtm(current: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn initialize(current: &Path) {
    assert!(
        tbtm(current, &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
}

fn create(current: &Path, title: &str, extra: &[&str]) -> Value {
    let mut arguments = vec!["task", "create", "--title", title, "--type", "task"];
    arguments.extend_from_slice(extra);
    arguments.push("--json");
    let output = tbtm(current, &arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn data(current: &Path, arguments: &[&str]) -> Value {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

#[test]
fn available_applies_every_predicate_and_effective_upstream_completion() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let eligible = create(temp.path(), "Eligible", &[]);
    let completed = create(temp.path(), "Completed", &["--status", "done"]);
    let archived = create(temp.path(), "Archived", &[]);
    let claimed = create(temp.path(), "Claimed", &[]);
    let unresolved = create(temp.path(), "Unresolved upstream", &[]);
    let archived_upstream = create(temp.path(), "Archived upstream", &[]);
    let completed_upstream = create(temp.path(), "Completed upstream", &["--status", "done"]);
    let blocked = create(temp.path(), "Blocked", &[]);
    let resolved = create(temp.path(), "Resolved", &[]);
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id IN (?1, ?2)",
            params![
                archived["id"].as_str().unwrap(),
                archived_upstream["id"].as_str().unwrap()
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO agents (id, base_name, display_name, created_at) VALUES ('agent-1', 'Agent', 'Agent', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, 'agent-1', '2026-01-01T00:00:00Z')",
            [claimed["id"].as_str().unwrap()],
        )
        .unwrap();
    for (downstream, upstream) in [
        (&blocked, &unresolved),
        (&resolved, &archived_upstream),
        (&resolved, &completed_upstream),
    ] {
        connection
            .execute(
                "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
                params![downstream["id"].as_str().unwrap(), upstream["id"].as_str().unwrap()],
            )
            .unwrap();
    }

    let tasks = data(temp.path(), &["task", "available", "--json"]);
    let titles: Vec<_> = tasks
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["title"].as_str().unwrap())
        .collect();
    assert!(titles.contains(&"Eligible"));
    assert!(titles.contains(&"Resolved"));
    for excluded in ["Completed", "Archived", "Claimed", "Blocked"] {
        assert!(!titles.contains(&excluded));
    }
    for task in tasks.as_array().unwrap() {
        assert_eq!(task["archived"], false);
        assert_eq!(task["status"]["completed"], false);
        assert!(task["claim"].is_null());
        assert!(task.get("available").is_none());
    }

    connection
        .execute(
            "UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1",
            [unresolved["id"].as_str().unwrap()],
        )
        .unwrap();
    let changed = data(temp.path(), &["task", "available", "--json"]);
    assert!(
        changed
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["title"] == "Blocked")
    );
    assert_ne!(eligible["id"], completed["id"]);
}

#[test]
fn available_filters_orders_and_does_not_duplicate_rows() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let older = create(
        temp.path(),
        "Older",
        &[
            "--priority",
            "90",
            "--status",
            "in_progress",
            "--tag",
            "Rust",
            "--tag",
            "CLI",
        ],
    );
    let newer = create(temp.path(), "Newer", &["--priority", "90", "--tag", "Rust"]);
    create(temp.path(), "Low", &["--priority", "10", "--tag", "Rust"]);
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-01T00:00:00Z' WHERE id = ?1",
            [older["id"].as_str().unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-02T00:00:00Z' WHERE id = ?1",
            [newer["id"].as_str().unwrap()],
        )
        .unwrap();

    let tasks = data(
        temp.path(),
        &[
            "task",
            "available",
            "--status",
            "to_do",
            "--status",
            "in_progress",
            "--type",
            "task",
            "--tag",
            "Rust",
            "--tag",
            "CLI",
            "--json",
        ],
    );
    let titles: Vec<_> = tasks
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["Older", "Newer", "Low"]);
    assert_eq!(titles.iter().filter(|title| **title == "Older").count(), 1);

    let wrong_case = data(
        temp.path(),
        &["task", "available", "--tag", "rust", "--json"],
    );
    assert_eq!(wrong_case, serde_json::json!([]));
    let completed = data(
        temp.path(),
        &["task", "available", "--status", "done", "--json"],
    );
    assert_eq!(completed, serde_json::json!([]));
}

#[test]
fn available_breaks_equal_priority_and_creation_time_ties_by_task_id() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let low = create(temp.path(), "Low", &["--priority", "10"]);
    let a = create(temp.path(), "A", &["--priority", "90"]);
    let b = create(temp.path(), "B", &["--priority", "90"]);
    let older = create(temp.path(), "Older", &["--priority", "90"]);
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-02T00:00:00Z' WHERE id IN (?1, ?2)",
            params![a["id"].as_str().unwrap(), b["id"].as_str().unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET created_at = '2026-01-01T00:00:00Z' WHERE id = ?1",
            [older["id"].as_str().unwrap()],
        )
        .unwrap();
    let actual: Vec<String> = data(temp.path(), &["task", "available", "--json"])
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["id"].as_str().unwrap().to_owned())
        .collect();
    let mut tied = [a["id"].as_str().unwrap(), b["id"].as_str().unwrap()];
    tied.sort();
    assert_eq!(
        actual,
        [
            older["id"].as_str().unwrap(),
            tied[0],
            tied[1],
            low["id"].as_str().unwrap()
        ]
    );
}

#[test]
fn available_outputs_and_errors_use_stable_contracts() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let empty = tbtm(temp.path(), &["task", "available"]);
    assert!(empty.status.success());
    assert_eq!(
        String::from_utf8(empty.stdout).unwrap(),
        "No available tasks found.\n"
    );

    for (arguments, code, exit) in [
        (
            vec!["task", "available", "--status", "missing", "--json"],
            "STATUS_NOT_FOUND",
            3,
        ),
        (
            vec!["task", "available", "--type", "feature", "--json"],
            "INVALID_TASK_TYPE",
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
fn selector_accepts_a_caller_owned_transaction() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    create(temp.path(), "Selected", &[]);
    let database = temp.path().join(".tbtm/tbtm.db");
    let mut connection = Connection::open(&database).unwrap();
    let transaction = connection.transaction().unwrap();
    let tasks =
        select_available_tasks(&transaction, &AvailableTasksInput::default(), &database).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Selected");
    transaction.commit().unwrap();
}
