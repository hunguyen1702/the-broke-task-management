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

fn initialize(current: &Path) {
    data(current, &["init", "--prefix", "project", "--json"]);
}

fn create(current: &Path, title: &str, status: &str) -> String {
    data(
        current,
        &[
            "task", "create", "--title", title, "--type", "task", "--status", status, "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn register(current: &Path, name: &str) -> String {
    data(current, &["agent", "register", name, "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn view(current: &Path, task: &str) -> Value {
    data(current, &["task", "view", task, "--json"])
}

fn claim(current: &Path, task: &str, agent: &str) -> Value {
    data(
        current,
        &["task", "claim", task, "--agent", agent, "--json"],
    )
}

fn update_status(current: &Path, task: &str, status: &str, actor: Option<&str>) -> Value {
    let mut arguments = vec!["task", "update", task, "--status", status];
    if let Some(agent) = actor {
        arguments.extend(["--agent", agent]);
    }
    arguments.push("--json");
    data(current, &arguments)
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
fn claimed_status_transitions_and_noops_preserve_the_exact_claim_for_every_actor() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let owner = register(temp.path(), "owner");
    let foreign = register(temp.path(), "foreign");

    for (index, initial_status) in ["to_do", "in_progress"].into_iter().enumerate() {
        let task = create(temp.path(), &format!("Matrix {index}"), initial_status);
        let before_claim = view(temp.path(), &task);
        let claimed = claim(temp.path(), &task, &owner);
        let exact_claim = claimed["claim"].clone();
        assert_eq!(claimed["status"], before_claim["status"]);
        assert_eq!(claimed["updatedAt"], before_claim["updatedAt"]);
        assert_eq!(claimed["updatedBy"], before_claim["updatedBy"]);

        let completed = update_status(temp.path(), &task, "done", None);
        assert_eq!(completed["status"]["code"], "done");
        assert_eq!(completed["updatedBy"], "user");
        assert_eq!(completed["claim"], exact_claim);

        let reopened = update_status(temp.path(), &task, initial_status, Some(&foreign));
        assert_eq!(reopened["status"]["code"], initial_status);
        assert_eq!(reopened["updatedBy"], foreign);
        assert_eq!(reopened["claim"], exact_claim);

        let owner_update = update_status(temp.path(), &task, "done", Some(&owner));
        assert_eq!(owner_update["updatedBy"], owner);
        assert_eq!(owner_update["claim"], exact_claim);
        let before_noop = owner_update.clone();
        let noop = update_status(temp.path(), &task, "done", Some(&foreign));
        assert_eq!(noop["updatedAt"], before_noop["updatedAt"]);
        assert_eq!(noop["updatedBy"], before_noop["updatedBy"]);
        assert_eq!(noop["claim"], exact_claim);

        let available = data(temp.path(), &["task", "available", "--json"]);
        assert!(
            available
                .as_array()
                .unwrap()
                .iter()
                .all(|candidate| candidate["id"] != task)
        );
        let blockers = data(temp.path(), &["task", "blockers", &task, "--json"]);
        assert_eq!(
            blockers["reasons"],
            serde_json::json!(["completed", "claimed"])
        );
    }
}

#[test]
fn owner_unclaim_of_a_completed_task_changes_only_the_claim() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let owner = register(temp.path(), "owner");
    let task = create(temp.path(), "Completed claim", "to_do");
    claim(temp.path(), &task, &owner);
    let before = update_status(temp.path(), &task, "done", None);

    let released = data(
        temp.path(),
        &["task", "unclaim", &task, "--agent", &owner, "--json"],
    );
    assert!(released["claim"].is_null());
    for field in ["status", "updatedAt", "updatedBy", "archived"] {
        assert_eq!(released[field], before[field], "field {field}");
    }
}

#[test]
fn archive_is_the_only_automatic_release_and_unarchive_never_restores_claims() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let owner = register(temp.path(), "owner");

    for (index, force) in [false, true].into_iter().enumerate() {
        let task = create(temp.path(), &format!("Archive completed {index}"), "to_do");
        let exact_claim = claim(temp.path(), &task, &owner)["claim"].clone();
        let completed = update_status(temp.path(), &task, "done", None);
        assert_eq!(completed["claim"], exact_claim);

        let archived = if force {
            data(
                temp.path(),
                &[
                    "task", "archive", &task, "--reason", "obsolete", "--force", "--yes", "--json",
                ],
            )
        } else {
            data(
                temp.path(),
                &[
                    "task", "archive", &task, "--reason", "obsolete", "--agent", &owner, "--json",
                ],
            )
        };
        assert_eq!(archived["status"], completed["status"]);
        assert!(archived["claim"].is_null());
        assert_eq!(archived["archived"], true);

        let restored = data(temp.path(), &["task", "unarchive", &task, "--json"]);
        assert_eq!(restored["task"]["status"], completed["status"]);
        assert!(restored["task"]["claim"].is_null());
        assert_eq!(restored["task"]["archived"], false);
    }
}

#[test]
fn linked_worktree_status_update_preserves_the_shared_claim() {
    let (_temp, main, linked) = linked_worktree();
    initialize(&main);
    let owner = register(&main, "owner");
    let foreign = register(&linked, "foreign");
    let task = create(&main, "Shared lifecycle", "in_progress");
    let exact_claim = claim(&main, &task, &owner)["claim"].clone();

    let updated = update_status(&linked, &task, "done", Some(&foreign));
    assert_eq!(updated["status"]["code"], "done");
    assert_eq!(updated["updatedBy"], foreign);
    assert_eq!(updated["claim"], exact_claim);
    assert_eq!(view(&main, &task), updated);
    assert!(!linked.join(".tbtm").exists());
}

#[test]
fn racing_status_update_and_claim_produces_only_valid_serial_outcomes() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let agent = register(temp.path(), "racer");
    let task = create(temp.path(), "Race", "to_do");
    let root = temp.path().to_path_buf();
    let update_root = root.clone();
    let update_task = task.clone();
    let claim_root = root.clone();
    let claim_task = task.clone();
    let claim_agent = agent.clone();

    let update = std::thread::spawn(move || {
        tbtm(
            &update_root,
            &["task", "update", &update_task, "--status", "done", "--json"],
        )
    });
    let claim = std::thread::spawn(move || {
        tbtm(
            &claim_root,
            &[
                "task",
                "claim",
                &claim_task,
                "--agent",
                &claim_agent,
                "--json",
            ],
        )
    });
    let update = update.join().unwrap();
    let claim = claim.join().unwrap();
    assert!(update.status.success());
    assert!(claim.status.success() || claim.status.code() == Some(2));
    if !claim.status.success() {
        let error: Value = serde_json::from_slice(&claim.stdout).unwrap();
        assert_eq!(error["error"]["code"], "TASK_NOT_AVAILABLE");
        assert_eq!(error["error"]["details"]["reason"], "completed");
    }
    let final_task = view(&root, &task);
    assert_eq!(final_task["status"]["code"], "done");
    if claim.status.success() {
        assert_eq!(final_task["claim"]["agent"]["id"], agent);
    } else {
        assert!(final_task["claim"].is_null());
    }
}
