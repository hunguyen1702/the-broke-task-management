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
