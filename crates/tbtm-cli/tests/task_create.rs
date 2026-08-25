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

#[test]
fn minimum_create_returns_defaults_and_persists_one_task() {
    let temp = tempdir().unwrap();
    initialize(temp.path());

    let output = tbtm(
        temp.path(),
        &[
            "task",
            "create",
            "--title",
            "  Implement parser  ",
            "--type",
            "task",
            "--json",
        ],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let task = &response["data"];
    assert_eq!(task["title"], "Implement parser");
    assert_eq!(task["type"], "task");
    assert_eq!(task["status"]["code"], "to_do");
    assert_eq!(task["priority"], 50);
    assert_eq!(task["createdBy"], "user");
    assert_eq!(task["createdAt"], task["updatedAt"]);
    assert_eq!(task["tags"], serde_json::json!([]));
    assert!(task["estimate"].is_null());
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn full_create_preserves_structured_context_and_agent_actor() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let registered = tbtm(temp.path(), &["agent", "register", "worker", "--json"]);
    let agent: Value = serde_json::from_slice(&registered.stdout).unwrap();
    let agent_id = agent["data"]["id"].as_str().unwrap();

    let output = tbtm(
        temp.path(),
        &[
            "task",
            "create",
            "--title",
            "Fix it",
            "--type",
            "bug",
            "--description",
            "Details",
            "--goal",
            "Green CI",
            "--acceptance-criteria",
            "Tests pass",
            "--status",
            "in_progress",
            "--priority",
            "0",
            "--estimate",
            "1.5",
            "--tag",
            "rust",
            "--tag",
            "cli",
            "--url",
            "https://example.com/issue/1",
            "--code-ref",
            "src/main.rs:10-20::entry",
            "--agent",
            agent_id,
            "--json",
        ],
    );

    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let task = &response["data"];
    assert_eq!(task["createdBy"], agent_id);
    assert_eq!(task["status"]["name"], "In progress");
    assert_eq!(task["tags"], serde_json::json!(["rust", "cli"]));
    assert_eq!(task["codeReferences"][0]["startLine"], 10);
    assert_eq!(task["codeReferences"][0]["endLine"], 20);
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    let counts: (i64, i64, i64) = connection.query_row(
        "SELECT (SELECT COUNT(*) FROM task_tags), (SELECT COUNT(*) FROM task_external_urls), (SELECT COUNT(*) FROM task_code_references)",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap();
    assert_eq!(counts, (2, 1, 1));
}

#[test]
fn invalid_context_and_unknown_references_do_not_mutate() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    for arguments in [
        vec![
            "task", "create", "--title", "x", "--type", "task", "--tag", "dup", "--tag", " dup ",
            "--json",
        ],
        vec![
            "task",
            "create",
            "--title",
            "x",
            "--type",
            "task",
            "--url",
            "file:///tmp/x",
            "--json",
        ],
        vec![
            "task",
            "create",
            "--title",
            "x",
            "--type",
            "task",
            "--code-ref",
            "../secret:1",
            "--json",
        ],
        vec![
            "task", "create", "--title", "x", "--type", "task", "--status", "Todo", "--json",
        ],
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert!(
            matches!(output.status.code(), Some(2 | 3)),
            "stdout={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn human_create_shows_contract_fields() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let output = tbtm(
        temp.path(),
        &[
            "task",
            "create",
            "--title",
            "Write tests",
            "--type",
            "testing",
        ],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for field in [
        "Task ID: project-testing-",
        "Title: Write tests",
        "Type: testing",
        "Status: Todo",
        "Priority: 50",
        "Actor: user",
    ] {
        assert!(stdout.contains(field), "missing {field:?} in {stdout:?}");
    }
}

#[test]
fn concurrent_creates_have_unique_ids_and_keep_database_usable() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = temp.path().to_path_buf();
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let root = root.clone();
            std::thread::spawn(move || {
                let title = format!("Task {index}");
                let output = tbtm(
                    &root,
                    &[
                        "task", "create", "--title", &title, "--type", "task", "--json",
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
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 8);
    let connection = Connection::open(root.join(".tbtm/tbtm.db")).unwrap();
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}
