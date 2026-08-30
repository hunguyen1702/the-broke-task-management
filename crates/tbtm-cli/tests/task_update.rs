use serde_json::Value;
use std::process::Command;
use tempfile::tempdir;
use uuid::Uuid;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn setup(current: &std::path::Path) -> String {
    assert!(
        tbtm(current, &["init", "--prefix", "project", "--json"])
            .status
            .success()
    );
    let output = tbtm(
        current,
        &[
            "task",
            "create",
            "--title",
            "Original",
            "--type",
            "task",
            "--estimate",
            "1",
            "--tag",
            "old",
            "--url",
            "https://old.example",
            "--code-ref",
            "src/old.rs:1",
            "--json",
        ],
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn replaces_and_clears_structured_context_in_order() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    let output = tbtm(
        temp.path(),
        &[
            "task",
            "update",
            &id,
            "--estimate",
            "0",
            "--tag",
            " first ",
            "--tag",
            "Second",
            "--url",
            "https://example.com/a",
            "--url",
            "http://example.com/b",
            "--code-ref",
            "src/a.rs:2-4::A",
            "--code-ref",
            "missing/file.rs",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"]["estimate"], 0.0);
    assert_eq!(
        value["data"]["tags"],
        serde_json::json!(["first", "Second"])
    );
    assert_eq!(
        value["data"]["externalUrls"],
        serde_json::json!(["https://example.com/a", "http://example.com/b"])
    );
    assert_eq!(
        value["data"]["codeReferences"][1]["path"],
        "missing/file.rs"
    );
    let cleared: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &[
                "task",
                "update",
                &id,
                "--clear-estimate",
                "--clear-tags",
                "--clear-urls",
                "--clear-code-refs",
                "--json",
            ],
        )
        .stdout,
    )
    .unwrap();
    assert!(cleared["data"]["estimate"].is_null());
    assert_eq!(cleared["data"]["tags"], serde_json::json!([]));
    assert_eq!(cleared["data"]["externalUrls"], serde_json::json!([]));
    assert_eq!(cleared["data"]["codeReferences"], serde_json::json!([]));
}

#[test]
fn identical_patch_preserves_metadata_and_agent_change_is_attributed() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &id, "--json"]).stdout)
            .unwrap();
    let same: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &[
                "task",
                "update",
                &id,
                "--estimate",
                "1",
                "--tag",
                "old",
                "--json",
            ],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(same["data"]["updatedAt"], before["data"]["updatedAt"]);
    let registered: Value = serde_json::from_slice(
        &tbtm(temp.path(), &["agent", "register", "worker", "--json"]).stdout,
    )
    .unwrap();
    let agent = registered["data"]["id"].as_str().unwrap();
    let changed: Value = serde_json::from_slice(
        &tbtm(
            temp.path(),
            &[
                "task",
                "update",
                &id,
                "--estimate",
                "2",
                "--agent",
                agent,
                "--json",
            ],
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(changed["data"]["updatedBy"], agent);
    assert_eq!(changed["data"]["tags"], serde_json::json!(["old"]));
}

#[test]
fn failures_have_stable_codes_and_leave_context_unchanged() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    for (arguments, code, exit) in [
        (vec!["task", "update", &id, "--json"], "NO_UPDATE_FIELDS", 2),
        (
            vec!["task", "update", &id, "--estimate", "NaN", "--json"],
            "INVALID_ESTIMATE",
            2,
        ),
        (
            vec![
                "task",
                "update",
                &id,
                "--tag",
                "x",
                "--clear-tags",
                "--json",
            ],
            "CONFLICTING_ARGUMENTS",
            2,
        ),
        (
            vec![
                "task", "update", &id, "--tag", "x", "--tag", " x ", "--json",
            ],
            "DUPLICATE_TASK_CONTEXT",
            2,
        ),
        (
            vec!["task", "update", &id, "--url", "file:///x", "--json"],
            "INVALID_EXTERNAL_URL",
            2,
        ),
        (
            vec!["task", "update", &id, "--code-ref", "../x:1", "--json"],
            "INVALID_CODE_REFERENCE",
            2,
        ),
        (
            vec!["task", "update", "missing", "--clear-tags", "--json"],
            "TASK_NOT_FOUND",
            3,
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
            code
        );
    }
    let after: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &id, "--json"]).stdout)
            .unwrap();
    assert_eq!(after["data"]["estimate"], 1.0);
    assert_eq!(after["data"]["tags"], serde_json::json!(["old"]));
}

