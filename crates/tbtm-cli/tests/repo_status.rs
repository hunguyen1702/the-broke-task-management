use serde_json::Value;
use std::{fs, process::Command};
use tempfile::tempdir;

fn tbtm(current: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn assert_recovery_failure(
    root: &std::path::Path,
    exit: i32,
    code: &str,
    check: Option<&str>,
    guidance: &str,
) {
    let workspace = root.join(".tbtm");
    let config = workspace.join("config.json");
    let database = workspace.join("tbtm.db");
    let original_config = fs::read(&config).ok();
    let original_database = fs::read(&database).ok();

    let json = tbtm(root, &["repo", "status", "--json"]);
    assert_eq!(
        json.status.code(),
        Some(exit),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&json.stdout),
        String::from_utf8_lossy(&json.stderr)
    );
    assert!(
        json.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let response: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(response["error"]["code"], code);
    assert!(
        response["error"]["details"]["suggestion"]
            .as_str()
            .unwrap()
            .contains(guidance)
    );
    if let Some(check) = check {
        assert_eq!(response["error"]["details"]["check"], check);
        assert!(response["error"]["details"]["path"].as_str().is_some());
    }

    let human = tbtm(root, &["repo", "status"]);
    assert_eq!(
        human.status.code(),
        Some(exit),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&human.stdout),
        String::from_utf8_lossy(&human.stderr)
    );
    assert!(human.stdout.is_empty());
    let stderr = String::from_utf8(human.stderr).unwrap();
    assert!(
        stderr.contains(code) && stderr.contains("Next step:") && stderr.contains(guidance),
        "{stderr}"
    );
    assert_eq!(fs::read(&config).ok(), original_config);
    assert_eq!(fs::read(&database).ok(), original_database);
}

