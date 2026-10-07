use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tempfile::tempdir;

fn run(root: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    response["data"].clone()
}

fn recovery_script(path: &Path) {
    let doc = include_str!("../../../docs/recovery.md");
    let script = doc
        .split("```python\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    fs::write(path, script).unwrap();
}

fn recover(script: &Path, args: &[&Path]) -> std::process::Output {
    Command::new("python3")
        .arg(script)
        .args(args)
        .env("TBTM_BIN", env!("CARGO_BIN_EXE_tbtm"))
        .output()
        .unwrap()
}

#[test]
fn documented_offline_backup_restores_saved_state_and_survives_uninstall() {
    let temp = tempdir().unwrap();
    let main = temp.path().join("main");
    let linked = temp.path().join("linked");
    let external = temp.path().join("external");
    fs::create_dir(&main).unwrap();
    fs::create_dir(&external).unwrap();
    for args in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Test"],
        vec!["commit", "--allow-empty", "--quiet", "-m", "initial"],
    ] {
        assert!(
            Command::new("git")
                .current_dir(&main)
                .args(args)
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
    run(&main, &["init", "--prefix", "saved", "--json"]);
    let agent = run(&main, &["agent", "register", "recovery-agent", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let parent = run(
        &main,
        &[
            "task",
            "create",
            "--title",
            "Saved parent",
            "--type",
            "epic",
            "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let child = run(
        &main,
        &[
            "task",
            "create",
            "--title",
            "Saved child",
            "--type",
            "task",
            "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let upstream = run(
        &main,
        &[
            "task",
            "create",
            "--title",
            "Saved upstream",
            "--type",
            "task",
            "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    run(
        &main,
        &[
            "task", "parent", "set", &child, "--parent", &parent, "--json",
        ],
    );
    run(
        &main,
        &[
            "task",
            "dependency",
            "add",
            &child,
            "--depends-on",
            &upstream,
            "--json",
        ],
    );
    run(
        &main,
        &["task", "claim", &upstream, "--agent", &agent, "--json"],
    );
    let saved_status = run(&linked, &["repo", "status", "--json"]);
    let saved_agents = run(&linked, &["agent", "list", "--json"]);
    let saved_child = run(&linked, &["task", "view", &child, "--json"]);
    let saved_upstream = run(&linked, &["task", "view", &upstream, "--json"]);

    let script = temp.path().join("recovery.py");
    recovery_script(&script);
    let backup = external.join("backup.tbtm");
    let displaced = external.join("displaced.tbtm");
    let linked_backup = linked.join("backup.tbtm");
    let refused = recover(&script, &[Path::new("backup"), &linked, &linked_backup]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("outside every repository worktree"));
    assert!(!linked_backup.exists());
    let output = recover(&script, &[Path::new("backup"), &linked, &backup]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !recover(&script, &[Path::new("backup"), &linked, &backup])
            .status
            .success()
    );
    fs::create_dir(&linked_backup).unwrap();
    fs::copy(
        backup.join("config.json"),
        linked_backup.join("config.json"),
    )
    .unwrap();
    fs::copy(backup.join("tbtm.db"), linked_backup.join("tbtm.db")).unwrap();
    let refused = recover(
        &script,
        &[Path::new("restore"), &linked, &linked_backup, &displaced],
    );
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("outside every repository worktree"));
    assert!(!displaced.exists());
    fs::remove_dir_all(&linked_backup).unwrap();
    let linked_displaced = linked.join("displaced.tbtm");
    let refused = recover(
        &script,
        &[Path::new("restore"), &linked, &backup, &linked_displaced],
    );
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("outside every repository worktree"));
    assert!(!linked_displaced.exists());

    run(
        &main,
        &[
            "task",
            "update",
            &child,
            "--title",
            "Changed live child",
            "--json",
        ],
    );
    run(&main, &["agent", "register", "live-only-agent", "--json"]);
    assert_ne!(run(&main, &["task", "view", &child, "--json"]), saved_child);
    let output = recover(
        &script,
        &[Path::new("restore"), &linked, &backup, &displaced],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !recover(
            &script,
            &[Path::new("restore"), &linked, &backup, &displaced]
        )
        .status
        .success()
    );
    assert!(backup.join("config.json").is_file());
    assert!(displaced.join("config.json").is_file());
    let displaced_db = rusqlite::Connection::open(displaced.join("tbtm.db")).unwrap();
    let displaced_title: String = displaced_db
        .query_row("SELECT title FROM tasks WHERE id = ?1", [&child], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(displaced_title, "Changed live child");
    let main_status = run(&main, &["repo", "status", "--json"]);
    assert_eq!(main_status["repositoryId"], saved_status["repositoryId"]);
    assert_eq!(main_status["health"], "healthy");
    assert_eq!(run(&linked, &["agent", "list", "--json"]), saved_agents);
    assert_eq!(
        run(&linked, &["task", "view", &child, "--json"]),
        saved_child
    );
    assert_eq!(
        run(&main, &["task", "view", &upstream, "--json"]),
        saved_upstream
    );
    assert_eq!(run(&linked, &["repo", "status", "--json"]), saved_status);

    let output = Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(&linked)
        .args(["uninstall", "--yes", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!main.join(".tbtm").exists());
    assert!(backup.join("tbtm.db").is_file());
    assert!(displaced.join("tbtm.db").is_file());
}
