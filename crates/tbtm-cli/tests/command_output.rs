use serde_json::Value;
use std::process::{Command, Output};
use tempfile::TempDir;

fn tbtm(directory: &std::path::Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(directory)
        .args(arguments)
        .output()
        .unwrap()
}

fn json_output(directory: &std::path::Path, arguments: &[&str]) -> Value {
    let output = tbtm(directory, arguments);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn global_json_is_inherited_idempotently_by_every_top_level_family() {
    let directory = TempDir::new().unwrap();
    assert_eq!(
        json_output(directory.path(), &["--json", "init"])["ok"],
        true
    );

    for arguments in [
        vec!["--json", "repo", "status"],
        vec!["agent", "--json", "list"],
        vec!["status", "list", "--json"],
        vec!["--json", "task", "--json", "list", "--json"],
    ] {
        let response = json_output(directory.path(), &arguments);
        assert_eq!(response["ok"], true);
        assert!(response["data"].is_object() || response["data"].is_array());
        assert!(response["error"].is_null());
    }
}

#[test]
fn exact_json_token_owns_parse_errors_but_typos_use_clap_output() {
    let directory = TempDir::new().unwrap();
    let json = tbtm(directory.path(), &["task", "wat", "--json"]);
    assert_eq!(json.status.code(), Some(2));
    assert!(json.stderr.is_empty());
    let response: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "INVALID_ARGUMENTS");

    let human = tbtm(directory.path(), &["task", "wat", "--jsoon"]);
    assert_eq!(human.status.code(), Some(2));
    assert!(human.stdout.is_empty());
    assert!(!human.stderr.is_empty());
}

#[test]
fn help_stays_human_and_noninteractive_json_never_prompts() {
    let directory = TempDir::new().unwrap();
    let help = tbtm(directory.path(), &["--json", "task", "--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("Manage repository-local tasks"));

    let confirmation = tbtm(directory.path(), &["uninstall", "--json"]);
    assert_eq!(confirmation.status.code(), Some(2));
    assert!(confirmation.stderr.is_empty());
    let response: Value = serde_json::from_slice(&confirmation.stdout).unwrap();
    assert_eq!(response["error"]["code"], "CONFIRMATION_REQUIRED");
    assert!(!String::from_utf8_lossy(&confirmation.stdout).contains("[y/N]"));
}