fn initialize(current: &std::path::Path) {
    let output = tbtm(current, &["init", "--prefix", "stored-prefix", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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
                linked.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    (temp, main, linked)
}

#[test]
fn linked_worktree_uses_only_main_worktree_store() {
    let (_temp, main, linked) = linked_worktree();
    let local_workspace = linked.join(".tbtm");
    fs::create_dir(&local_workspace).unwrap();
    fs::write(local_workspace.join("sentinel"), b"linked-local").unwrap();

    let init = tbtm(
        &linked,
        &["init", "--prefix", "shared", "--stealth", "--json"],
    );
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(main.join(".tbtm/config.json").is_file());
    assert!(!linked.join(".gitignore").exists());
    assert_eq!(
        fs::read(linked.join(".tbtm/sentinel")).unwrap(),
        b"linked-local"
    );
    assert!(
        fs::read_to_string(main.join(".gitignore"))
            .unwrap()
            .contains("/.tbtm/")
    );

    let status = tbtm(&linked, &["repo", "status", "--json"]);
    assert!(status.status.success());
    let response: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        response["data"]["repositoryRoot"],
        main.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    assert_eq!(
        response["data"]["worktreeRoot"],
        linked.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    assert_eq!(response["data"]["prefix"], "shared");

    let uninstall = tbtm(&linked, &["uninstall", "--dry-run", "--json"]);
    assert!(uninstall.status.success());
    let response: Value = serde_json::from_slice(&uninstall.stdout).unwrap();
    let planned = response["data"]["planned"].as_array().unwrap();
    assert!(planned.iter().any(|path| {
        path.as_str()
            == Some(
                main.join(".tbtm")
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            )
    }));
    assert!(planned.iter().all(|path| {
        !path
            .as_str()
            .unwrap()
            .starts_with(linked.to_string_lossy().as_ref())
    }));
    assert_eq!(
        fs::read(linked.join(".tbtm/sentinel")).unwrap(),
        b"linked-local"
    );
}

#[test]
fn bare_git_repository_is_unavailable_without_mutation() {
    let temp = tempdir().unwrap();
    assert!(
        Command::new("git")
            .current_dir(temp.path())
            .args(["init", "--bare", "--quiet"])
            .status()
            .unwrap()
            .success()
    );

    let output = tbtm(temp.path(), &["init", "--json"]);

    assert_eq!(output.status.code(), Some(1));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "REPOSITORY_UNAVAILABLE");
    assert_eq!(
        response["error"]["details"]["phase"],
        "current worktree resolution"
    );
    assert!(!temp.path().join(".tbtm").exists());
}

#[test]
fn repo_status_json_reports_healthy_repository_from_nested_directory() {
    let temp = tempdir().unwrap();
    let git = Command::new("git")
        .current_dir(temp.path())
        .args(["init", "--quiet"])
        .status()
        .unwrap();
    assert!(git.success());
    initialize(temp.path());
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let output = tbtm(&nested, &["repo", "status", "--json"]);

    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let canonical_root = temp.path().canonicalize().unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["data"]["prefix"], "stored-prefix");
    assert_eq!(response["data"]["schemaVersion"], 1);
    assert_eq!(response["data"]["health"], "healthy");
    assert_eq!(
        response["data"]["repositoryRoot"],
        canonical_root.to_string_lossy().as_ref()
    );
    assert_eq!(
        response["data"]["worktreeRoot"],
        canonical_root.to_string_lossy().as_ref()
    );
    assert_eq!(response["error"], Value::Null);
}

#[test]
fn repo_status_human_output_is_concise() {
    let temp = tempdir().unwrap();
    initialize(temp.path());

    let output = tbtm(temp.path(), &["repo", "status"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for field in [
        "Repository root:",
        "Worktree root:",
        "Config:",
        "Database:",
        "Repository ID:",
        "Prefix: stored-prefix",
        "Schema version: 1",
        "Health: healthy",
    ] {
        assert!(stdout.contains(field), "missing {field:?} in {stdout:?}");
    }
}

#[test]
fn repo_status_human_error_includes_recovery_direction() {
    let temp = tempdir().unwrap();

    let output = tbtm(temp.path(), &["repo", "status"]);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("REPOSITORY_NOT_INITIALIZED"));
    assert!(stderr.contains("Next step: Run `tbtm init` from the repository root."));
}

#[cfg(unix)]
#[test]
fn repo_status_unreadable_database_exits_five() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    initialize(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let original_database = fs::read(&database).unwrap();
    let original_config = fs::read(temp.path().join(".tbtm/config.json")).unwrap();
    let original_permissions = fs::metadata(&database).unwrap().permissions();
    fs::set_permissions(&database, fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(
        fs::File::open(&database).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied,
        "permission fixture must deny access"
    );

    let output = tbtm(temp.path(), &["repo", "status", "--json"]);
    let human = tbtm(temp.path(), &["repo", "status"]);

    fs::set_permissions(&database, original_permissions).unwrap();
    assert_eq!(output.status.code(), Some(5));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "PERMISSION_DENIED");
    assert_eq!(response["error"]["details"]["check"], "database open");
    assert!(
        response["error"]["details"]["suggestion"]
            .as_str()
            .unwrap()
            .contains("Grant access")
    );
    assert!(output.stderr.is_empty());
    assert_eq!(human.status.code(), Some(5));
    assert!(human.stdout.is_empty());
    let stderr = String::from_utf8(human.stderr).unwrap();
    assert!(stderr.contains("PERMISSION_DENIED") && stderr.contains("Next step: Grant access"));
    assert_eq!(fs::read(&database).unwrap(), original_database);
    assert_eq!(
        fs::read(temp.path().join(".tbtm/config.json")).unwrap(),
        original_config
    );
}

#[cfg(windows)]
#[test]
fn repo_status_windows_acl_denial_exits_five() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let database = temp.path().join(".tbtm/tbtm.db");
    let identity_output = Command::new("whoami").output().unwrap();
    assert!(identity_output.status.success());
    let identity = String::from_utf8(identity_output.stdout)
        .unwrap()
        .trim()
        .to_owned();
    let deny_rule = format!("{identity}:(R)");
    let deny = Command::new("icacls")
        .arg(&database)
        .args(["/deny", deny_rule.as_str()])
        .output()
        .unwrap();
    assert!(
        deny.status.success(),
        "{}",
        String::from_utf8_lossy(&deny.stderr)
    );

    let output = tbtm(temp.path(), &["repo", "status", "--json"]);

    let restore = Command::new("icacls")
        .arg(&database)
        .args(["/remove:d", identity.as_str()])
        .output()
        .unwrap();
    assert!(
        restore.status.success(),
        "{}",
        String::from_utf8_lossy(&restore.stderr)
    );
    assert_eq!(output.status.code(), Some(5));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "PERMISSION_DENIED");
}

