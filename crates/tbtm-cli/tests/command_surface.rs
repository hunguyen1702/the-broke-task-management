use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tempfile::{TempDir, tempdir};

fn tbtm(current: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn successful_text(current: &Path, arguments: &[&str]) -> String {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "args={arguments:?} stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "args={arguments:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn successful_json(current: &Path, arguments: &[&str]) -> Value {
    let text = successful_text(current, arguments);
    let response: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(response["ok"], true, "args={arguments:?}");
    assert!(response["error"].is_null(), "args={arguments:?}");
    response["data"].clone()
}

fn assert_contains_all(text: &str, expected: &[&str], context: &[&str]) {
    for value in expected {
        assert!(
            text.contains(value),
            "missing {value:?} for {context:?} in:\n{text}"
        );
    }
}

#[test]
fn owned_commands_and_options_are_discoverable_from_help() {
    let current = tempdir().unwrap();
    for (arguments, expected) in [
        (
            vec!["--help"],
            vec!["init", "uninstall", "repo", "agent", "task", "--json"],
        ),
        (vec!["repo", "--help"], vec!["status", "--json"]),
        (vec!["agent", "--help"], vec!["register", "list", "--json"]),
        (
            vec!["task", "--help"],
            vec![
                "create",
                "view",
                "list",
                "update",
                "archive",
                "unarchive",
                "--json",
            ],
        ),
        (
            vec!["task", "create", "--help"],
            vec![
                "--title <TITLE>",
                "--type <TASK_TYPE>",
                "--tag <TAGS>",
                "--url <URLS>",
                "--code-ref <CODE_REFERENCES>",
                "--agent <AGENT>",
                "--json",
            ],
        ),
        (
            vec!["task", "list", "--help"],
            vec![
                "--archived",
                "--all",
                "--status",
                "--type",
                "--tag",
                "--json",
            ],
        ),
        (
            vec!["task", "update", "--help"],
            vec![
                "--clear-estimate",
                "--clear-tags",
                "--clear-urls",
                "--clear-code-refs",
                "--agent <AGENT>",
                "--json",
            ],
        ),
        (
            vec!["task", "archive", "--help"],
            vec![
                "--reason <REASON>",
                "--agent <AGENT>",
                "--force",
                "--yes",
                "--json",
            ],
        ),
        (
            vec!["task", "unarchive", "--help"],
            vec!["<ID>", "--yes", "--json"],
        ),
    ] {
        let help = successful_text(current.path(), &arguments);
        assert_contains_all(&help, &expected, &arguments);
    }
}

#[test]
fn actor_option_is_limited_to_authorized_task_mutations() {
    let current = tempdir().unwrap();
    for arguments in [
        vec![
            "repo",
            "status",
            "--agent",
            "00000000-0000-0000-0000-000000000000",
        ],
        vec![
            "agent",
            "list",
            "--agent",
            "00000000-0000-0000-0000-000000000000",
        ],
        vec![
            "task",
            "view",
            "TASK-1",
            "--agent",
            "00000000-0000-0000-0000-000000000000",
        ],
        vec![
            "task",
            "list",
            "--agent",
            "00000000-0000-0000-0000-000000000000",
        ],
        vec![
            "task",
            "unarchive",
            "TASK-1",
            "--agent",
            "00000000-0000-0000-0000-000000000000",
        ],
    ] {
        let output = tbtm(current.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "args={arguments:?}");
        assert!(output.stdout.is_empty(), "args={arguments:?}");
        assert_contains_all(
            &String::from_utf8(output.stderr).unwrap(),
            &["unexpected argument '--agent'"],
            &arguments,
        );
    }
    assert!(!current.path().join(".tbtm").exists());
}

fn linked_worktree() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
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
fn linked_worktree_runs_the_owned_lifecycle_against_the_canonical_store() {
    let (_temp, main, linked) = linked_worktree();
    successful_json(&main, &["--json", "init", "--prefix", "surface"]);

    let status = successful_json(&linked, &["repo", "--json", "status"]);
    assert_eq!(status["prefix"], "surface");
    assert_eq!(
        status["repositoryRoot"],
        main.canonicalize().unwrap().to_string_lossy().as_ref()
    );

    let agent = successful_json(&linked, &["agent", "register", "surface-auditor", "--json"]);
    let agent_id = agent["id"].as_str().unwrap();
    let agents = successful_json(&main, &["--json", "agent", "list"]);
    assert_eq!(agents["agents"].as_array().unwrap().len(), 1);

    let created = successful_json(
        &linked,
        &[
            "task",
            "create",
            "--title",
            "Audit surface",
            "--type",
            "task",
            "--tag",
            "cli",
            "--agent",
            agent_id,
            "--json",
        ],
    );
    let task_id = created["id"].as_str().unwrap();
    assert_eq!(created["createdBy"], agent_id);

    let viewed = successful_json(&main, &["task", "view", task_id, "--json"]);
    assert_eq!(viewed["title"], "Audit surface");
    let updated = successful_json(
        &main,
        &[
            "--json",
            "task",
            "update",
            task_id,
            "--title",
            "Audited surface",
            "--goal",
            "Protect CLI",
            "--tag",
            "cli",
            "--tag",
            "regression",
            "--agent",
            agent_id,
        ],
    );
    assert_eq!(updated["updatedBy"], agent_id);

    let listed = successful_json(&linked, &["task", "list", "--tag", "regression", "--json"]);
    assert_eq!(listed.as_array().unwrap().len(), 1);
    successful_json(
        &linked,
        &[
            "task",
            "archive",
            task_id,
            "--reason",
            "lifecycle regression",
            "--agent",
            agent_id,
            "--json",
        ],
    );
    let archived = successful_json(&main, &["task", "list", "--archived", "--json"]);
    assert_eq!(archived.as_array().unwrap().len(), 1);
    successful_json(&main, &["task", "unarchive", task_id, "--yes", "--json"]);

    let active = successful_json(&linked, &["task", "list", "--json"]);
    assert_eq!(active[0]["id"], task_id);
    assert!(!linked.join(".tbtm").exists());
    assert!(main.join(".tbtm/tbtm.db").is_file());
}
