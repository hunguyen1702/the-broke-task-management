use serde_json::Value;
use std::{path::Path, process::Command};
use tempfile::tempdir;

fn tbtm(current: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn successful_text(current: &Path, arguments: &[&str]) -> String {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "args={arguments:?} stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "args={arguments:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn assert_contains_all(text: &str, expected: &[&str], context: &[&str]) {
    for value in expected {
        assert!(
            text.contains(value),
            "missing {value:?} for {context:?} in:\n{text}"
        );
    }
}

#[test]
fn comment_inventory_arguments_and_correction_are_discoverable() {
    let current = tempdir().unwrap();
    for (arguments, expected) in [
        (vec!["--help"], vec!["task", "--json"]),
        (vec!["task", "--help"], vec!["comment", "immutable"]),
        (
            vec!["task", "comment", "--help"],
            vec![
                "add",
                "list",
                "delete",
                "immutable",
                "delete it and then add",
                "two independent operations",
                "--json",
            ],
        ),
        (
            vec!["task", "comment", "add", "--help"],
            vec![
                "<TASK_ID>",
                "Task receiving the comment",
                "--content <CONTENT>",
                "Markdown comment content",
                "--agent <AGENT>",
                "logical user",
                "--json",
            ],
        ),
        (
            vec!["task", "comment", "list", "--help"],
            vec!["<TASK_ID>", "Task whose comments to list", "--json"],
        ),
        (
            vec!["task", "comment", "delete", "--help"],
            vec![
                "<TASK_ID>",
                "Task owning the comment",
                "<COMMENT_ID>",
                "Comment UUID to delete",
                "--agent <AGENT>",
                "logical-user authority",
                "--json",
            ],
        ),
    ] {
        let help = successful_text(current.path(), &arguments);
        assert_contains_all(&help, &expected, &arguments);
    }
}

#[test]
fn unsupported_comment_surfaces_are_rejected_before_repository_access() {
    let current = tempdir().unwrap();
    let uuid = "00000000-0000-0000-0000-000000000000";
    for arguments in [
        vec!["comment", "list", "TASK-1"],
        vec!["task", "comment", "edit", "TASK-1", uuid],
        vec!["task", "comment", "update", "TASK-1", uuid],
        vec!["task", "comment", "amend", "TASK-1", uuid],
        vec!["task", "comment", "replace", "TASK-1", uuid],
        vec!["task", "comment", "move", "TASK-1", uuid],
        vec!["task", "comment", "add", "TASK-1", "--file", "note.md"],
        vec!["task", "comment", "add", "TASK-1", "--stdin"],
        vec!["task", "comment", "list", "TASK-1", "--agent", uuid],
        vec!["task", "comment", "delete", "TASK-1", uuid, "--yes"],
    ] {
        let output = tbtm(current.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "args={arguments:?}");
        assert!(output.stdout.is_empty(), "args={arguments:?}");
    }
    assert!(!current.path().join(".tbtm").exists());
}

#[test]
fn all_comment_leaves_use_global_repeatable_json_transport() {
    let current = tempdir().unwrap();
    let initialized: Value = serde_json::from_str(&successful_text(
        current.path(),
        &["--json", "init", "--prefix", "comment-surface"],
    ))
    .unwrap();
    assert_eq!(initialized["ok"], true);
    let created: Value = serde_json::from_str(&successful_text(
        current.path(),
        &[
            "task", "create", "--title", "Comments", "--type", "task", "--json",
        ],
    ))
    .unwrap();
    let task_id = created["data"]["id"].as_str().unwrap();

    let added: Value = serde_json::from_str(&successful_text(
        current.path(),
        &[
            "--json",
            "task",
            "comment",
            "add",
            task_id,
            "--content",
            "note",
            "--json",
        ],
    ))
    .unwrap();
    assert_eq!(added["ok"], true);
    assert!(added["error"].is_null());
    let comment_id = added["data"]["id"].as_str().unwrap();

    let listed: Value = serde_json::from_str(&successful_text(
        current.path(),
        &["task", "--json", "comment", "list", task_id, "--json"],
    ))
    .unwrap();
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    let deleted: Value = serde_json::from_str(&successful_text(
        current.path(),
        &[
            "--json", "task", "comment", "delete", task_id, comment_id, "--json",
        ],
    ))
    .unwrap();
    assert_eq!(deleted["data"]["id"], comment_id);
}
