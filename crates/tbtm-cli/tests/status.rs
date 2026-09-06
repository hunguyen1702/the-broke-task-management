use rusqlite::Connection;
use serde_json::Value;
use std::process::{Command, Output};
use tempfile::tempdir;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> Output {
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

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn list_create_rename_and_move_preserve_status_contract() {
    let temp = tempdir().unwrap();
    initialize(temp.path());

    let created = tbtm(
        temp.path(),
        &[
            "status", "create", "--code", "review", "--name", " Review ", "--before", "done",
            "--json",
        ],
    );
    assert!(created.status.success());
    let created = json(&created);
    assert_eq!(created["data"]["code"], "review");
    assert_eq!(created["data"]["name"], "Review");
    assert_eq!(created["data"]["completed"], false);
    assert_eq!(created["data"]["displayOrder"], 2);
    assert_eq!(created["data"]["isDefault"], false);

    let id = created["data"]["id"].clone();
    let renamed = tbtm(
        temp.path(),
        &[
            "status",
            "rename",
            "review",
            "--name",
            "QA Review",
            "--json",
        ],
    );
    assert!(renamed.status.success());
    assert_eq!(json(&renamed)["data"]["id"], id);

    let moved = tbtm(
        temp.path(),
        &["status", "move", "done", "--before", "to_do", "--json"],
    );
    assert!(moved.status.success());
    let moved = json(&moved);
    assert_eq!(moved["data"]["status"]["code"], "done");
    let codes: Vec<_> = moved["data"]["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|status| status["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["done", "to_do", "in_progress", "review"]);
    for (position, status) in moved["data"]["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(status["displayOrder"], position as i64);
    }

    let listed = tbtm(temp.path(), &["status", "list", "--json"]);
    assert!(listed.status.success());
    assert_eq!(json(&listed)["data"], moved["data"]["statuses"]);
}

#[test]
fn validation_conflicts_and_missing_roles_have_stable_errors_without_mutation() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let before = json(&tbtm(temp.path(), &["status", "list", "--json"]))["data"].clone();

    for (arguments, exit, code, details) in [
        (
            vec![
                "status", "create", "--code", "Bad", "--name", "Valid", "--json",
            ],
            2,
            "INVALID_STATUS_CODE",
            serde_json::json!({"code":"Bad"}),
        ),
        (
            vec![
                "status", "create", "--code", "valid", "--name", "   ", "--json",
            ],
            2,
            "INVALID_STATUS_NAME",
            serde_json::json!({"name":""}),
        ),
        (
            vec![
                "status", "create", "--code", "done", "--name", "Other", "--json",
            ],
            4,
            "STATUS_CODE_CONFLICT",
            serde_json::json!({"code":"done"}),
        ),
        (
            vec![
                "status", "create", "--code", "other", "--name", "todo", "--json",
            ],
            4,
            "STATUS_NAME_CONFLICT",
            serde_json::json!({"name":"todo"}),
        ),
        (
            vec!["status", "move", "missing", "--before", "done", "--json"],
            3,
            "STATUS_NOT_FOUND",
            serde_json::json!({"code":"missing","role":"source"}),
        ),
        (
            vec!["status", "move", "done", "--before", "missing", "--json"],
            3,
            "STATUS_NOT_FOUND",
            serde_json::json!({"code":"missing","role":"target"}),
        ),
        (
            vec!["status", "move", "done", "--before", "done", "--json"],
            2,
            "INVALID_STATUS_POSITION",
            serde_json::json!({"code":"done","targetCode":"done"}),
        ),
        (
            vec!["status", "rename", "done", "--name", "Done", "--json"],
            5,
            "STATUS_DEFAULT_IMMUTABLE",
            serde_json::json!({"code":"done"}),
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit), "{arguments:?}");
        let response = json(&output);
        assert_eq!(response["error"]["code"], code);
        assert_eq!(response["error"]["details"], details);
        assert_eq!(
            json(&tbtm(temp.path(), &["status", "list", "--json"]))["data"],
            before
        );
    }
}

#[test]
fn status_mutation_applies_pending_name_migration_while_list_does_not() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 9", [])
        .unwrap();
    connection
        .execute("DROP INDEX statuses_name_nocase", [])
        .unwrap();

    let list = tbtm(temp.path(), &["status", "list", "--json"]);
    assert_eq!(list.status.code(), Some(1));
    let create = tbtm(
        temp.path(),
        &[
            "status", "create", "--code", "review", "--name", "Review", "--json",
        ],
    );
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let version: i64 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(version, 9);
}

