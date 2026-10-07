use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
use tempfile::tempdir;

const GATE_DEADLINE: Duration = Duration::from_secs(3);
const SENTINEL: &str = "2026-01-01T00:00:00Z-claim-journal-marker";

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
        "{args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

fn task(root: &Path, title: &str) -> String {
    data(
        root,
        &[
            "task", "create", "--title", title, "--type", "task", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn story(root: &Path, title: &str) -> String {
    data(
        root,
        &[
            "task", "create", "--title", title, "--type", "story", "--json",
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn journal_for(database: &Path) -> PathBuf {
    database.with_file_name(format!(
        "{}-journal",
        database.file_name().unwrap().to_string_lossy()
    ))
}

struct BlockedWriter {
    child: Option<Child>,
}

impl Drop for BlockedWriter {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl BlockedWriter {
    fn establish(root: &Path, database: &Path, args: &[&str], marker: &str) -> (Connection, Self) {
        let reader =
            Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        reader
            .execute_batch("BEGIN; SELECT 1 FROM task_claims LIMIT 1;")
            .unwrap();
        // A deferred BEGIN is not enough: the query above acquires the SHARED lock.
        assert!(!journal_for(database).exists());
        let child = Command::new(env!("CARGO_BIN_EXE_tbtm"))
            .current_dir(root)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut writer = Self { child: Some(child) };
        let start = Instant::now();
        loop {
            if let Some(status) = writer.child.as_mut().unwrap().try_wait().unwrap() {
                let output = writer.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "writer exited before mutation gate: {status}; command: {args:?}; stdout: {}; stderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            if let Ok(journal) = fs::read(journal_for(database))
                && journal
                    .windows(marker.len())
                    .any(|bytes| bytes == marker.as_bytes())
            {
                // The marker lives in a preexisting row of the mutated table. Its page in
                // the rollback journal proves this writer reached that table before commit.
                return (reader, writer);
            }
            assert!(
                start.elapsed() < GATE_DEADLINE,
                "mutation journal marker {marker:?} absent before deadline for {args:?}; journal size: {:?}",
                fs::metadata(journal_for(database)).map(|m| m.len())
            );
            std::thread::yield_now();
        }
    }
}

#[test]
fn killed_claim_writer_preserves_original_claims_and_repository_health() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let existing = task(root, "Already owned");
    let target = task(root, "New claim target");
    let unrelated = task(root, "Unrelated");
    let owner = data(root, &["agent", "register", "owner", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let contender = data(root, &["agent", "register", "contender", "--json"])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let database = root.join(".tbtm/tbtm.db");
    let connection = Connection::open(&database).unwrap();
    let mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "delete");
    connection
        .execute(
            "INSERT INTO task_claims (task_id, agent_id, claimed_at) VALUES (?1, ?2, ?3)",
            (&existing, &owner, SENTINEL),
        )
        .unwrap();
    drop(connection);
    let before: Vec<Value> = [&existing, &target, &unrelated]
        .iter()
        .map(|id| data(root, &["task", "view", id, "--json"]))
        .collect();
    let (reader, writer) = BlockedWriter::establish(
        root,
        &database,
        &["task", "claim", &target, "--agent", &contender, "--json"],
        SENTINEL,
    );
    drop(writer); // kill and reap before releasing the reader lock
    reader.execute_batch("ROLLBACK").unwrap();
    drop(reader);
    // Open the same database writable to allow SQLite to recover any hot journal.
    let connection =
        Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(violations, 0);
    drop(connection);
    let after: Vec<Value> = [&existing, &target, &unrelated]
        .iter()
        .map(|id| data(root, &["task", "view", id, "--json"]))
        .collect();
    assert_eq!(
        after, before,
        "claim, task metadata, and unrelated task changed"
    );
    let health = data(root, &["repo", "status", "--json"]);
    assert_eq!(health["health"], "healthy");
}

fn assert_recovered(root: &Path, database: &Path, ids: &[String], before: &[Value]) {
    let connection =
        Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(violations, 0);
    drop(connection);
    let after: Vec<Value> = ids
        .iter()
        .map(|id| data(root, &["task", "view", id, "--json"]))
        .collect();
    assert_eq!(
        after, before,
        "relationships, claims, or task metadata changed"
    );
    assert_eq!(
        data(root, &["repo", "status", "--json"])["health"],
        "healthy"
    );
}

#[test]
fn killed_dependency_add_writer_preserves_original_edges_and_repository_health() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let downstream = task(root, "Downstream");
    let original_upstream = task(root, "Original upstream");
    let new_upstream = task(root, "New upstream");
    let unrelated = task(root, "Unrelated");
    data(
        root,
        &[
            "task",
            "dependency",
            "add",
            &downstream,
            "--depends-on",
            &original_upstream,
            "--json",
        ],
    );
    let ids = [
        downstream.clone(),
        original_upstream.clone(),
        new_upstream.clone(),
        unrelated,
    ];
    let before: Vec<Value> = ids
        .iter()
        .map(|id| data(root, &["task", "view", id, "--json"]))
        .collect();
    let database = root.join(".tbtm/tbtm.db");
    let (reader, writer) = BlockedWriter::establish(
        root,
        &database,
        &[
            "task",
            "dependency",
            "add",
            &downstream,
            "--depends-on",
            &new_upstream,
            "--json",
        ],
        &original_upstream,
    );
    drop(writer);
    reader.execute_batch("ROLLBACK").unwrap();
    drop(reader);
    assert_recovered(root, &database, &ids, &before);
}

#[test]
fn killed_parent_replace_writer_preserves_original_edge_and_repository_health() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    data(root, &["init", "--prefix", "project", "--json"]);
    let child = task(root, "Child");
    let original_parent = story(root, "Original parent");
    let new_parent = story(root, "New parent");
    let unrelated = task(root, "Unrelated");
    data(
        root,
        &[
            "task",
            "parent",
            "set",
            &child,
            "--parent",
            &original_parent,
            "--json",
        ],
    );
    let ids = [
        child.clone(),
        original_parent.clone(),
        new_parent.clone(),
        unrelated,
    ];
    let before: Vec<Value> = ids
        .iter()
        .map(|id| data(root, &["task", "view", id, "--json"]))
        .collect();
    let database = root.join(".tbtm/tbtm.db");
    let (reader, writer) = BlockedWriter::establish(
        root,
        &database,
        &[
            "task",
            "parent",
            "set",
            &child,
            "--parent",
            &new_parent,
            "--json",
        ],
        &original_parent,
    );
    drop(writer);
    reader.execute_batch("ROLLBACK").unwrap();
    drop(reader);
    assert_recovered(root, &database, &ids, &before);
}
