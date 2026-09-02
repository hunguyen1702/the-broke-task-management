use rusqlite::Connection;
use serde_json::Value;
use std::process::Command;
use tbtm_core::{
    Error,
    task::{ForceUnclaimTaskInput, ObservedClaim, force_unclaim_task},
};
use tempfile::tempdir;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn data(current: &std::path::Path, arguments: &[&str]) -> Value {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn linked_worktree() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let main = temp.path().join("main");
    let linked = temp.path().join("linked");
    std::fs::create_dir(&main).unwrap();
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

#[test]
fn owner_unclaim_removes_only_claim_and_allows_reclaim() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Release", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let next = data(temp.path(), &["agent", "register", "next", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &task, "--agent", &owner, "--json"],
    );
    let before = data(temp.path(), &["task", "view", &task, "--json"]);

    let released = data(
        temp.path(),
        &["task", "unclaim", &task, "--agent", &owner, "--json"],
    );
    assert!(released["claim"].is_null());
    for field in [
        "status",
        "archived",
        "updatedAt",
        "updatedBy",
        "title",
        "description",
        "goal",
        "acceptanceCriteria",
        "dependencies",
    ] {
        assert_eq!(released[field], before[field], "field {field}");
    }
    let reclaimed = data(
        temp.path(),
        &["task", "claim", &task, "--agent", &next, "--json"],
    );
    assert_eq!(reclaimed["claim"]["agent"]["id"], next);
}

#[test]
fn unclaim_errors_preserve_claim_and_have_stable_details() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Owned", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let empty = data(
        temp.path(),
        &[
            "task", "create", "--title", "Empty", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let other = data(temp.path(), &["agent", "register", "other", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let claimed = data(
        temp.path(),
        &["task", "claim", &task, "--agent", &owner, "--json"],
    );

    let foreign = tbtm(
        temp.path(),
        &["task", "unclaim", &task, "--agent", &other, "--json"],
    );
    assert_eq!(foreign.status.code(), Some(5));
    let foreign: Value = serde_json::from_slice(&foreign.stdout).unwrap();
    assert_eq!(foreign["error"]["code"], "CLAIM_NOT_OWNED");
    assert_eq!(foreign["error"]["details"]["agent"]["id"], owner);
    assert_eq!(
        foreign["error"]["details"]["claimedAt"],
        claimed["claim"]["claimedAt"]
    );

    let missing = tbtm(
        temp.path(),
        &["task", "unclaim", &empty, "--agent", &owner, "--json"],
    );
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["error"]["code"],
        "CLAIM_NOT_FOUND"
    );
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM task_claims WHERE task_id = ?1",
            [&task],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn concurrent_owner_unclaims_have_one_winner() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Race", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &task, "--agent", &owner, "--json"],
    );
    let root = temp.path().to_path_buf();
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let root = root.clone();
            let task = task.clone();
            let owner = owner.clone();
            std::thread::spawn(move || {
                tbtm(
                    &root,
                    &["task", "unclaim", &task, "--agent", &owner, "--json"],
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
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.code() == Some(3))
            .count(),
        1
    );
}

#[test]
fn unclaim_human_output_and_arguments_are_stable() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Human", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let agent_data = data(temp.path(), &["agent", "register", "worker", "--json"]);
    let agent = agent_data["id"].as_str().unwrap();
    let display_name = agent_data["displayName"].as_str().unwrap();
    data(
        temp.path(),
        &["task", "claim", &task, "--agent", agent, "--json"],
    );
    let output = tbtm(temp.path(), &["task", "unclaim", &task, "--agent", agent]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Unclaimed task: {task}")));
    assert!(stdout.contains(&format!("Agent: {display_name}")));
    assert_eq!(
        tbtm(temp.path(), &["task", "unclaim", &task]).status.code(),
        Some(2)
    );
    assert_eq!(
        tbtm(temp.path(), &["task", "unclaim", &task, "--agent", "bad"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn linked_worktree_unclaim_updates_shared_claim_state() {
    let (_temp, main, linked) = linked_worktree();
    data(&main, &["init", "--prefix", "project", "--json"]);
    let task = data(
        &main,
        &[
            "task", "create", "--title", "Shared", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner = data(&linked, &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        &main,
        &["task", "claim", &task, "--agent", &owner, "--json"],
    );
    let released = data(
        &linked,
        &["task", "unclaim", &task, "--agent", &owner, "--json"],
    );
    assert!(released["claim"].is_null());
    assert!(data(&main, &["task", "view", &task, "--json"])["claim"].is_null());
}

#[test]
fn force_unclaim_yes_returns_released_claim_and_authoritative_availability() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let upstream = data(
        temp.path(),
        &[
            "task", "create", "--title", "Upstream", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Stale", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let agent = data(temp.path(), &["agent", "register", "stale", "--json"]);
    let agent_id = agent["id"].as_str().unwrap();
    let claimed = data(
        temp.path(),
        &["task", "claim", &task, "--agent", agent_id, "--json"],
    );
    data(
        temp.path(),
        &[
            "task",
            "dependency",
            "add",
            &task,
            "--depends-on",
            &upstream,
            "--json",
        ],
    );

    let result = data(
        temp.path(),
        &["task", "unclaim", &task, "--force", "--yes", "--json"],
    );
    assert!(result["task"]["claim"].is_null());
    assert_eq!(result["releasedClaim"]["agent"]["id"], agent["id"]);
    assert_eq!(
        result["releasedClaim"]["agent"]["displayName"],
        agent["displayName"]
    );
    assert_eq!(
        result["releasedClaim"]["claimedAt"],
        claimed["claim"]["claimedAt"]
    );
    assert_eq!(result["availability"]["available"], false);
    assert_eq!(result["availability"]["reason"], "dependencies_blocked");
    assert_eq!(
        result["availability"]["unresolvedUpstreamIds"],
        serde_json::json!([upstream])
    );
}

#[test]
fn force_unclaim_validates_flags_confirmation_and_observed_claim() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let task = data(
        temp.path(),
        &[
            "task", "create", "--title", "Guarded", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let agent = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let claimed = data(
        temp.path(),
        &["task", "claim", &task, "--agent", &agent, "--json"],
    );

    for arguments in [
        vec!["task", "unclaim", &task, "--force", "--json"],
        vec!["task", "unclaim", &task, "--yes", "--json"],
        vec![
            "task", "unclaim", &task, "--agent", &agent, "--force", "--yes", "--json",
        ],
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(2));
    }

    let error = force_unclaim_task(
        temp.path(),
        ForceUnclaimTaskInput {
            task_id: task.clone(),
            observed_claim: Some(ObservedClaim {
                agent_id: agent.clone(),
                claimed_at: "2000-01-01T00:00:00Z".to_owned(),
            }),
        },
    )
    .unwrap_err();
    assert!(matches!(error, Error::ClaimChanged { .. }));
    assert_eq!(
        data(temp.path(), &["task", "view", &task, "--json"])["claim"],
        claimed["claim"]
    );
}
