use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};
use tempfile::tempdir;

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn data(root: &Path, args: &[&str]) -> Value {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "operation {args:?}: exit={:?} stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn task(root: &Path, title: &str, status: &str) -> String {
    data(
        root,
        &[
            "task", "create", "--title", title, "--type", "task", "--status", status, "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn available(root: &Path, expected: &[&str]) {
    let args = ["task", "available", "--json"];
    let output = run(root, &args);
    assert!(
        output.status.success(),
        "operation {args:?}: exit={:?} stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let actual: Vec<_> = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        actual,
        expected,
        "operation {args:?}: exit={:?} stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn depends(root: &Path, downstream: &str, upstream: &str) {
    data(
        root,
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

#[test]
fn chain_claims_follow_direct_effective_completion_and_hierarchy_is_independent() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let agent = data(root, &["agent", "register", "worker", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let a = task(root, "A", "to_do");
    let b = task(root, "B", "to_do");
    let c = task(root, "C", "to_do");
    depends(root, &b, &a);
    depends(root, &c, &b);
    available(root, &[&a]);
    let blocked = run(root, &["task", "claim", &c, "--agent", &agent, "--json"]);
    assert_eq!(
        blocked.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&blocked.stdout)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&blocked.stdout).unwrap()["error"]["code"],
        "TASK_NOT_AVAILABLE"
    );
    data(root, &["task", "claim", &a, "--agent", &agent, "--json"]);
    available(root, &[]);
    data(root, &["task", "update", &a, "--status", "done", "--json"]);
    available(root, &[&b]);
    data(root, &["task", "claim", &b, "--agent", &agent, "--json"]);
    available(root, &[]);
    data(root, &["task", "update", &b, "--status", "done", "--json"]);
    available(root, &[&c]);
    data(root, &["task", "claim", &c, "--agent", &agent, "--json"]);
    available(root, &[]);

    let isolated = tempdir().unwrap();
    let root = isolated.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let a = task(root, "A", "to_do");
    let b = task(root, "B", "done");
    let c = task(root, "C", "to_do");
    depends(root, &b, &a);
    depends(root, &c, &b);
    available(root, &[&a, &c]);
    let blockers = data(root, &["task", "blockers", &c, "--json"]);
    assert_eq!(
        blockers["unresolvedDependencies"]["direct"],
        serde_json::json!([])
    );
    assert_eq!(
        blockers["unresolvedDependencies"]["recursive"],
        serde_json::json!([])
    );
    let map = data(
        root,
        &["task", "map", &c, "--direction", "upstream", "--json"],
    );
    let nodes: Vec<_> = map["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["id"].as_str().unwrap())
        .collect();
    assert!(nodes.contains(&a.as_str()));
    assert!(nodes.contains(&b.as_str()));
    let parent = data(
        root,
        &[
            "task", "create", "--title", "Parent", "--type", "epic", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        root,
        &["task", "parent", "set", &c, "--parent", &parent, "--json"],
    );
    available(root, &[&a, &c, &parent]);
}

#[test]
fn archive_and_shared_status_transitions_preserve_existing_claims() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let agent = data(root, &["agent", "register", "worker", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    data(
        root,
        &[
            "status", "create", "--code", "review", "--name", "Review", "--json",
        ],
    );
    let a = task(root, "A", "review");
    let b = task(root, "B", "review");
    let c = task(root, "C", "to_do");
    let d = task(root, "D", "to_do");
    let e = task(root, "E", "review");
    let f = task(root, "F", "to_do");
    depends(root, &c, &a);
    depends(root, &d, &b);
    depends(root, &f, &e);
    available(root, &[&a, &b, &e]);
    data(
        root,
        &["task", "archive", &e, "--reason", "obsolete", "--json"],
    );
    available(root, &[&a, &b, &f]);
    data(
        root,
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
    available(root, &[&c, &d, &f]);
    let claimed = data(root, &["task", "claim", &c, "--agent", &agent, "--json"]);
    let original_claim = claimed["claim"].clone();
    assert_eq!(original_claim["agent"]["id"], agent);
    assert!(original_claim["claimedAt"].as_str().is_some());
    available(root, &[&d, &f]);
    data(
        root,
        &[
            "status",
            "set-completed",
            "review",
            "--completed",
            "false",
            "--yes",
            "--json",
        ],
    );
    available(root, &[&a, &b, &f]);
    let blocked = data(root, &["task", "blockers", &c, "--json"]);
    assert_eq!(blocked["claim"], original_claim);
    assert_eq!(blocked["unresolvedDependencies"]["direct"][0]["id"], a);
    data(
        root,
        &["task", "archive", &a, "--reason", "obsolete", "--json"],
    );
    available(root, &[&b, &f]);
    assert_eq!(
        data(root, &["task", "blockers", &c, "--json"])["claim"],
        original_claim
    );
    data(root, &["task", "unarchive", &a, "--yes", "--json"]);
    available(root, &[&a, &b, &f]);
    assert_eq!(
        data(root, &["task", "blockers", &c, "--json"])["claim"],
        original_claim
    );
    data(root, &["task", "claim", &a, "--agent", &agent, "--json"]);
    data(
        root,
        &[
            "task", "archive", &a, "--reason", "obsolete", "--force", "--yes", "--json",
        ],
    );
    assert!(data(root, &["task", "blockers", &a, "--json"])["claim"].is_null());
    data(root, &["task", "unarchive", &a, "--yes", "--json"]);
    assert!(data(root, &["task", "blockers", &a, "--json"])["claim"].is_null());
    available(root, &[&a, &b, &f]);
}