#[test]
fn patches_all_scalar_fields_and_preserves_structured_context() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    let output = tbtm(
        temp.path(),
        &[
            "task",
            "update",
            &id,
            "--title",
            "  Updated  ",
            "--description",
            "",
            "--goal",
            "Ship it",
            "--acceptance-criteria",
            "Green tests",
            "--type",
            "bug",
            "--status",
            "in_progress",
            "--priority",
            "1000000",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let task: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(task["data"]["title"], "Updated");
    assert_eq!(task["data"]["description"], "");
    assert_eq!(task["data"]["goal"], "Ship it");
    assert_eq!(task["data"]["acceptanceCriteria"], "Green tests");
    assert_eq!(task["data"]["type"], "bug");
    assert_eq!(task["data"]["status"]["code"], "in_progress");
    assert_eq!(task["data"]["priority"], 1_000_000);
    assert_eq!(task["data"]["estimate"], 1.0);
    assert_eq!(task["data"]["tags"], serde_json::json!(["old"]));
    assert_eq!(task["data"]["createdBy"], "user");
}

#[test]
fn scalar_no_op_validates_inputs_and_preserves_metadata() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    let before: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &id, "--json"]).stdout)
            .unwrap();
    let same = tbtm(
        temp.path(),
        &[
            "task",
            "update",
            &id,
            "--title",
            " Original ",
            "--type",
            "task",
            "--status",
            "to_do",
            "--priority",
            "50",
            "--json",
        ],
    );
    assert!(same.status.success());
    let after: Value = serde_json::from_slice(&same.stdout).unwrap();
    assert_eq!(after["data"]["updatedAt"], before["data"]["updatedAt"]);
    assert_eq!(after["data"]["updatedBy"], before["data"]["updatedBy"]);

    let missing_agent = Uuid::new_v4().to_string();
    let invalid = tbtm(
        temp.path(),
        &[
            "task",
            "update",
            &id,
            "--title",
            "Original",
            "--agent",
            &missing_agent,
            "--json",
        ],
    );
    assert_eq!(invalid.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid.stdout).unwrap()["error"]["code"],
        "AGENT_NOT_FOUND"
    );
}

#[test]
fn scalar_errors_and_hierarchy_validation_are_atomic() {
    let temp = tempdir().unwrap();
    let id = setup(temp.path());
    for (arguments, code, exit) in [
        (
            vec!["task", "update", &id, "--title", "   ", "--json"],
            "INVALID_TASK_TITLE",
            2,
        ),
        (
            vec!["task", "update", &id, "--priority", "1000001", "--json"],
            "INVALID_TASK_PRIORITY",
            2,
        ),
        (
            vec!["task", "update", &id, "--status", "missing", "--json"],
            "STATUS_NOT_FOUND",
            3,
        ),
        (
            vec!["task", "update", &id, "--type", "feature", "--json"],
            "INVALID_TASK_TYPE",
            2,
        ),
    ] {
        let output = tbtm(temp.path(), &arguments);
        assert_eq!(output.status.code(), Some(exit));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
            code
        );
    }

    let epic = create_task(temp.path(), "Epic", "epic");
    let story = create_task(temp.path(), "Story", "story");
    assert!(
        tbtm(
            temp.path(),
            &["task", "parent", "set", &story, "--parent", &epic, "--json"]
        )
        .status
        .success()
    );
    let rejected = tbtm(
        temp.path(),
        &["task", "update", &story, "--type", "epic", "--json"],
    );
    assert_eq!(rejected.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&rejected.stdout).unwrap()["error"]["code"],
        "INVALID_TASK_HIERARCHY"
    );
    let unchanged: Value =
        serde_json::from_slice(&tbtm(temp.path(), &["task", "view", &story, "--json"]).stdout)
            .unwrap();
    assert_eq!(unchanged["data"]["type"], "story");

    assert!(
        tbtm(
            temp.path(),
            &["task", "archive", &id, "--reason", "obsolete", "--json"]
        )
        .status
        .success()
    );
    let archived = tbtm(
        temp.path(),
        &["task", "update", &id, "--title", "Changed", "--json"],
    );
    assert_eq!(archived.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&archived.stdout).unwrap()["error"]["code"],
        "TASK_ARCHIVED"
    );
}

fn create_task(current: &std::path::Path, title: &str, task_type: &str) -> String {
    let output = tbtm(
        current,
        &[
            "task", "create", "--title", title, "--type", task_type, "--json",
        ],
    );
    assert!(output.status.success());
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}
