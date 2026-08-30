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

fn create(current: &std::path::Path, title: &str) -> String {
    let output = tbtm(
        current,
        &[
            "task", "create", "--title", title, "--type", "task", "--json",
        ],
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn dependency(
    current: &std::path::Path,
    operation: &str,
    task: &str,
    upstream: &str,
) -> std::process::Output {
    tbtm(
        current,
        &[
            "task",
            "dependency",
            operation,
            task,
            "--depends-on",
            upstream,
            "--json",
        ],
    )
}

#[test]
fn add_remove_and_view_preserve_direction_and_summary_order() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let downstream = create(temp.path(), "Build");
    let upstream_b = create(temp.path(), "Schema B");
    let upstream_a = create(temp.path(), "Schema A");

    for upstream in [&upstream_b, &upstream_a] {
        let output = dependency(temp.path(), "add", &downstream, upstream);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["data"],
            serde_json::json!({"taskId": downstream, "dependsOn": upstream})
        );
    }
    let detail: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &downstream, "--json"]).stdout)
            .unwrap();
    let ids: Vec<_> = detail["data"]["dependencies"]["upstream"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    let mut expected = vec![upstream_a.as_str(), upstream_b.as_str()];
    expected.sort_unstable();
    assert_eq!(ids, expected);
    assert_eq!(
        detail["data"]["dependencies"]["upstream"][0]["status"]["code"],
        "to_do"
    );
    let reverse: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &upstream_a, "--json"]).stdout)
            .unwrap();
    assert_eq!(
        reverse["data"]["dependencies"]["downstream"][0]["id"],
        downstream
    );

    assert!(
        dependency(temp.path(), "remove", &downstream, &upstream_a)
            .status
            .success()
    );
    let detail: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &downstream, "--json"]).stdout)
            .unwrap();
    assert_eq!(
        detail["data"]["dependencies"]["upstream"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn invalid_edges_return_stable_errors_without_metadata_writes() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let a = create(temp.path(), "A");
    let b = create(temp.path(), "B");
    let c = create(temp.path(), "C");
    assert!(dependency(temp.path(), "add", &a, &b).status.success());
    assert!(dependency(temp.path(), "add", &b, &c).status.success());
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &a, "--json"]).stdout).unwrap();
    for (output, code, exit) in [
        (dependency(temp.path(), "add", &a, &a), "SELF_DEPENDENCY", 2),
        (
            dependency(temp.path(), "add", &a, &b),
            "DEPENDENCY_EXISTS",
            2,
        ),
        (
            dependency(temp.path(), "add", &c, &a),
            "DEPENDENCY_CYCLE",
            2,
        ),
        (
            dependency(temp.path(), "remove", &a, &c),
            "DEPENDENCY_NOT_FOUND",
            3,
        ),
        (
            dependency(temp.path(), "add", "missing", &a),
            "TASK_NOT_FOUND",
            3,
        ),
    ] {
        assert_eq!(output.status.code(), Some(exit));
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["error"]["code"], code);
    }
    let after: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &a, "--json"]).stdout).unwrap();
    assert_eq!(after["data"]["updatedAt"], before["data"]["updatedAt"]);
    assert_eq!(
        after["data"]["dependencies"]["upstream"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn archived_downstream_rejects_but_archived_upstream_is_allowed() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let downstream = create(temp.path(), "Downstream");
    let upstream = create(temp.path(), "Upstream");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [&upstream],
        )
        .unwrap();
    assert!(
        dependency(temp.path(), "add", &downstream, &upstream)
            .status
            .success()
    );
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [&downstream],
        )
        .unwrap();
    let output = dependency(temp.path(), "remove", &downstream, &upstream);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
        "TASK_ARCHIVED"
    );
}

#[test]
fn view_rejects_pending_migration_until_mutation_upgrades() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let id = create(temp.path(), "Old schema task");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version >= 4", [])
        .unwrap();
    connection.execute("DROP TABLE task_hierarchy", []).unwrap();
    connection.execute("DROP TABLE task_claims", []).unwrap();
    connection
        .execute("DROP TABLE task_dependencies", [])
        .unwrap();
    connection
        .execute("ALTER TABLE tasks DROP COLUMN archive_reason", [])
        .unwrap();
    drop(connection);
    let view = tbtm(temp.path(), &["task", "view", &id, "--json"]);
    assert_eq!(view.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&view.stdout).unwrap()["error"]["code"],
        "DATABASE_UNAVAILABLE"
    );
    assert!(
        tbtm(
            temp.path(),
            &["task", "update", &id, "--clear-tags", "--json"]
        )
        .status
        .success()
    );
    assert!(
        tbtm(temp.path(), &["task", "view", &id, "--json"])
            .status
            .success()
    );
}

#[test]
fn human_output_is_concise() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let downstream = create(temp.path(), "Downstream");
    let upstream = create(temp.path(), "Upstream");
    let output = tbtm(
        temp.path(),
        &[
            "task",
            "dependency",
            "add",
            &downstream,
            "--depends-on",
            &upstream,
        ],
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Added dependency: {downstream} depends on {upstream}\n")
    );
}

#[test]
fn satisfaction_requires_every_upstream_completed_or_archived() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let downstream = create(temp.path(), "Downstream");
    let first = create(temp.path(), "First");
    let second = create(temp.path(), "Second");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    assert!(tbtm_core::task::dependencies_satisfied(&connection, &downstream).unwrap());
    drop(connection);
    assert!(
        dependency(temp.path(), "add", &downstream, &first)
            .status
            .success()
    );
    assert!(
        dependency(temp.path(), "add", &downstream, &second)
            .status
            .success()
    );
    let connection = Connection::open(&database).unwrap();
    assert!(!tbtm_core::task::dependencies_satisfied(&connection, &downstream).unwrap());
    connection.execute("UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1", [&first]).unwrap();
    assert!(!tbtm_core::task::dependencies_satisfied(&connection, &downstream).unwrap());
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [&second],
        )
        .unwrap();
    assert!(tbtm_core::task::dependencies_satisfied(&connection, &downstream).unwrap());
}

#[test]
fn competing_opposite_edges_cannot_form_a_cycle() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let a = create(temp.path(), "A");
    let b = create(temp.path(), "B");
    let root = temp.path().to_path_buf();
    let left = {
        let root = root.clone();
        let a = a.clone();
        let b = b.clone();
        std::thread::spawn(move || dependency(&root, "add", &a, &b))
    };
    let right = std::thread::spawn(move || dependency(&root, "add", &b, &a));
    let outputs = [left.join().unwrap(), right.join().unwrap()];
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        1
    );
    let failure = outputs
        .iter()
        .find(|output| !output.status.success())
        .unwrap();
    assert_eq!(failure.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&failure.stdout).unwrap()["error"]["code"],
        "DEPENDENCY_CYCLE"
    );
}
