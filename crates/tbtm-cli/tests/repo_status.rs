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

fn initialize(current: &std::path::Path) {
    let output = tbtm(current, &["init", "--prefix", "stored-prefix", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    assert_eq!(response["ok"], true);
    assert_eq!(response["data"]["prefix"], "stored-prefix");
    assert_eq!(response["data"]["schemaVersion"], 1);
    assert_eq!(response["data"]["health"], "healthy");
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
    let original_permissions = fs::metadata(&database).unwrap().permissions();
    fs::set_permissions(&database, fs::Permissions::from_mode(0o000)).unwrap();

    let output = tbtm(temp.path(), &["repo", "status", "--json"]);

    fs::set_permissions(&database, original_permissions).unwrap();
    assert_eq!(output.status.code(), Some(5));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["error"]["code"], "PERMISSION_DENIED");
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
