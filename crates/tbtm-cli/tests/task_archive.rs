use rusqlite::Connection;
use serde_json::Value;
use std::{path::Path, process::Command};
use tbtm_core::task::{ArchiveTaskInput, ObservedClaim, archive_task};
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
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn setup(current: &Path) -> String {
    data(current, &["init", "--prefix", "project", "--json"]);
    data(
        current,
        &[
            "task",
            "create",
            "--title",
            "Archive me",
            "--type",
            "task",
            "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn archive_preserves_detail_releases_own_claim_and_is_idempotent() {
    let temp = tempdir().unwrap();
    let task = setup(temp.path());
    let agent = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &task, "--agent", &agent, "--json"],
    );
    let before = data(temp.path(), &["task", "view", &task, "--json"]);
    let archived = data(
        temp.path(),
        &[
            "task",
            "archive",
            &task,
            "--reason",
            "  **obsolete**  ",
            "--agent",
            &agent,
            "--json",
        ],
    );
    assert_eq!(archived["archived"], true);
    assert_eq!(archived["archiveReason"], "**obsolete**");
    assert!(archived["claim"].is_null());
    for field in [
        "id",
        "title",
        "description",
        "goal",
        "acceptanceCriteria",
        "status",
        "dependencies",
        "createdAt",
        "createdBy",
    ] {
        assert_eq!(archived[field], before[field], "field {field}");
    }
    let repeated = data(
        temp.path(),
        &[
            "task",
            "archive",
            &task,
            "--reason",
            "**obsolete**",
            "--agent",
            &agent,
            "--json",
        ],
    );
    assert_eq!(repeated["updatedAt"], archived["updatedAt"]);
    assert_eq!(repeated["updatedBy"], archived["updatedBy"]);
    assert_eq!(
        data(temp.path(), &["task", "view", &task, "--json"]),
        archived
    );
}

#[test]
fn archive_validation_claim_guards_and_force_have_stable_exits() {
    let temp = tempdir().unwrap();
    let task = setup(temp.path());
    let owner = data(temp.path(), &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let other = data(temp.path(), &["agent", "register", "other", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
        &["task", "claim", &task, "--agent", &owner, "--json"],
    );

    for (arguments, code, exit) in [
        (
            vec!["task", "archive", &task, "--reason", " ", "--json"],
            "INVALID_ARCHIVE_REASON",
            2,
        ),
        (
            vec!["task", "archive", &task, "--reason", "x", "--yes", "--json"],
            "CONFLICTING_ARGUMENTS",
            2,
        ),
        (
            vec![
                "task", "archive", &task, "--reason", "x", "--force", "--json",
            ],
            "CONFIRMATION_REQUIRED",
            2,
        ),
        (
            vec![
                "task", "archive", &task, "--reason", "x", "--agent", &other, "--json",
            ],
            "TASK_CLAIMED",
            4,
        ),
        (
            vec![
                "task", "archive", &task, "--reason", "x", "--agent", &other, "--force", "--yes",
                "--json",
            ],
            "PERMISSION_DENIED",
            5,
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit));
        let body: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(body["error"]["code"], code);
        if code == "TASK_CLAIMED" {
            assert_eq!(body["error"]["details"]["agent"]["id"], owner);
            assert!(body["error"]["details"]["claimedAt"].is_string());
        }
    }
    let forced = data(
        temp.path(),
        &[
            "task", "archive", &task, "--reason", "x", "--force", "--yes", "--json",
        ],
    );
    assert_eq!(forced["archiveReason"], "x");
    assert!(forced["claim"].is_null());
}

#[test]
fn force_never_releases_a_replacement_claim_and_archive_unblocks_downstream() {
    let temp = tempdir().unwrap();
    let upstream = setup(temp.path());
    let downstream = data(
        temp.path(),
        &[
            "task",
            "create",
            "--title",
            "Downstream",
            "--type",
            "task",
            "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        temp.path(),
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
    let first = data(temp.path(), &["agent", "register", "first", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let second = data(temp.path(), &["agent", "register", "second", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let claim = data(
        temp.path(),
        &["task", "claim", &upstream, "--agent", &first, "--json"],
    );
    data(
        temp.path(),
        &["task", "unclaim", &upstream, "--agent", &first, "--json"],
    );
    data(
        temp.path(),
        &["task", "claim", &upstream, "--agent", &second, "--json"],
    );
    let error = archive_task(
        temp.path(),
        ArchiveTaskInput {
            task_id: upstream.clone(),
            reason: "obsolete".to_owned(),
            agent_id: None,
            force: true,
            observed_claim: Some(ObservedClaim {
                agent_id: first,
                claimed_at: claim["claim"]["claimedAt"].as_str().unwrap().to_owned(),
            }),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "TASK_CLAIMED");
    assert_eq!(error.details()["agent"]["id"], second);
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM task_claims WHERE task_id = ?1",
                [&upstream],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    drop(connection);
    data(
        temp.path(),
        &[
            "task", "archive", &upstream, "--reason", "obsolete", "--force", "--yes", "--json",
        ],
    );
    let available = data(temp.path(), &["task", "available", "--json"]);
    assert!(
        available
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == downstream)
    );
}

#[test]
fn create_exposes_null_reason_and_compact_list_omits_it() {
    let temp = tempdir().unwrap();
    let task = setup(temp.path());
    let viewed = data(temp.path(), &["task", "view", &task, "--json"]);
    assert!(viewed["archiveReason"].is_null());
    let listed = data(temp.path(), &["task", "list", "--json"]);
    assert!(listed[0].get("archiveReason").is_none());
}
