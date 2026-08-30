use rusqlite::{Connection, params};
use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tbtm_core::task::{
    AvailableTasksInput, select_available_tasks, select_task_blocking_explanation,
    unresolved_upstream_task_ids,
};
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

fn create(current: &Path, title: &str) -> String {
    let output = tbtm(
        current,
        &[
            "task", "create", "--title", title, "--type", "task", "--json",
        ],
    );
    assert!(output.status.success());
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn explain(current: &Path, id: &str) -> Value {
    let output = tbtm(current, &["task", "blockers", id, "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
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
fn available_and_all_simultaneous_reasons_have_stable_shapes() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let available = create(temp.path(), "Available");
    let human = tbtm(temp.path(), &["task", "blockers", &available]);
    assert_eq!(
        String::from_utf8(human.stdout).unwrap(),
        format!("{available} is available.\n")
    );
    assert_eq!(
        explain(temp.path(), &available)["reasons"],
        serde_json::json!([])
    );

    let root = create(temp.path(), "Root");
    let upstream = create(temp.path(), "Upstream");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    connection.execute("UPDATE tasks SET archived = 1, archive_reason = 'test', status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1", [&root]).unwrap();
    connection.execute("INSERT INTO agents (id, base_name, display_name, created_at) VALUES ('agent-1', 'Agent', 'Agent One', '2026-01-01T00:00:00Z')", []).unwrap();
    connection.execute("INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, 'agent-1', '2026-01-01T00:00:00Z')", [&root]).unwrap();
    connection
        .execute(
            "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
            params![root, upstream],
        )
        .unwrap();

    let data = explain(temp.path(), &root);
    assert_eq!(data["available"], false);
    assert_eq!(
        data["reasons"],
        serde_json::json!(["archived", "completed", "claimed", "dependencies_blocked"])
    );
    assert_eq!(data["claim"]["agent"]["displayName"], "Agent One");
    assert_eq!(data["unresolvedDependencies"]["direct"][0]["id"], upstream);
}

#[test]
fn unresolved_graph_is_deduplicated_sorted_and_stops_at_completed_nodes() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root");
    let a = create(temp.path(), "A");
    let b = create(temp.path(), "B");
    let shared = create(temp.path(), "Shared");
    let stopped = create(temp.path(), "Stopped");
    let hidden = create(temp.path(), "Hidden");
    let database = temp.path().join(".tbtm/tbtm.db");
    let mut connection = Connection::open(&database).unwrap();
    for (downstream, upstream) in [
        (&root, &a),
        (&root, &b),
        (&a, &shared),
        (&b, &shared),
        (&a, &stopped),
        (&stopped, &hidden),
    ] {
        connection.execute("INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)", params![downstream, upstream]).unwrap();
    }
    connection.execute("UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1", [&stopped]).unwrap();

    let data = explain(temp.path(), &root);
    let direct: Vec<_> = data["unresolvedDependencies"]["direct"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap())
        .collect();
    let mut expected_direct = vec![a.as_str(), b.as_str()];
    expected_direct.sort();
    assert_eq!(direct, expected_direct);
    for item in data["unresolvedDependencies"]["direct"].as_array().unwrap() {
        assert_eq!(item["blockedByTaskIds"], serde_json::json!([shared]));
    }
    let recursive = data["unresolvedDependencies"]["recursive"]
        .as_array()
        .unwrap();
    assert_eq!(recursive.len(), 1);
    assert_eq!(recursive[0]["id"], shared);
    assert!(!data.to_string().contains(&hidden));

    let transaction = connection.transaction().unwrap();
    let selected = select_task_blocking_explanation(&transaction, &root, &database).unwrap();
    assert!(!selected.available);
    let unresolved = unresolved_upstream_task_ids(&transaction, &root, &database).unwrap();
    assert_eq!(unresolved, expected_direct);
    transaction.commit().unwrap();
}

#[test]
fn errors_and_unsupported_actor_use_documented_exits() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let missing = tbtm(temp.path(), &["task", "blockers", "missing", "--json"]);
    assert_eq!(missing.status.code(), Some(3));
    let value: Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(value["error"]["code"], "TASK_NOT_FOUND");
    let missing_human = tbtm(temp.path(), &["task", "blockers", "missing"]);
    assert_eq!(missing_human.status.code(), Some(3));
    assert!(missing_human.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_human.stderr).contains("missing"));
    let actor = tbtm(
        temp.path(),
        &["task", "blockers", "missing", "--agent", "x"],
    );
    assert_eq!(actor.status.code(), Some(2));
    assert!(actor.stdout.is_empty());
    let missing_argument = tbtm(temp.path(), &["task", "blockers"]);
    assert_eq!(missing_argument.status.code(), Some(2));
    assert!(missing_argument.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn unreadable_database_returns_permission_exit_without_partial_output() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Permission denied");
    let database = temp.path().join(".tbtm/tbtm.db");
    let original = fs::metadata(&database).unwrap().permissions();
    fs::set_permissions(&database, fs::Permissions::from_mode(0o000)).unwrap();
    let output = tbtm(temp.path(), &["task", "blockers", &task, "--json"]);
    fs::set_permissions(&database, original).unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stderr.is_empty());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!response["error"]["code"].as_str().unwrap().is_empty());
    assert!(response["data"].is_null());
}

