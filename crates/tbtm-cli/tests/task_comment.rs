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
    assert!(
        tbtm(current, &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
}

fn linked_worktree() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let main = temp.path().join("main");
    let linked = temp.path().join("linked");
    fs::create_dir(&main).unwrap();
    for arguments in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Test"],
        vec!["commit", "--allow-empty", "--quiet", "-m", "initial"],
    ] {
        assert!(
            Command::new("git")
                .current_dir(&main)
                .args(arguments)
                .status()
                .unwrap()
                .success()
        );
    }
    assert!(
        Command::new("git")
            .current_dir(&main)
            .args([
                "worktree",
                "add",
                "--quiet",
                "-b",
                "linked",
                linked.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    (temp, main, linked)
}

fn create_task(current: &std::path::Path) -> String {
    let output = tbtm(
        current,
        &[
            "task",
            "create",
            "--title",
            "Comment target",
            "--type",
            "task",
            "--json",
        ],
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn register_agent(current: &std::path::Path, name: &str) -> String {
    let output = tbtm(current, &["agent", "register", name, "--json"]);
    assert!(output.status.success());
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn add_comment(
    current: &std::path::Path,
    task_id: &str,
    content: &str,
    agent_id: Option<&str>,
) -> Value {
    let mut arguments = vec![
        "task",
        "comment",
        "add",
        task_id,
        "--content",
        content,
        "--json",
    ];
    if let Some(agent_id) = agent_id {
        arguments.extend(["--agent", agent_id]);
    }
    let output = tbtm(current, &arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

#[test]
fn user_comment_preserves_content_and_task_metadata() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let before = tbtm(temp.path(), &["task", "view", &task_id, "--json"]);
    let before: Value = serde_json::from_slice(&before.stdout).unwrap();
    let content = "  **Decision:** keep bytes.\n";

    let output = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "add",
            &task_id,
            "--content",
            content,
            "--json",
        ],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["data"]["taskId"], task_id);
    assert_eq!(response["data"]["content"], content);
    assert_eq!(response["data"]["author"], "user");
    assert!(uuid::Uuid::parse_str(response["data"]["id"].as_str().unwrap()).is_ok());
    assert!(response["data"].get("authorDisplayName").is_none());

    let after = tbtm(temp.path(), &["task", "view", &task_id, "--json"]);
    let after: Value = serde_json::from_slice(&after.stdout).unwrap();
    assert_eq!(before["data"], after["data"]);
    assert!(after["data"].get("comments").is_none());
}

#[test]
fn agent_and_archived_comments_list_chronologically() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let registered = tbtm(temp.path(), &["agent", "register", "worker", "--json"]);
    let agent: Value = serde_json::from_slice(&registered.stdout).unwrap();
    let agent_id = agent["data"]["id"].as_str().unwrap();
    let display_name = agent["data"]["displayName"].as_str().unwrap();
    assert!(
        tbtm(
            temp.path(),
            &["task", "archive", &task_id, "--reason", "Done", "--json"]
        )
        .status
        .success()
    );
    let archived_before = tbtm(temp.path(), &["task", "view", &task_id, "--json"]);
    let archived_before: Value = serde_json::from_slice(&archived_before.stdout).unwrap();

    let first = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "add",
            &task_id,
            "--content",
            "user note",
            "--json",
        ],
    );
    assert!(first.status.success());
    let second = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "add",
            &task_id,
            "--content",
            "agent note",
            "--agent",
            agent_id,
            "--json",
        ],
    );
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let list = tbtm(
        temp.path(),
        &["task", "comment", "list", &task_id, "--json"],
    );
    assert!(list.status.success());
    let response: Value = serde_json::from_slice(&list.stdout).unwrap();
    let comments = response["data"].as_array().unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0]["content"], "user note");
    assert_eq!(comments[1]["author"], agent_id);
    let archived_after = tbtm(temp.path(), &["task", "view", &task_id, "--json"]);
    let archived_after: Value = serde_json::from_slice(&archived_after.stdout).unwrap();
    assert_eq!(archived_before["data"], archived_after["data"]);

    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE task_comments SET created_at = '2000-01-01T00:00:00Z' WHERE task_id = ?1",
            [&task_id],
        )
        .unwrap();
    drop(connection);
    let tied = tbtm(
        temp.path(),
        &["task", "comment", "list", &task_id, "--json"],
    );
    let tied: Value = serde_json::from_slice(&tied.stdout).unwrap();
    let tied = tied["data"].as_array().unwrap();
    assert!(tied[0]["id"].as_str().unwrap() < tied[1]["id"].as_str().unwrap());

    let human = tbtm(temp.path(), &["task", "comment", "list", &task_id]);
    let stdout = String::from_utf8(human.stdout).unwrap();
    assert!(stdout.contains("Author: user"));
    assert!(stdout.contains(&format!("Author: {display_name} ({agent_id})")));
}

