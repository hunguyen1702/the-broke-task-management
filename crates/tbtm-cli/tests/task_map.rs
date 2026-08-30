use rusqlite::{Connection, params};
use serde_json::Value;
use std::{fs, path::Path, process::Command};
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
        tbtm(current, &["init", "--prefix", "project"])
            .status
            .success()
    );
}

fn create(current: &Path, title: &str, task_type: &str) -> String {
    let output = tbtm(
        current,
        &[
            "task", "create", "--title", title, "--type", task_type, "--json",
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
}

fn dependency(current: &Path, downstream: &str, upstream: &str) {
    let output = tbtm(
        current,
        &[
            "task",
            "dependency",
            "add",
            downstream,
            "--depends-on",
            upstream,
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn parent(current: &Path, child: &str, parent: &str) {
    let output = tbtm(
        current,
        &["task", "parent", "set", child, "--parent", parent],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn map(current: &Path, root: &str, direction: Option<&str>) -> Value {
    let mut arguments = vec!["task", "map", root, "--json"];
    if let Some(direction) = direction {
        arguments.extend(["--direction", direction]);
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
fn empty_missing_and_invalid_maps_have_stable_contracts() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root", "task");

    let data = map(temp.path(), &root, None);
    assert_eq!(data["rootTaskId"], root);
    assert_eq!(data["direction"], "all");
    assert_eq!(data["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(data["nodes"][0]["reachedBy"], serde_json::json!([]));
    assert_eq!(data["edges"], serde_json::json!([]));
    let human = tbtm(temp.path(), &["task", "map", &root]);
    assert_eq!(
        String::from_utf8(human.stdout).unwrap(),
        format!(
            "Relationship map for {root} (all)\nTarget: {root} [task, to_do] [ready] Root\n\nNo relationships found.\n"
        )
    );

    let missing = tbtm(temp.path(), &["task", "map", "missing", "--json"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["error"]["code"],
        "TASK_NOT_FOUND"
    );
    let invalid = tbtm(
        temp.path(),
        &["task", "map", &root, "--direction", "sideways"],
    );
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
}

#[test]
fn all_deduplicates_nodes_retains_edges_and_orders_shortest_reaches() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let epic = create(temp.path(), "Epic", "epic");
    let story = create(temp.path(), "Story", "story");
    let root = create(temp.path(), "Root", "task");
    let left = create(temp.path(), "Left", "task");
    let right = create(temp.path(), "Right", "task");
    let shared = create(temp.path(), "Shared", "task");
    parent(temp.path(), &story, &epic);
    parent(temp.path(), &root, &story);
    dependency(temp.path(), &left, &root);
    dependency(temp.path(), &right, &root);
    dependency(temp.path(), &shared, &left);
    dependency(temp.path(), &shared, &right);

    let data = map(temp.path(), &root, None);
    let ids: Vec<_> = data["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids[0], root);
    assert_eq!(ids.len(), 6);
    assert_eq!(data["edges"].as_array().unwrap().len(), 6);
    let shared_node = data["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == shared)
        .unwrap();
    assert_eq!(
        shared_node["reachedBy"],
        serde_json::json!([{"direction": "downstream", "depth": 2}])
    );
    assert_eq!(shared_node["blocked"], true);

    let parents = map(temp.path(), &root, Some("parent"));
    assert_eq!(parents["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(parents["nodes"][1]["reachedBy"][0]["depth"], 1);
    assert_eq!(parents["nodes"][2]["reachedBy"][0]["depth"], 2);
    let upstream = map(temp.path(), &shared, Some("upstream"));
    assert_eq!(upstream["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(upstream["edges"].as_array().unwrap().len(), 4);
}

#[test]
fn human_diamond_uses_breadth_first_tree_and_one_reference() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root", "task");
    let left = create(temp.path(), "Left", "task");
    let right = create(temp.path(), "Right", "task");
    let shared = create(temp.path(), "Shared", "task");
    dependency(temp.path(), &left, &root);
    dependency(temp.path(), &right, &root);
    dependency(temp.path(), &shared, &left);
    dependency(temp.path(), &shared, &right);

    let output = tbtm(
        temp.path(),
        &["task", "map", &root, "--direction", "downstream"],
    );
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).unwrap();
    assert!(human.starts_with(&format!(
        "Relationship map for {root} (downstream)\nTarget: {root} [task, to_do] [ready] Root\n\nDownstream dependencies:\n"
    )));
    assert_eq!(human.matches(&format!("{shared} [task, to_do]")).count(), 1);
    assert_eq!(
        human
            .matches(&format!("↩ {shared} (already shown)"))
            .count(),
        1
    );
}

#[test]
fn traversal_survives_completed_archived_nodes_and_corrupt_cycles() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root", "task");
    let middle = create(temp.path(), "Middle", "task");
    let leaf = create(temp.path(), "Leaf", "task");
    dependency(temp.path(), &middle, &root);
    dependency(temp.path(), &leaf, &middle);
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'old' WHERE id = ?1",
            [&middle],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1",
            [&leaf],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
            params![root, leaf],
        )
        .unwrap();
    drop(connection);

    let data = map(temp.path(), &root, Some("downstream"));
    assert_eq!(data["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(data["edges"].as_array().unwrap().len(), 3);
    let middle_node = data["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == middle)
        .unwrap();
    assert_eq!(middle_node["archived"], true);
    let leaf_node = data["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == leaf)
        .unwrap();
    assert_eq!(leaf_node["status"]["completed"], true);
}

#[test]
fn maps_are_migration_free_read_only_and_linked_worktree_consistent() {
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
                linked.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success()
    );
    initialize(&main);
    let root = create(&main, "Root", "task");
    let child = create(&main, "Child", "task");
    dependency(&main, &child, &root);
    let database = main.join(".tbtm/tbtm.db");
    let before = fs::read(&database).unwrap();
    assert_eq!(map(&main, &root, None), map(&linked, &root, None));
    assert_eq!(fs::read(&database).unwrap(), before);

    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 7", [])
        .unwrap();
    drop(connection);
    let pending = tbtm(&linked, &["task", "map", &root, "--json"]);
    assert!(!pending.status.success());
    let connection = Connection::open(database).unwrap();
    let version_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 7)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!version_exists);
}
