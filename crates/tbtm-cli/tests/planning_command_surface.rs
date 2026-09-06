use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tempfile::{TempDir, tempdir};

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

fn successful_json(current: &Path, arguments: &[&str]) -> Value {
    let response: Value = serde_json::from_str(&successful_text(current, arguments)).unwrap();
    assert_eq!(response["ok"], true, "args={arguments:?}");
    assert!(response["error"].is_null(), "args={arguments:?}");
    response["data"].clone()
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
fn planning_commands_roles_and_options_are_discoverable_from_help() {
    let current = tempdir().unwrap();
    for (arguments, expected) in [
        (vec!["--help"], vec!["status", "task", "--json"]),
        (
            vec!["status", "--help"],
            vec![
                "list",
                "create",
                "rename",
                "move",
                "delete",
                "set-completed",
            ],
        ),
        (
            vec!["status", "create", "--help"],
            vec!["--code", "--name", "--before", "--after", "--json"],
        ),
        (
            vec!["status", "set-completed", "--help"],
            vec![
                "Status code",
                "--completed",
                "true",
                "false",
                "--yes",
                "Confirm",
            ],
        ),
        (vec!["task", "parent", "--help"], vec!["set", "remove"]),
        (
            vec!["task", "parent", "set", "--help"],
            vec![
                "Child task",
                "--parent",
                "Parent task",
                "--agent",
                "logical-user",
            ],
        ),
        (vec!["task", "dependency", "--help"], vec!["add", "remove"]),
        (
            vec!["task", "dependency", "add", "--help"],
            vec![
                "Downstream task",
                "--depends-on",
                "upstream task",
                "--agent",
            ],
        ),
        (
            vec!["task", "hierarchy", "--help"],
            vec!["direct hierarchy", "--recursive", "ancestor", "descendant"],
        ),
        (
            vec!["task", "map", "--help"],
            vec![
                "Root task",
                "--direction",
                "default: all",
                "upstream",
                "downstream",
                "parent",
                "child",
            ],
        ),
    ] {
        let help = successful_text(current.path(), &arguments);
        assert_contains_all(&help, &expected, &arguments);
    }
}

#[test]
fn actor_and_confirmation_options_stay_on_their_owned_mutations() {
    let current = tempdir().unwrap();
    let agent = "00000000-0000-0000-0000-000000000000";
    for arguments in [
        vec!["status", "list", "--agent", agent],
        vec![
            "status",
            "set-completed",
            "done",
            "--completed",
            "false",
            "--agent",
            agent,
        ],
        vec!["task", "hierarchy", "TASK-1", "--agent", agent],
        vec!["task", "map", "TASK-1", "--agent", agent],
        vec![
            "task", "parent", "set", "TASK-1", "--parent", "TASK-2", "--yes",
        ],
        vec![
            "task",
            "dependency",
            "add",
            "TASK-1",
            "--depends-on",
            "TASK-2",
            "--yes",
        ],
    ] {
        let output = tbtm(current.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "args={arguments:?}");
        assert!(output.stdout.is_empty(), "args={arguments:?}");
        assert_contains_all(
            &String::from_utf8(output.stderr).unwrap(),
            &["unexpected argument"],
            &arguments,
        );
    }
    assert!(!current.path().join(".tbtm").exists());
}

fn linked_worktree() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
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
    (temp, main, linked)
}

fn create_task(current: &Path, title: &str, task_type: &str) -> String {
    successful_json(
        current,
        &[
            "task", "create", "--title", title, "--type", task_type, "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn linked_worktrees_share_status_relationship_hierarchy_and_map_operations() {
    let (_temp, main, linked) = linked_worktree();
    successful_json(&main, &["--json", "init", "--prefix", "planning"]);
    let agent = successful_json(&linked, &["agent", "register", "planner", "--json"]);
    let agent_id = agent["id"].as_str().unwrap();

    successful_json(
        &linked,
        &[
            "status", "create", "--code", "review", "--name", "Review", "--before", "done",
            "--json",
        ],
    );
    successful_json(
        &main,
        &[
            "status",
            "rename",
            "review",
            "--name",
            "QA Review",
            "--json",
        ],
    );
    let statuses = successful_json(&linked, &["--json", "status", "list"]);
    assert!(
        statuses
            .as_array()
            .unwrap()
            .iter()
            .any(|status| { status["code"] == "review" && status["name"] == "QA Review" })
    );

    let epic = create_task(&main, "Epic", "epic");
    let story = create_task(&linked, "Story", "story");
    let child = create_task(&main, "Child", "task");
    let upstream = create_task(&linked, "Upstream", "task");
    let downstream = create_task(&main, "Downstream", "task");
    successful_json(
        &linked,
        &[
            "task", "parent", "set", &story, "--parent", &epic, "--agent", agent_id, "--json",
        ],
    );
    successful_json(
        &main,
        &[
            "task", "parent", "set", &child, "--parent", &story, "--json",
        ],
    );
    successful_json(
        &linked,
        &[
            "task",
            "dependency",
            "add",
            &story,
            "--depends-on",
            &upstream,
            "--agent",
            agent_id,
            "--json",
        ],
    );
    successful_json(
        &main,
        &[
            "task",
            "dependency",
            "add",
            &downstream,
            "--depends-on",
            &story,
            "--json",
        ],
    );

    let hierarchy = successful_json(
        &linked,
        &["task", "hierarchy", &story, "--recursive", "--json"],
    );
    assert_eq!(hierarchy["parent"]["id"], epic);
    assert_eq!(hierarchy["descendants"][0]["id"], child);

    for (direction, expected) in [
        ("parent", &epic),
        ("child", &child),
        ("upstream", &upstream),
        ("downstream", &downstream),
    ] {
        let map = successful_json(
            &main,
            &["task", "map", &story, "--direction", direction, "--json"],
        );
        assert_eq!(map["direction"], direction);
        assert!(
            map["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|node| node["id"] == *expected)
        );
    }
    let default_map = successful_json(&linked, &["task", "map", &story, "--json"]);
    let explicit_all = successful_json(
        &main,
        &["task", "map", &story, "--direction", "all", "--json"],
    );
    assert_eq!(default_map, explicit_all);
    assert_eq!(default_map["nodes"].as_array().unwrap().len(), 5);

    successful_json(
        &linked,
        &["task", "update", &child, "--status", "review", "--json"],
    );
    let required = tbtm(
        &main,
        &[
            "status",
            "set-completed",
            "review",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert_eq!(required.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&required.stdout).unwrap()["error"]["code"],
        "CONFIRMATION_REQUIRED"
    );
    let changed = successful_json(
        &main,
        &[
            "status",
            "set-completed",
            "review",
            "--completed",
            "true",
            "--yes",
            "--json",
        ],
    );
    assert_eq!(changed["status"]["completed"], true);
    let no_op = successful_json(
        &linked,
        &[
            "status",
            "set-completed",
            "review",
            "--completed",
            "true",
            "--json",
        ],
    );
    assert_eq!(
        no_op["impact"],
        serde_json::json!({"statusTasks": [], "downstreamTasks": []})
    );

    assert!(!linked.join(".tbtm").exists());
    assert!(main.join(".tbtm/tbtm.db").is_file());
}