#[test]
fn concurrent_additions_are_distinct_and_durable() {
    let (_temp, main, linked) = linked_worktree();
    initialize(&main);
    let task_id = create_task(&main);
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let root = if index % 2 == 0 {
                main.clone()
            } else {
                linked.clone()
            };
            let task_id = task_id.clone();
            std::thread::spawn(move || {
                let content = format!("concurrent-{index}");
                let output = tbtm(
                    &root,
                    &[
                        "task",
                        "comment",
                        "add",
                        &task_id,
                        "--content",
                        &content,
                        "--json",
                    ],
                );
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
        })
        .collect();
    let mut ids: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8);
    let listed = tbtm(&linked, &["task", "comment", "list", &task_id, "--json"]);
    let listed: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed["data"].as_array().unwrap().len(), 8);
    let connection = Connection::open(main.join(".tbtm/tbtm.db")).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn invalid_comment_references_do_not_insert_rows() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let unknown_agent = uuid::Uuid::new_v4().to_string();
    for (arguments, exit, code) in [
        (
            vec![
                "task",
                "comment",
                "add",
                &task_id,
                "--content",
                "  ",
                "--json",
            ],
            2,
            "INVALID_COMMENT_CONTENT",
        ),
        (
            vec![
                "task",
                "comment",
                "add",
                &task_id,
                "--content",
                "note",
                "--agent",
                &unknown_agent,
                "--json",
            ],
            3,
            "AGENT_NOT_FOUND",
        ),
        (
            vec![
                "task",
                "comment",
                "add",
                "missing",
                "--content",
                "note",
                "--json",
            ],
            3,
            "TASK_NOT_FOUND",
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit));
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["error"]["code"], code);
    }
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM task_comments", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn empty_list_succeeds_and_does_not_apply_pending_migration() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 8", [])
        .unwrap();
    drop(connection);

    let pending = tbtm(
        temp.path(),
        &["task", "comment", "list", &task_id, "--json"],
    );
    assert_eq!(pending.status.code(), Some(1));
    let connection = Connection::open(&database).unwrap();
    let applied: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = 8",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 0);
    connection
        .execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (8, 'test')",
            [],
        )
        .unwrap();
    drop(connection);

    let before = std::fs::read(&database).unwrap();
    let json = tbtm(
        temp.path(),
        &["task", "comment", "list", &task_id, "--json"],
    );
    assert!(json.status.success());
    let response: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(response["data"], serde_json::json!([]));
    assert_eq!(std::fs::read(&database).unwrap(), before);
    let human = tbtm(temp.path(), &["task", "comment", "list", &task_id]);
    assert_eq!(
        String::from_utf8(human.stdout).unwrap(),
        "No comments found.\n"
    );
}

#[test]
fn add_upgrades_v7_and_comment_constraints_preserve_existing_data() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let config = temp.path().join(".tbtm/config.json");
    let config_before = std::fs::read(&config).unwrap();
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection.execute("DROP TABLE task_comments", []).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version >= 8", [])
        .unwrap();
    drop(connection);

    let added = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "add",
            &task_id,
            "--content",
            "migrated",
            "--json",
        ],
    );
    assert!(added.status.success());
    assert_eq!(std::fs::read(config).unwrap(), config_before);
    let connection = Connection::open(database).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = ON")
        .unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE id = ?1",
                [&task_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(connection.execute("INSERT INTO task_comments (id, task_id, content, author_actor_type, author_agent_id, created_at) VALUES ('bad-actor', ?1, 'x', 'user', 'dangling', 'now')", [&task_id]).is_err());
    assert!(connection.execute("INSERT INTO task_comments (id, task_id, content, author_actor_type, author_agent_id, created_at) VALUES ('bad-task', 'missing', 'x', 'user', NULL, 'now')", []).is_err());
    assert!(connection.execute("INSERT INTO task_comments (id, task_id, content, author_actor_type, author_agent_id, created_at) VALUES ('blank', ?1, '  ', 'user', NULL, 'now')", [&task_id]).is_err());
}