#[test]
fn repo_status_classifies_uninitialized_invalid_config_and_missing_database() {
    let uninitialized = tempdir().unwrap();
    let output = tbtm(uninitialized.path(), &["repo", "status", "--json"]);
    assert_eq!(output.status.code(), Some(3));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "REPOSITORY_NOT_INITIALIZED");
    assert_eq!(response["error"]["details"]["check"], "workspace");

    let invalid = tempdir().unwrap();
    initialize(invalid.path());
    fs::write(invalid.path().join(".tbtm/config.json"), b"not json").unwrap();
    let output = tbtm(invalid.path(), &["repo", "status", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "INVALID_CONFIGURATION");
    assert_eq!(response["error"]["details"]["check"], "config parse");

    let missing_database = tempdir().unwrap();
    initialize(missing_database.path());
    let database = missing_database.path().join(".tbtm/tbtm.db");
    fs::remove_file(&database).unwrap();
    let output = tbtm(missing_database.path(), &["repo", "status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "DATABASE_UNAVAILABLE");
    assert!(!database.exists());
}

#[test]
fn repo_status_rejects_unsafe_database_value_before_access() {
    let temp = tempdir().unwrap();
    initialize(temp.path());
    let config_path = temp.path().join(".tbtm/config.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["database"] = Value::String("../outside.db".to_owned());
    fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

    let output = tbtm(temp.path(), &["repo", "status", "--json"]);

    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "INVALID_CONFIGURATION");
    assert_eq!(response["error"]["details"]["check"], "database");
    assert!(!temp.path().join("outside.db").exists());
}

#[test]
fn repository_failures_give_recovery_steps_without_repairing_data() {
    let bare = tempdir().unwrap();
    assert!(
        Command::new("git")
            .current_dir(bare.path())
            .args(["init", "--bare", "--quiet"])
            .status()
            .unwrap()
            .success()
    );
    assert_recovery_failure(
        bare.path(),
        1,
        "REPOSITORY_UNAVAILABLE",
        None,
        "Git worktree metadata",
    );

    let missing = tempdir().unwrap();
    assert_recovery_failure(
        missing.path(),
        3,
        "REPOSITORY_NOT_INITIALIZED",
        Some("workspace"),
        "tbtm init",
    );

    let malformed = tempdir().unwrap();
    initialize(malformed.path());
    fs::write(malformed.path().join(".tbtm/config.json"), b"not json").unwrap();
    assert_recovery_failure(
        malformed.path(),
        2,
        "INVALID_CONFIGURATION",
        Some("config parse"),
        "matching config and database",
    );

    let unsafe_path = tempdir().unwrap();
    initialize(unsafe_path.path());
    let config_path = unsafe_path.path().join(".tbtm/config.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["database"] = Value::String("../outside.db".into());
    fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    assert_recovery_failure(
        unsafe_path.path(),
        2,
        "INVALID_CONFIGURATION",
        Some("database"),
        "matching config and database",
    );
    assert!(!unsafe_path.path().join("outside.db").exists());

    let mismatched = tempdir().unwrap();
    initialize(mismatched.path());
    let config_path = mismatched.path().join(".tbtm/config.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["repositoryId"] = Value::String("00000000-0000-0000-0000-000000000000".into());
    fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    assert_recovery_failure(
        mismatched.path(),
        2,
        "INVALID_CONFIGURATION",
        Some("config/database metadata agreement"),
        "matching config and database",
    );

    let missing_db = tempdir().unwrap();
    initialize(missing_db.path());
    fs::remove_file(missing_db.path().join(".tbtm/tbtm.db")).unwrap();
    assert_recovery_failure(
        missing_db.path(),
        1,
        "DATABASE_UNAVAILABLE",
        Some("database access"),
        "valid database from backup",
    );

    let invalid_db = tempdir().unwrap();
    initialize(invalid_db.path());
    fs::write(invalid_db.path().join(".tbtm/tbtm.db"), b"not sqlite").unwrap();
    assert_recovery_failure(
        invalid_db.path(),
        1,
        "DATABASE_UNAVAILABLE",
        Some("database validation"),
        "valid database from backup",
    );
}
