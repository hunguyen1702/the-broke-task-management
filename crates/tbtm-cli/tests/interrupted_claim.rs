use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
use tempfile::tempdir;

const GATE_DEADLINE: Duration = Duration::from_secs(7);
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
    fn establish(root: &Path, database: &Path, args: &[&str]) -> (Connection, Self) {
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
                    .windows(SENTINEL.len())
                    .any(|bytes| bytes == SENTINEL.as_bytes())
            {
                // The marker lives only in a preexisting task_claims row. Its page in the
                // rollback journal proves this writer changed the claim table before commit.
                return (reader, writer);
            }
            assert!(
                start.elapsed() < GATE_DEADLINE,
                "claim journal marker absent before deadline; journal size: {:?}",
                fs::metadata(journal_for(database)).map(|m| m.len())
            );
            std::thread::yield_now();
        }
    }
}

#[test]
#[ignore = "feasibility: pre-held SHARED reader blocks migration commit before claim write"]
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