#[test]
fn status_help_has_required_placement_and_no_agent_authority() {
    let help = tbtm(tempdir().unwrap().path(), &["status", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for command in ["list", "create", "rename", "move", "set-completed"] {
        assert!(text.contains(command));
    }
    assert!(!text.contains("--agent"));

    let missing = tbtm(tempdir().unwrap().path(), &["status", "move", "done"]);
    assert_eq!(missing.status.code(), Some(2));
}

#[test]
fn migration_name_collision_rolls_back_and_valid_noops_do_not_write() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 9", [])
        .unwrap();
    connection
        .execute("DROP INDEX statuses_name_nocase", [])
        .unwrap();
    connection
        .execute(
            "INSERT INTO statuses (id, code, name, completed, display_order, is_default)
             VALUES ('collision', 'collision', 'todo', 0, 3, 0)",
            [],
        )
        .unwrap();
    drop(connection);

    let failed = tbtm(
        temp.path(),
        &[
            "status", "create", "--code", "review", "--name", "Review", "--json",
        ],
    );
    assert_eq!(failed.status.code(), Some(1));
    let connection = Connection::open(&database).unwrap();
    let applied: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = 9",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 0);
    let inserted: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM statuses WHERE code = 'review'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(inserted, 0);
    connection
        .execute("DELETE FROM statuses WHERE id = 'collision'", [])
        .unwrap();
    drop(connection);

    assert!(
        tbtm(
            temp.path(),
            &[
                "status", "create", "--code", "review", "--name", "Review", "--json",
            ],
        )
        .status
        .success()
    );
    let before = std::fs::read(&database).unwrap();
    assert!(
        tbtm(
            temp.path(),
            &["status", "rename", "review", "--name", " Review ", "--json"],
        )
        .status
        .success()
    );
    assert_eq!(std::fs::read(&database).unwrap(), before);
    assert!(
        tbtm(
            temp.path(),
            &["status", "move", "review", "--after", "done", "--json"],
        )
        .status
        .success()
    );
    assert_eq!(std::fs::read(database).unwrap(), before);
}

fn create_task(current: &std::path::Path, title: &str, status: &str) -> String {
    let output = tbtm(
        current,
        &[
            "task", "create", "--title", title, "--type", "task", "--status", status, "--json",
        ],
    );
    assert!(output.status.success());
    json(&output)["data"]["id"].as_str().unwrap().to_owned()
}

#[test]
fn set_completed_reports_exact_impact_requires_confirmation_and_changes_semantics() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let upstream = create_task(temp.path(), "Review work", "in_progress");
    let downstream = create_task(temp.path(), "Ship work", "to_do");
    assert!(
        tbtm(
            temp.path(),
            &[
                "task",
                "dependency",
                "add",
                &downstream,
                "--depends-on",
                &upstream,
                "--json"
            ]
        )
        .status
        .success()
    );

    let unconfirmed = tbtm(
        temp.path(),
        &[
            "status",
            "set-completed",
            "in_progress",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert_eq!(unconfirmed.status.code(), Some(2));
    let error = json(&unconfirmed);
    assert_eq!(error["error"]["code"], "CONFIRMATION_REQUIRED");
    assert_eq!(
        error["error"]["details"]["statusTasks"][0]["taskId"],
        upstream
    );
    assert_eq!(
        error["error"]["details"]["statusTasks"][0]["availableBefore"],
        true
    );
    assert_eq!(
        error["error"]["details"]["statusTasks"][0]["availableAfter"],
        false
    );
    assert_eq!(
        error["error"]["details"]["downstreamTasks"][0]["taskId"],
        downstream
    );
    assert_eq!(
        error["error"]["details"]["downstreamTasks"][0]["unresolvedUpstreamTaskIdsBefore"],
        serde_json::json!([upstream])
    );
    assert_eq!(
        error["error"]["details"]["downstreamTasks"][0]["unresolvedUpstreamTaskIdsAfter"],
        serde_json::json!([])
    );

    let changed = tbtm(
        temp.path(),
        &[
            "status",
            "set-completed",
            "in_progress",
            "--completed",
            "true",
            "--yes",
            "--json",
        ],
    );
    assert!(
        changed.status.success(),
        "{}",
        String::from_utf8_lossy(&changed.stderr)
    );
    let changed = json(&changed);
    assert_eq!(changed["data"]["status"]["completed"], true);
    let available = json(&tbtm(temp.path(), &["task", "available", "--json"]))["data"].clone();
    assert!(
        available
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == downstream)
    );

    let database = temp.path().join(".tbtm/tbtm.db");
    let before = std::fs::read(&database).unwrap();
    let noop = tbtm(
        temp.path(),
        &[
            "status",
            "set-completed",
            "in_progress",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert!(noop.status.success());
    assert_eq!(
        json(&noop)["data"]["impact"],
        serde_json::json!({"statusTasks":[],"downstreamTasks":[]})
    );
    assert_eq!(std::fs::read(database).unwrap(), before);
}

#[test]
fn set_completed_supports_empty_impact_and_stable_input_errors() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let changed = tbtm(
        temp.path(),
        &[
            "status",
            "set-completed",
            "in_progress",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert!(changed.status.success());
    assert_eq!(
        json(&changed)["data"]["impact"],
        serde_json::json!({"statusTasks":[],"downstreamTasks":[]})
    );

    let missing = tbtm(
        temp.path(),
        &[
            "status",
            "set-completed",
            "missing",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        json(&missing)["error"]["details"],
        serde_json::json!({"code":"missing","role":"source"})
    );
    assert_eq!(
        tbtm(
            temp.path(),
            &["status", "set-completed", "done", "--completed", "maybe"]
        )
        .status
        .code(),
        Some(2)
    );
}