#[test]
fn user_deletes_any_comment_from_archived_task_without_changing_task_metadata() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let agent_id = register_agent(temp.path(), "worker");
    let comment = add_comment(temp.path(), &task_id, "remove me", Some(&agent_id));
    assert!(
        tbtm(
            temp.path(),
            &["task", "archive", &task_id, "--reason", "Done", "--json"]
        )
        .status
        .success()
    );
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &task_id, "--json"]).stdout)
            .unwrap();

    let deleted = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "delete",
            &task_id,
            comment["id"].as_str().unwrap(),
            "--json",
        ],
    );
    assert!(deleted.status.success());
    let response: Value = serde_json::from_slice(&deleted.stdout).unwrap();
    assert_eq!(response["data"], comment);
    let listed: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &["task", "comment", "list", &task_id, "--json"],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(listed["data"], serde_json::json!([]));
    let after: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &task_id, "--json"]).stdout)
            .unwrap();
    assert_eq!(before["data"], after["data"]);
}

#[test]
fn agent_can_delete_only_its_own_comment() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let owner = register_agent(temp.path(), "owner");
    let foreign = register_agent(temp.path(), "foreign");
    let user_comment = add_comment(temp.path(), &task_id, "user note", None);
    let owned_comment = add_comment(temp.path(), &task_id, "owned note", Some(&owner));

    for comment in [&user_comment, &owned_comment] {
        let denied = tbtm(
            temp.path(),
            &[
                "task",
                "comment",
                "delete",
                &task_id,
                comment["id"].as_str().unwrap(),
                "--agent",
                &foreign,
                "--json",
            ],
        );
        assert_eq!(denied.status.code(), Some(5));
        let response: Value = serde_json::from_slice(&denied.stdout).unwrap();
        assert_eq!(response["error"]["code"], "COMMENT_DELETE_FORBIDDEN");
        assert_eq!(
            response["error"]["details"],
            serde_json::json!({
                "taskId": task_id,
                "commentId": comment["id"],
                "author": comment["author"]
            })
        );
    }

    let deleted = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "delete",
            &task_id,
            owned_comment["id"].as_str().unwrap(),
            "--agent",
            &owner,
        ],
    );
    assert!(deleted.status.success());
    assert_eq!(
        String::from_utf8(deleted.stdout).unwrap(),
        format!(
            "Deleted comment {} from task {}.\n",
            owned_comment["id"].as_str().unwrap(),
            task_id
        )
    );
    let listed: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &["task", "comment", "list", &task_id, "--json"],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(listed["data"][0]["id"], user_comment["id"]);
}

