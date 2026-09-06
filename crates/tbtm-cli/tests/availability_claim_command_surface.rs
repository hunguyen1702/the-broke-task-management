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
fn availability_and_claim_roles_are_discoverable_from_help() {
    let current = tempdir().unwrap();
    for (arguments, expected) in [
        (
            vec!["task", "--help"],
            vec!["available", "blockers", "claim", "claim-next", "unclaim"],
        ),
        (
            vec!["task", "available", "--help"],
            vec![
                "currently available",
                "--status",
                "--type",
                "--tag",
                "repeatable",
            ],
        ),
        (
            vec!["task", "blockers", "--help"],
            vec!["unavailable", "without reserving or claiming"],
        ),
        (
            vec!["task", "claim", "--help"],
            vec![
                "claim atomically",
                "--agent <AGENT>",
                "Registered agent UUID",
            ],
        ),
        (
            vec!["task", "claim-next", "--help"],
            vec![
                "next available",
                "--agent <AGENT>",
                "--status",
                "--type",
                "--tag",
                "repeatable",
            ],
        ),
        (
            vec!["task", "unclaim", "--help"],
            vec![
                "--agent <AGENT>",
                "owning-agent UUID",
                "--force",
                "logical user",
                "--yes",
                "interactive prompt",
            ],
        ),
    ] {
        let help = successful_text(current.path(), &arguments);
        assert_contains_all(&help, &expected, &arguments);
    }
}

#[test]
fn actor_force_and_confirmation_options_stay_on_their_owned_paths() {
    let current = tempdir().unwrap();
    let agent = "00000000-0000-0000-0000-000000000000";
    for arguments in [
        vec!["task", "available", "--agent", agent],
        vec!["task", "blockers", "TASK-1", "--agent", agent],
        vec!["task", "claim", "TASK-1"],
        vec!["task", "claim-next"],
        vec!["task", "unclaim", "TASK-1"],
    ] {
        let output = tbtm(current.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "args={arguments:?}");
        assert!(output.stdout.is_empty(), "args={arguments:?}");
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

fn create_task(current: &Path, title: &str) -> String {
    successful_json(
        current,
        &[
            "task", "create", "--title", title, "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn linked_worktrees_share_the_availability_claim_and_release_lifecycle() {
    let (_temp, main, linked) = linked_worktree();
    successful_json(&main, &["--json", "init", "--prefix", "acquire"]);
    let agent = successful_json(&linked, &["agent", "register", "worker", "--json"]);
    let agent_id = agent["id"].as_str().unwrap();
    let free = create_task(&main, "Free");
    let upstream = create_task(&linked, "Upstream");
    let downstream = create_task(&main, "Downstream");
    successful_json(
        &linked,
        &[
            "task",
            "dependency",
            "add",
            &downstream,
            "--depends-on",
            &upstream,
            "--json",
        ],
    );

    let available = successful_json(
        &linked,
        &[
            "task",
            "available",
            "--type",
            "task",
            "--type",
            "task",
            "--json",
        ],
    );
    assert!(
        available
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == free)
    );
    assert!(
        !available
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == downstream)
    );
    let blockers = successful_json(&main, &["--json", "task", "blockers", &downstream]);
    assert_eq!(blockers["available"], false);

    let claimed = successful_json(
        &linked,
        &["task", "claim", &free, "--agent", agent_id, "--json"],
    );
    assert_eq!(claimed["claim"]["agent"]["id"], agent_id);
    let conflict = tbtm(
        &main,
        &["task", "claim", &free, "--agent", agent_id, "--json"],
    );
    assert_eq!(conflict.status.code(), Some(4));
    assert_eq!(
        serde_json::from_slice::<Value>(&conflict.stdout).unwrap()["error"]["code"],
        "CLAIM_CONFLICT"
    );
    successful_json(
        &main,
        &["task", "unclaim", &free, "--agent", agent_id, "--json"],
    );
    successful_json(
        &main,
        &["task", "update", &free, "--status", "done", "--json"],
    );

    successful_json(
        &linked,
        &["task", "update", &upstream, "--status", "done", "--json"],
    );
    let next = successful_json(
        &main,
        &[
            "--json",
            "task",
            "claim-next",
            "--agent",
            agent_id,
            "--status",
            "to_do",
        ],
    );
    assert_eq!(next["id"], downstream);
    let released = successful_json(
        &linked,
        &["task", "unclaim", &downstream, "--force", "--yes", "--json"],
    );
    assert_eq!(released["releasedClaim"]["agent"]["id"], agent_id);
    successful_json(
        &main,
        &["task", "update", &downstream, "--status", "done", "--json"],
    );

    let empty = successful_json(
        &main,
        &["task", "claim-next", "--agent", agent_id, "--json"],
    );
    assert!(empty.is_null());
}
