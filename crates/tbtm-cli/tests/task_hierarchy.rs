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

fn create(current: &std::path::Path, title: &str, task_type: &str) -> String {
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

fn set(current: &std::path::Path, child: &str, parent: &str) -> std::process::Output {
    tbtm(
        current,
        &["task", "parent", "set", child, "--parent", parent, "--json"],
    )
}

#[test]
fn set_replace_remove_hydrates_and_traverses() {
    let temp = tempdir().unwrap();
    assert!(
        tbtm(temp.path(), &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
    let epic = create(temp.path(), "Epic", "epic");
    let other_epic = create(temp.path(), "Other", "epic");
    let story = create(temp.path(), "Story", "story");
    let leaf = create(temp.path(), "Leaf", "task");
    assert!(set(temp.path(), &story, &epic).status.success());
    assert!(set(temp.path(), &leaf, &story).status.success());

    let hierarchy: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &["task", "hierarchy", &epic, "--recursive", "--json"],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(hierarchy["data"]["children"][0]["id"], story);
    assert_eq!(hierarchy["data"]["descendants"][0]["depth"], 1);
    assert_eq!(hierarchy["data"]["descendants"][1]["depth"], 2);
    let detail: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &leaf, "--json"]).stdout)
            .unwrap();
    assert_eq!(detail["data"]["hierarchy"]["parent"]["id"], story);

    assert!(set(temp.path(), &story, &other_epic).status.success());
    let removed = tbtm(temp.path(), &["task", "parent", "remove", &story, "--json"]);
    assert!(removed.status.success());
    assert!(
        serde_json::from_slice::<Value>(&removed.stdout).unwrap()["data"]["parentId"].is_null()
    );
}

#[test]
fn invalid_types_self_and_cycles_have_stable_errors() {
    let temp = tempdir().unwrap();
    assert!(
        tbtm(temp.path(), &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
    let epic = create(temp.path(), "Epic", "epic");
    let story = create(temp.path(), "Story", "story");
    let leaf = create(temp.path(), "Leaf", "task");
    assert!(set(temp.path(), &story, &epic).status.success());
    assert!(set(temp.path(), &leaf, &story).status.success());
    for (output, code) in [
        (set(temp.path(), &epic, &epic), "SELF_PARENT"),
        (set(temp.path(), &story, &leaf), "INVALID_TASK_HIERARCHY"),
    ] {
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
            code
        );
    }
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    connection
        .execute(
            "DELETE FROM task_hierarchy WHERE child_task_id = ?1",
            [&story],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO task_hierarchy (child_task_id, parent_task_id) VALUES (?1, ?2)",
            [&story, &leaf],
        )
        .unwrap();
    let cycle = set(temp.path(), &leaf, &story);
    assert_eq!(cycle.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&cycle.stdout).unwrap()["error"]["code"],
        "HIERARCHY_CYCLE"
    );
}

#[test]
fn same_parent_is_metadata_preserving_and_missing_remove_is_not_found() {
    let temp = tempdir().unwrap();
    assert!(
        tbtm(temp.path(), &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
    let epic = create(temp.path(), "Epic", "epic");
    let story = create(temp.path(), "Story", "story");
    assert!(set(temp.path(), &story, &epic).status.success());
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &story, "--json"]).stdout)
            .unwrap();
    assert!(set(temp.path(), &story, &epic).status.success());
    let after: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &story, "--json"]).stdout)
            .unwrap();
    assert_eq!(before["data"]["updatedAt"], after["data"]["updatedAt"]);
    assert!(
        tbtm(temp.path(), &["task", "parent", "remove", &story, "--json"])
            .status
            .success()
    );
    let missing = tbtm(temp.path(), &["task", "parent", "remove", &story, "--json"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["error"]["code"],
        "PARENT_NOT_FOUND"
    );
}