#[test]
fn individual_reasons_match_the_authoritative_available_selector_and_recompute() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let archived = create(temp.path(), "Archived");
    let completed = create(temp.path(), "Completed");
    let claimed = create(temp.path(), "Claimed");
    let blocked = create(temp.path(), "Blocked");
    let upstream = create(temp.path(), "Upstream");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'test' WHERE id = ?1",
            [&archived],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1",
            [&completed],
        )
        .unwrap();
    connection.execute("INSERT INTO agents (id, base_name, display_name, created_at) VALUES ('agent-only', 'Agent', 'Agent Only', '2026-01-01T00:00:00Z')", []).unwrap();
    connection.execute("INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, 'agent-only', '2026-01-01T00:00:00Z')", [&claimed]).unwrap();
    connection
        .execute(
            "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
            params![blocked, upstream],
        )
        .unwrap();

    for (id, reason) in [
        (&archived, "archived"),
        (&completed, "completed"),
        (&claimed, "claimed"),
        (&blocked, "dependencies_blocked"),
    ] {
        assert_eq!(
            explain(temp.path(), id)["reasons"],
            serde_json::json!([reason])
        );
    }
    let available =
        select_available_tasks(&connection, &AvailableTasksInput::default(), &database).unwrap();
    for id in [&archived, &completed, &claimed, &blocked] {
        assert!(!available.iter().any(|task| &task.id == id));
    }

    connection
        .execute(
            "UPDATE tasks SET status_id = (SELECT id FROM statuses WHERE code = 'done') WHERE id = ?1",
            [&upstream],
        )
        .unwrap();
    let changed = explain(temp.path(), &blocked);
    assert_eq!(changed["available"], true);
    assert_eq!(changed["reasons"], serde_json::json!([]));
    assert!(
        changed["unresolvedDependencies"]["direct"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unavailable_human_and_json_outputs_are_complete_and_read_only() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root");
    let upstream = create(temp.path(), "Upstream");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection.execute("INSERT INTO agents (id, base_name, display_name, created_at) VALUES ('agent-1', 'Agent', 'Agent One', '2026-01-01T00:00:00Z')", []).unwrap();
    connection.execute("INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, 'agent-1', '2026-01-01T00:00:00Z')", [&root]).unwrap();
    connection
        .execute(
            "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
            params![root, upstream],
        )
        .unwrap();
    drop(connection);
    let before_database = fs::read(&database).unwrap();
    let before_entries: Vec<_> = fs::read_dir(temp.path().join(".tbtm"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();

    let human = tbtm(temp.path(), &["task", "blockers", &root]);
    assert!(human.status.success());
    assert_eq!(
        String::from_utf8(human.stdout).unwrap(),
        format!(
            "{root} is unavailable.\nReasons: claimed, dependencies_blocked\nClaim: Agent One (agent-1) since 2026-01-01T00:00:00Z\nDirect unresolved dependencies:\n- {upstream} [task, to_do] Upstream; blocked by: none\n"
        )
    );
    let json = explain(temp.path(), &root);
    assert_eq!(json["taskId"], root);
    assert_eq!(json["claim"]["agent"]["id"], "agent-1");
    assert_eq!(json["claim"]["claimedAt"], "2026-01-01T00:00:00Z");
    assert_eq!(
        json["unresolvedDependencies"]["direct"][0]["title"],
        "Upstream"
    );
    assert_eq!(fs::read(&database).unwrap(), before_database);
    let after_entries: Vec<_> = fs::read_dir(temp.path().join(".tbtm"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(after_entries, before_entries);
}

#[test]
fn direct_membership_wins_over_recursive_and_archived_edges_stop_traversal() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root");
    let direct = create(temp.path(), "Direct");
    let shared = create(temp.path(), "Shared");
    let archived = create(temp.path(), "Archived stop");
    let hidden = create(temp.path(), "Hidden");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(database).unwrap();
    for (downstream, upstream) in [
        (&root, &direct),
        (&root, &shared),
        (&direct, &shared),
        (&direct, &archived),
        (&archived, &hidden),
    ] {
        connection
            .execute(
                "INSERT INTO task_dependencies (downstream_task_id, upstream_task_id) VALUES (?1, ?2)",
                params![downstream, upstream],
            )
            .unwrap();
    }
    connection
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'satisfied' WHERE id = ?1",
            [&archived],
        )
        .unwrap();
    let data = explain(temp.path(), &root);
    let direct_ids: Vec<_> = data["unresolvedDependencies"]["direct"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    let mut expected = vec![direct.as_str(), shared.as_str()];
    expected.sort();
    assert_eq!(direct_ids, expected);
    assert!(
        data["unresolvedDependencies"]["recursive"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!data.to_string().contains(&archived));
    assert!(!data.to_string().contains(&hidden));
}

#[test]
fn explanation_uses_one_snapshot_and_scales_to_large_branched_graphs() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let root = create(temp.path(), "Root");
    let database = temp.path().join(".tbtm/tbtm.db");
    let mut reader = Connection::open(&database).unwrap();
    reader.pragma_update(None, "journal_mode", "WAL").unwrap();
    let writer = Connection::open(&database).unwrap();
    let transaction = reader.transaction().unwrap();
    transaction
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get::<_, i64>(0))
        .unwrap();
    writer
        .execute(
            "UPDATE tasks SET archived = 1, archive_reason = 'concurrent' WHERE id = ?1",
            [&root],
        )
        .unwrap();
    let snapshot = select_task_blocking_explanation(&transaction, &root, &database).unwrap();
    assert!(snapshot.available);
    transaction.commit().unwrap();
    assert_eq!(
        explain(temp.path(), &root)["reasons"],
        serde_json::json!(["archived"])
    );

    writer
        .execute(
            "UPDATE tasks SET archived = 0, archive_reason = NULL WHERE id = ?1",
            [&root],
        )
        .unwrap();
    writer.execute_batch(&format!(
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 1000)
         INSERT INTO tasks (id, short_suffix, task_type, title, description, goal, acceptance_criteria,
           status_id, priority, estimate_hours, archived, created_actor_type, created_agent_id,
           updated_actor_type, updated_agent_id, created_at, updated_at)
         SELECT printf('project-task-%08x', i), printf('%08x', i), 'task', printf('Node %d', i),
           '', '', '', (SELECT id FROM statuses WHERE code = 'to_do'), 0, NULL, 0, 'user', NULL,
           'user', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z' FROM n;
         WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 1000)
         INSERT INTO task_dependencies (downstream_task_id, upstream_task_id)
         SELECT '{}', printf('project-task-%08x', i) FROM n;",
        root.replace('\'', "''")
    )).unwrap();
    let large = explain(temp.path(), &root);
    assert_eq!(
        large["unresolvedDependencies"]["direct"]
            .as_array()
            .unwrap()
            .len(),
        1000
    );
    assert!(
        large["unresolvedDependencies"]["recursive"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn pending_migrations_are_not_applied_and_linked_worktrees_share_explanations() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let task = create(temp.path(), "Old schema");
    let database = temp.path().join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version >= 6", [])
        .unwrap();
    drop(connection);
    let output = tbtm(temp.path(), &["task", "blockers", &task, "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
        "DATABASE_UNAVAILABLE"
    );
    let connection = Connection::open(database).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 6",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        0
    );

    let (_temp, main, linked) = linked_worktree();
    initialize(&main);
    let shared = create(&main, "Shared");
    assert_eq!(explain(&main, &shared), explain(&linked, &shared));
    assert!(!linked.join(".tbtm").exists());
}
