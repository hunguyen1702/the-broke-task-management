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

fn register(current: &std::path::Path, name: &str) -> (String, String) {
    let output = tbtm(current, &["agent", "register", name, "--json"]);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    (
        value["data"]["id"].as_str().unwrap().to_owned(),
        value["data"]["displayName"].as_str().unwrap().to_owned(),
    )
}

fn claim(current: &std::path::Path, task: &str, agent: &str) -> std::process::Output {
    tbtm(
        current,
        &["task", "claim", task, "--agent", agent, "--json"],
    )
}

#[test]
fn claim_populates_detail_and_list_without_mutating_task_metadata() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Claim me");
    let (agent, display_name) = register(temp.path(), "worker");
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &task, "--json"]).stdout)
            .unwrap();

    let output = claim(temp.path(), &task, &agent);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["data"]["claim"]["agent"]["id"], agent);
    assert_eq!(
        result["data"]["claim"]["agent"]["displayName"],
        display_name
    );
    assert!(
        result["data"]["claim"]["claimedAt"]
            .as_str()
            .unwrap()
            .ends_with('Z')
    );
    assert_eq!(result["data"]["updatedAt"], before["data"]["updatedAt"]);
    assert_eq!(result["data"]["updatedBy"], before["data"]["updatedBy"]);
    assert_eq!(result["data"]["status"], before["data"]["status"]);

    let detail: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &task, "--json"]).stdout)
            .unwrap();
    let list: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "list", "--json"]).stdout).unwrap();
    assert_eq!(detail["data"]["claim"], result["data"]["claim"]);
    assert_eq!(list["data"][0]["claim"], result["data"]["claim"]);
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    assert!(connection.execute(
        "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, ?2, '2026-01-01T00:00:00Z')",
        [&task, &agent],
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES ('missing', ?1, '2026-01-01T00:00:00Z')",
        [&agent],
    ).is_err());
}

#[test]
fn claim_validation_has_stable_precedence_reasons_and_details() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let (agent, _) = register(temp.path(), "owner");
    let (other, _) = register(temp.path(), "other");
    let task = create(temp.path(), "Claimed");
    let winner: Value = serde_json::from_slice(&claim(temp.path(), &task, &agent).stdout).unwrap();
    let conflict = claim(temp.path(), &task, &other);
    assert_eq!(conflict.status.code(), Some(4));
    let conflict: Value = serde_json::from_slice(&conflict.stdout).unwrap();
    assert_eq!(conflict["error"]["code"], "CLAIM_CONFLICT");
    assert_eq!(conflict["error"]["details"]["agent"]["id"], agent);
    assert_eq!(
        conflict["error"]["details"]["claimedAt"],
        winner["data"]["claim"]["claimedAt"]
    );
    assert_eq!(claim(temp.path(), &task, &agent).status.code(), Some(4));

    let archived = create(temp.path(), "Archived");
    let completed = create(temp.path(), "Completed");
    let blocked = create(temp.path(), "Blocked");
    let upstream_b = create(temp.path(), "Upstream B");
    let upstream_a = create(temp.path(), "Upstream A");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [&archived],
        )
        .unwrap();
    connection.execute("UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1", [&completed]).unwrap();
    for upstream in [&upstream_b, &upstream_a] {
        connection.execute("INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)", [&blocked, upstream]).unwrap();
    }
    for (id, reason) in [
        (&archived, "archived"),
        (&completed, "completed"),
        (&blocked, "dependencies_blocked"),
    ] {
        let output = claim(temp.path(), id, &other);
        assert_eq!(output.status.code(), Some(2));
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["error"]["code"], "TASK_NOT_AVAILABLE");
        assert_eq!(value["error"]["details"]["reason"], reason);
        if reason == "dependencies_blocked" {
            let mut expected = vec![upstream_a.clone(), upstream_b.clone()];
            expected.sort();
            assert_eq!(
                value["error"]["details"]["unresolvedUpstreamIds"],
                serde_json::json!(expected)
            );
        }
    }
    let missing = claim(temp.path(), "missing", &other);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["error"]["code"],
        "TASK_NOT_FOUND"
    );
    let unknown = uuid::Uuid::new_v4().to_string();
    let missing_agent = claim(temp.path(), &task, &unknown);
    assert_eq!(missing_agent.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing_agent.stdout).unwrap()["error"]["code"],
        "AGENT_NOT_FOUND"
    );
}

#[test]
fn archived_and_completed_upstreams_satisfy_claim_dependencies() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let (agent, _) = register(temp.path(), "worker");
    let downstream = create(temp.path(), "Downstream");
    let done = create(temp.path(), "Done upstream");
    let archived = create(temp.path(), "Archived upstream");
    let connection = Connection::open(temp.path().join(".tbtm/tbtm.db")).unwrap();
    connection.execute("UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1", [&done]).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test archive' WHERE id = ?1",
            [&archived],
        )
        .unwrap();
    for upstream in [&done, &archived] {
        connection.execute("INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)", [&downstream, upstream]).unwrap();
    }
    assert!(claim(temp.path(), &downstream, &agent).status.success());
}

#[test]
fn read_commands_reject_pending_claim_migration_until_a_write_upgrades() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Old schema");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version >= 5", [])
        .unwrap();
    connection.execute("DROP TABLE task_hierarchy", []).unwrap();
    connection.execute("DROP TABLE task_claims", []).unwrap();
    connection
        .execute("ALTER TABLE tasks DROP COLUMN archive_reason", [])
        .unwrap();
    drop(connection);
    for args in [
        vec!["task", "view", &task, "--json"],
        vec!["task", "list", "--json"],
    ] {
        let output = tbtm(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
            "DATABASE_UNAVAILABLE"
        );
    }
    assert!(
        tbtm(
            temp.path(),
            &["task", "update", &task, "--clear-tags", "--json"]
        )
        .status
        .success()
    );
    assert!(
        tbtm(temp.path(), &["task", "view", &task, "--json"])
            .status
            .success()
    );
}

#[test]
fn concurrent_claims_persist_exactly_one_winner() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Race");
    let (agent_a, _) = register(temp.path(), "a");
    let (agent_b, _) = register(temp.path(), "b");
    let root = temp.path().to_path_buf();
    let handles: Vec<_> = [agent_a, agent_b]
        .into_iter()
        .map(|agent| {
            let root = root.clone();
            let task = task.clone();
            std::thread::spawn(move || claim(&root, &task, &agent))
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
            .filter(|output| output.status.code() == Some(4))
            .count(),
        1
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
fn claim_human_output_and_argument_errors_are_stable() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Human");
    let (agent, display_name) = register(temp.path(), "human-worker");
    let output = tbtm(temp.path(), &["task", "claim", &task, "--agent", &agent]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Claimed task: {task}")));
    assert!(stdout.contains(&format!("Agent: {display_name}")));
    assert!(stdout.contains("Claimed at:"));
    assert_eq!(
        tbtm(temp.path(), &["task", "claim", &task]).status.code(),
        Some(2)
    );
    assert_eq!(
        tbtm(temp.path(), &["task", "claim", &task, "--agent", "bad"])
            .status
            .code(),
        Some(2)
    );
}
