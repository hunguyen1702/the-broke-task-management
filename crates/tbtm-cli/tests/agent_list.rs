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

fn data(current: &std::path::Path, arguments: &[&str]) -> Value {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

#[test]
fn empty_registry_is_successful_and_read_only() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let database = temp.path().join(".tbtm/tbtm.db");
    let before = std::fs::read(&database).unwrap();
    let result = data(temp.path(), &["agent", "list", "--json"]);
    assert_eq!(result["agents"], serde_json::json!([]));
    assert_eq!(std::fs::read(database).unwrap(), before);
}

#[test]
fn list_includes_unclaimed_agents_and_current_claim_summaries() {
    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let first = data(temp.path(), &["agent", "register", "first", "--json"]);
    let second = data(temp.path(), &["agent", "register", "second", "--json"]);
    let task_a = data(
        temp.path(),
        &[
            "task", "create", "--title", "Alpha", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let task_b = data(
        temp.path(),
        &[
            "task", "create", "--title", "Beta", "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let first_id = first["id"].as_str().unwrap();
    data(
        temp.path(),
        &["task", "claim", &task_b, "--agent", first_id, "--json"],
    );
    data(
        temp.path(),
        &["task", "claim", &task_a, "--agent", first_id, "--json"],
    );

    let agents = data(temp.path(), &["agent", "list", "--json"])["agents"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(agents.len(), 2);
    let claimed = agents
        .iter()
        .find(|agent| agent["id"] == first["id"])
        .unwrap();
    let unclaimed = agents
        .iter()
        .find(|agent| agent["id"] == second["id"])
        .unwrap();
    assert_eq!(claimed["baseName"], "first");
    assert_eq!(claimed["claims"].as_array().unwrap().len(), 2);
    assert_eq!(claimed["claims"][0]["status"], "Todo");
    assert!(
        claimed["claims"][0]["claimedAt"]
            .as_str()
            .unwrap()
            .ends_with('Z')
    );
    assert_eq!(unclaimed["claims"], serde_json::json!([]));

    data(
        temp.path(),
        &["task", "unclaim", &task_a, "--agent", first_id, "--json"],
    );
    let after = data(temp.path(), &["agent", "list", "--json"]);
    assert_eq!(after["agents"][0]["claims"].as_array().unwrap().len(), 1);
}

#[test]
fn human_output_and_repository_errors_are_stable() {
    let empty = tempdir().unwrap();
    let missing = tbtm(empty.path(), &["agent", "list", "--json"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["error"]["code"],
        "REPOSITORY_NOT_INITIALIZED"
    );

    let temp = tempdir().unwrap();
    data(temp.path(), &["init", "--prefix", "project", "--json"]);
    let empty_human = tbtm(temp.path(), &["agent", "list"]);
    assert_eq!(
        String::from_utf8(empty_human.stdout).unwrap(),
        "No registered agents.\n"
    );
    let agent = data(temp.path(), &["agent", "register", "worker", "--json"]);
    let human = String::from_utf8(tbtm(temp.path(), &["agent", "list"]).stdout).unwrap();
    assert!(human.contains(agent["displayName"].as_str().unwrap()));
    assert!(human.contains("Claims: none"));
    assert_eq!(
        tbtm(temp.path(), &["agent", "list", "extra"]).status.code(),
        Some(2)
    );
}
