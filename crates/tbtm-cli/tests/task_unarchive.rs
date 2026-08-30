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

fn data(current: &Path, arguments: &[&str]) -> Value {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn create(current: &Path, title: &str, status: Option<&str>) -> String {
    let mut arguments = vec![
        "task", "create", "--title", title, "--type", "task", "--json",
    ];
    if let Some(status) = status {
        arguments.extend(["--status", status]);
    }
    data(current, &arguments)["id"].as_str().unwrap().to_owned()
}

fn add_dependency(current: &Path, downstream: &str, upstream: &str) {
    data(
        current,
        &[
            "task",
            "dependency",
            "add",
            downstream,
            "--depends-on",
            upstream,
            "--json",
        ],
    );
}

fn archive(current: &Path, task: &str) {
    data(
        current,
        &[
            "task",
            "archive",
            task,
            "--reason",
            "temporarily removed",
            "--json",
        ],
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
                linked.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success()
    );
    (temp, main, linked)
}

#[test]
fn unarchive_requires_confirmation_and_returns_ordered_direct_impact() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let target = create(temp.path(), "Target", None);
    let available_b = create(temp.path(), "Available B", None);
    let available_a = create(temp.path(), "Available A", None);
    let claimed = create(temp.path(), "Claimed", None);
    let blocked = create(temp.path(), "Blocked", None);
    let other_blocker = create(temp.path(), "Other blocker", None);
    let recursive = create(temp.path(), "Recursive", None);
    for downstream in [&available_b, &available_a, &claimed, &blocked] {
        add_dependency(temp.path(), downstream, &target);
    }
    add_dependency(temp.path(), &blocked, &other_blocker);
    add_dependency(temp.path(), &recursive, &available_a);
    archive(temp.path(), &target);
    let agent = data(temp.path(), &["agent", "register", "worker", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &claimed, "--agent", &agent, "--json"],
    );

    let rejected = tbtm(temp.path(), &["task", "unarchive", &target, "--json"]);
    assert_eq!(rejected.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&rejected.stdout).unwrap();
    assert_eq!(error["error"]["code"], "CONFIRMATION_REQUIRED");
    let impact = &error["error"]["details"];
    assert_eq!(impact["claimed"][0]["taskId"], claimed);
    assert_eq!(impact["claimed"][0]["claim"]["agent"]["id"], agent);
    assert_eq!(impact["alreadyBlockedElsewhere"][0]["taskId"], blocked);
    assert_eq!(
        impact["alreadyBlockedElsewhere"][0]["otherUnresolvedUpstreamTaskIds"],
        serde_json::json!([other_blocker])
    );
    let mut expected_available = [available_a.clone(), available_b.clone()];
    expected_available.sort();
    assert_eq!(
        impact["otherwiseAvailable"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["taskId"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_available
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
    assert!(
        impact
            .as_object()
            .unwrap()
            .values()
            .flat_map(|group| group.as_array().unwrap())
            .all(|item| item["taskId"] != recursive)
    );
    assert_eq!(
        data(temp.path(), &["task", "view", &target, "--json"])["archived"],
        true
    );
    let non_interactive = tbtm(temp.path(), &["task", "unarchive", &target]);
    assert_eq!(non_interactive.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&non_interactive.stderr);
    assert!(stderr.contains("CONFIRMATION_REQUIRED"));
    assert!(stderr.contains(&claimed));
    assert!(stderr.contains(&blocked));

    let accepted = data(
        temp.path(),
        &["task", "unarchive", &target, "--yes", "--json"],
    );
    assert_eq!(accepted["task"]["archived"], false);
    assert!(accepted["task"]["archiveReason"].is_null());
    assert!(accepted["task"]["claim"].is_null());
    assert_eq!(accepted["task"]["updatedBy"], "user");
    assert_eq!(accepted["impact"], *impact);
    assert_eq!(
        data(temp.path(), &["task", "view", &claimed, "--json"])["claim"]["agent"]["id"],
        agent
    );
}

#[test]
fn unarchive_completed_target_has_no_impact_and_active_noop_preserves_claim() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let completed = create(temp.path(), "Completed", Some("done"));
    let downstream = create(temp.path(), "Downstream", None);
    add_dependency(temp.path(), &downstream, &completed);
    archive(temp.path(), &completed);
    let restored = data(temp.path(), &["task", "unarchive", &completed, "--json"]);
    assert_eq!(restored["task"]["archived"], false);
    assert_eq!(
        restored["impact"],
        serde_json::json!({
            "claimed": [],
            "otherwiseAvailable": [],
            "alreadyBlockedElsewhere": []
        })
    );

    let active = create(temp.path(), "Active", None);
    let agent = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &active, "--agent", &agent, "--json"],
    );
    let before = data(temp.path(), &["task", "view", &active, "--json"]);
    let noop = data(temp.path(), &["task", "unarchive", &active, "--json"]);
    assert_eq!(noop["task"], before);
    assert_eq!(noop["task"]["claim"]["agent"]["id"], agent);
    assert_eq!(noop["task"]["updatedAt"], before["updatedAt"]);
}

#[test]
fn unarchive_rejects_agent_and_force_options_at_parse_time() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = create(temp.path(), "Target", None);
    for option in ["--agent", "--force"] {
        let mut arguments = vec!["task", "unarchive", &task, option];
        if option == "--agent" {
            arguments.push("00000000-0000-0000-0000-000000000000");
        }
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    }
}

#[test]
fn unarchive_from_linked_worktree_mutates_only_the_shared_store() {
    let (_temp, main, linked) = linked_worktree();
    data(&main, &["init", "--prefix", "project", "--json"]);
    let target = create(&main, "Target", None);
    archive(&main, &target);
    let restored = data(&linked, &["task", "unarchive", &target, "--yes", "--json"]);
    assert_eq!(restored["task"]["archived"], false);
    assert_eq!(
        data(&main, &["task", "view", &target, "--json"])["archived"],
        false
    );
    assert!(!linked.join(".tbtm").exists());
}