#[test]
fn delete_validates_actor_task_and_scoped_comment_in_order() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let first_task = create_task(temp.path());
    let second_task = create_task(temp.path());
    let comment = add_comment(temp.path(), &first_task, "keep", None);
    let missing_agent = uuid::Uuid::new_v4().to_string();
    let missing_comment = uuid::Uuid::new_v4().to_string();

    for (arguments, code) in [
        (
            vec![
                "task",
                "comment",
                "delete",
                "missing",
                &missing_comment,
                "--agent",
                &missing_agent,
                "--json",
            ],
            "AGENT_NOT_FOUND",
        ),
        (
            vec![
                "task",
                "comment",
                "delete",
                "missing",
                &missing_comment,
                "--json",
            ],
            "TASK_NOT_FOUND",
        ),
        (
            vec![
                "task",
                "comment",
                "delete",
                &second_task,
                comment["id"].as_str().unwrap(),
                "--json",
            ],
            "COMMENT_NOT_FOUND",
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(3));
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["error"]["code"], code);
        match code {
            "AGENT_NOT_FOUND" | "TASK_NOT_FOUND" => {
                assert_eq!(response["error"]["details"], serde_json::json!({}));
            }
            "COMMENT_NOT_FOUND" => assert_eq!(
                response["error"]["details"],
                serde_json::json!({"taskId": second_task, "commentId": comment["id"]})
            ),
            _ => unreachable!(),
        }
    }
    let listed: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &["task", "comment", "list", &first_task, "--json"],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    for arguments in [
        vec![
            "task",
            "comment",
            "delete",
            &first_task,
            "not-a-uuid",
            "--json",
        ],
        vec![
            "task",
            "comment",
            "delete",
            &first_task,
            &missing_comment,
            "--agent",
            "not-a-uuid",
            "--json",
        ],
    ] {
        let malformed = tbtm(temp.path(), &arguments);
        assert_eq!(malformed.status.code(), Some(2));
        assert!(malformed.stderr.is_empty());
        let response: Value = serde_json::from_slice(&malformed.stdout).unwrap();
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "INVALID_ARGUMENTS");
    }
}

#[test]
fn concurrent_linked_worktree_deletes_have_one_winner() {
    let (_temp, main, linked) = linked_worktree();
    initialize(&main);
    let task_id = create_task(&main);
    let comment = add_comment(&main, &task_id, "race", None);
    let comment_id = comment["id"].as_str().unwrap().to_owned();
    let handles: Vec<_> = [main.clone(), linked]
        .into_iter()
        .map(|root| {
            let task_id = task_id.clone();
            let comment_id = comment_id.clone();
            std::thread::spawn(move || {
                tbtm(
                    &root,
                    &["task", "comment", "delete", &task_id, &comment_id, "--json"],
                )
            })
        })
        .collect();
    let outputs: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        1
    );
    let loser = outputs
        .iter()
        .find(|output| !output.status.success())
        .unwrap();
    assert_eq!(loser.status.code(), Some(3));
    let response: Value = serde_json::from_slice(&loser.stdout).unwrap();
    assert_eq!(response["error"]["code"], "COMMENT_NOT_FOUND");
}

#[test]
fn delete_failure_rolls_back_and_help_describes_actor_selection() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let comment = add_comment(temp.path(), &task_id, "survives", None);
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_comment_delete BEFORE DELETE ON task_comments BEGIN SELECT RAISE(ABORT, 'injected delete failure'); END;",
        )
        .unwrap();
    drop(connection);

    let failed = tbtm(
        temp.path(),
        &[
            "task",
            "comment",
            "delete",
            &task_id,
            comment["id"].as_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(failed.status.code(), Some(1));
    let response: Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(response["error"]["code"], "DATABASE_UNAVAILABLE");
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM task_comments WHERE id = ?1",
                [comment["id"].as_str().unwrap()],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );

    let help = tbtm(temp.path(), &["task", "comment", "delete", "--help"]);
    let stdout = String::from_utf8(help.stdout).unwrap();
    assert!(stdout.contains("omission selects logical-user authority"));
    assert!(!stdout.to_lowercase().contains("authenticate"));
}

#[test]
fn comment_command_surface_has_no_edit_operation() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let comment = add_comment(temp.path(), &task_id, "immutable", None);

    let help = tbtm(temp.path(), &["task", "comment", "--help"]);
    assert!(help.status.success());
    let stdout = String::from_utf8(help.stdout).unwrap();
    for command in ["add", "list", "delete"] {
        assert!(
            stdout
                .lines()
                .any(|line| line.trim_start().starts_with(command))
        );
    }
    for command in ["edit", "update", "amend", "replace"] {
        assert!(
            !stdout
                .lines()
                .any(|line| line.trim_start().starts_with(command))
        );
        let rejected = tbtm(
            temp.path(),
            &[
                "task",
                "comment",
                command,
                &task_id,
                comment["id"].as_str().unwrap(),
            ],
        );
        assert_eq!(rejected.status.code(), Some(2));
    }
}

#[test]
fn supported_operations_leave_existing_comment_fields_unchanged() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task_id = create_task(temp.path());
    let retained = add_comment(temp.path(), &task_id, "retain every field", None);
    let disposable = add_comment(temp.path(), &task_id, "later comment", None);

    for _ in 0..2 {
        let listed: Value = serde_json::from_slice(
            &tbtm(
                temp.path(),
                &["task", "comment", "list", &task_id, "--json"],
            )
            .stdout,
        )
        .unwrap();
        assert_eq!(listed["data"][0], retained);
    }
    assert!(
        tbtm(
            temp.path(),
            &["task", "update", &task_id, "--title", "Renamed", "--json"]
        )
        .status
        .success()
    );
    assert!(
        tbtm(
            temp.path(),
            &[
                "task",
                "comment",
                "delete",
                &task_id,
                disposable["id"].as_str().unwrap(),
                "--json",
            ]
        )
        .status
        .success()
    );
    let listed: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &["task", "comment", "list", &task_id, "--json"],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(listed["data"], serde_json::json!([retained]));
}

#[test]
fn correction_is_non_atomic_and_uses_a_new_identity_on_active_and_archived_tasks() {
    for archived in [false, true] {
        let temp = tempdir().unwrap();
        initialize(temp.path());
        let task_id = create_task(temp.path());
        let owner = register_agent(temp.path(), "owner");
        if archived {
            assert!(
                tbtm(
                    temp.path(),
                    &["task", "archive", &task_id, "--reason", "Done", "--json"]
                )
                .status
                .success()
            );
        }
        let original = add_comment(temp.path(), &task_id, "typo", Some(&owner));
        let task_before: Value = serde_json::from_slice(
            &tbtm(temp.path(), &["task", "view", &task_id, "--json"]).stdout,
        )
        .unwrap();

        let deleted = tbtm(
            temp.path(),
            &[
                "task",
                "comment",
                "delete",
                &task_id,
                original["id"].as_str().unwrap(),
                "--json",
            ],
        );
        assert!(deleted.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&deleted.stdout).unwrap()["data"],
            original
        );

        let failed_add = tbtm(
            temp.path(),
            &[
                "task",
                "comment",
                "add",
                &task_id,
                "--content",
                "  ",
                "--json",
            ],
        );
        assert_eq!(failed_add.status.code(), Some(2));
        let empty: Value = serde_json::from_slice(
            &tbtm(
                temp.path(),
                &["task", "comment", "list", &task_id, "--json"],
            )
            .stdout,
        )
        .unwrap();
        assert_eq!(empty["data"], serde_json::json!([]));

        let corrected = add_comment(temp.path(), &task_id, "corrected", None);
        assert_ne!(corrected["id"], original["id"]);
        assert_eq!(corrected["author"], "user");
        assert!(
            time::OffsetDateTime::parse(
                corrected["createdAt"].as_str().unwrap(),
                &time::format_description::well_known::Rfc3339,
            )
            .is_ok()
        );
        let task_after: Value = serde_json::from_slice(
            &tbtm(temp.path(), &["task", "view", &task_id, "--json"]).stdout,
        )
        .unwrap();
        assert_eq!(task_after["data"], task_before["data"]);
    }
}

#[test]
fn linked_worktree_add_list_delete_share_the_canonical_store() {
    let (_temp, main, linked) = linked_worktree();
    initialize(&main);
    let task_id = create_task(&main);
    let retained = add_comment(&main, &task_id, "stable", None);
    let added = add_comment(&linked, &task_id, "from linked worktree", None);

    let listed: Value = serde_json::from_slice(
        &tbtm(&main, &["task", "comment", "list", &task_id, "--json"]).stdout,
    )
    .unwrap();
    let comments = listed["data"].as_array().unwrap();
    assert_eq!(comments.len(), 2);
    assert!(comments.contains(&retained));
    assert!(comments.contains(&added));

    let deleted = tbtm(
        &linked,
        &[
            "task",
            "comment",
            "delete",
            &task_id,
            added["id"].as_str().unwrap(),
            "--json",
        ],
    );
    assert!(deleted.status.success());
    let listed: Value = serde_json::from_slice(
        &tbtm(&main, &["task", "comment", "list", &task_id, "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(listed["data"], serde_json::json!([retained]));
    assert!(!linked.join(".tbtm").exists());
}
