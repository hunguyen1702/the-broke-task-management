use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
use tempfile::{TempDir, tempdir};

const REPETITIONS: usize = 20;
const GATE_TIMEOUT: Duration = Duration::from_secs(30);
const CHILD_MARKER: &str = "TBTM_GATED_CHILD_RESULT=";

fn tbtm(current: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap()
}

fn git(current: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .current_dir(current)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn success_data(current: &Path, arguments: &[&str]) -> Value {
    let output = tbtm(current, arguments);
    assert!(
        output.status.success(),
        "tbtm {arguments:?} failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
}

struct SharedWorktrees {
    _temp: TempDir,
    main: PathBuf,
    linked_a: PathBuf,
    linked_b: PathBuf,
    database: PathBuf,
    agents: [String; 2],
}

impl SharedWorktrees {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let main = temp.path().join("main");
        let linked_a = temp.path().join("linked-a");
        let linked_b = temp.path().join("linked-b");
        fs::create_dir(&main).unwrap();
        git(&main, &["init", "--quiet"]);
        git(&main, &["config", "user.email", "test@example.com"]);
        git(&main, &["config", "user.name", "Test"]);
        git(
            &main,
            &["commit", "--allow-empty", "--quiet", "-m", "initial"],
        );
        git(
            &main,
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                "linked-a",
                linked_a.to_str().unwrap(),
            ],
        );
        git(
            &main,
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                "linked-b",
                linked_b.to_str().unwrap(),
            ],
        );
        success_data(&main, &["init", "--prefix", "race", "--json"]);
        let agents = [
            success_data(&linked_a, &["agent", "register", "racer-a", "--json"])["id"]
                .as_str()
                .unwrap()
                .to_owned(),
            success_data(&linked_b, &["agent", "register", "racer-b", "--json"])["id"]
                .as_str()
                .unwrap()
                .to_owned(),
        ];
        let database = main.join(".tbtm/tbtm.db").canonicalize().unwrap();
        for root in [&main, &linked_a, &linked_b] {
            let status = success_data(root, &["repo", "status", "--json"]);
            assert_eq!(
                Path::new(status["databasePath"].as_str().unwrap()),
                database
            );
        }
        assert!(!linked_a.join(".tbtm").exists());
        assert!(!linked_b.join(".tbtm").exists());
        Self {
            _temp: temp,
            main,
            linked_a,
            linked_b,
            database,
            agents,
        }
    }

    fn pairings(&self) -> [(&str, [&Path; 2]); 2] {
        [
            ("main-v-linked-a", [&self.main, &self.linked_a]),
            ("linked-a-v-linked-b", [&self.linked_a, &self.linked_b]),
        ]
    }

    fn create(&self, title: &str, tag: &str, priority: i64) -> String {
        success_data(
            &self.main,
            &[
                "task",
                "create",
                "--title",
                title,
                "--type",
                "task",
                "--priority",
                &priority.to_string(),
                "--tag",
                tag,
                "--json",
            ],
        )["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn assert_storage_and_integrity(&self) {
        let connection = Connection::open(&self.database).unwrap();
        let duplicates: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM (SELECT task_id FROM task_claims GROUP BY task_id HAVING COUNT(*) > 1)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(duplicates, 0);
        assert_eq!(integrity, "ok");
        assert!(!self.linked_a.join(".tbtm").exists());
        assert!(!self.linked_b.join(".tbtm").exists());
    }
}

#[derive(Debug)]
struct GatedResult {
    label: String,
    status: i32,
    stdout: String,
    stderr: String,
}

impl GatedResult {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|error| {
            panic!(
                "{} returned invalid JSON: {error}\nstdout: {}\nstderr: {}",
                self.label, self.stdout, self.stderr
            )
        })
    }
}

struct GatedChild {
    label: String,
    ready: PathBuf,
    child: Child,
}

fn launch_gated(
    gate_dir: &Path,
    index: usize,
    label: String,
    current: &Path,
    arguments: &[String],
) -> GatedChild {
    let ready = gate_dir.join(format!("ready-{index}"));
    let child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("gated_tbtm_child")
        .arg("--nocapture")
        .env("TBTM_GATED_CHILD", "1")
        .env("TBTM_GATED_READY", &ready)
        .env("TBTM_GATED_RELEASE", gate_dir.join("release"))
        .env("TBTM_GATED_ROOT", current)
        .env("TBTM_GATED_ARGS", serde_json::to_string(arguments).unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    GatedChild {
        label,
        ready,
        child,
    }
}

fn wait_until(description: &str, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + GATE_TIMEOUT;
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}"
        );
        std::thread::yield_now();
    }
}

fn gated_race(
    gate_owner: &TempDir,
    label: &str,
    roots: [&Path; 2],
    arguments: [Vec<String>; 2],
) -> Vec<GatedResult> {
    let gate_dir = gate_owner.path().join(label);
    fs::create_dir(&gate_dir).unwrap();
    let mut children: Vec<_> = roots
        .into_iter()
        .zip(arguments)
        .enumerate()
        .map(|(index, (root, arguments))| {
            launch_gated(
                &gate_dir,
                index,
                format!("{label}/contender-{index}"),
                root,
                &arguments,
            )
        })
        .collect();
    wait_until(label, || children.iter().all(|child| child.ready.exists()));
    fs::write(gate_dir.join("release"), []).unwrap();
    children
        .drain(..)
        .map(|child| {
            let wrapper = child.child.wait_with_output().unwrap();
            let wrapper_stdout = String::from_utf8_lossy(&wrapper.stdout);
            let payload = wrapper_stdout
                .lines()
                .find_map(|line| line.strip_prefix(CHILD_MARKER))
                .unwrap_or_else(|| {
                    panic!(
                        "{} wrapper failed: {:?}\nstdout: {}\nstderr: {}",
                        child.label,
                        wrapper.status,
                        wrapper_stdout,
                        String::from_utf8_lossy(&wrapper.stderr)
                    )
                });
            let value: Value = serde_json::from_str(payload).unwrap();
            GatedResult {
                label: child.label,
                status: value["status"].as_i64().unwrap() as i32,
                stdout: value["stdout"].as_str().unwrap().to_owned(),
                stderr: value["stderr"].as_str().unwrap().to_owned(),
            }
        })
        .collect()
}

#[test]
fn gated_tbtm_child() {
    if std::env::var_os("TBTM_GATED_CHILD").is_none() {
        return;
    }
    let ready = PathBuf::from(std::env::var_os("TBTM_GATED_READY").unwrap());
    let release = PathBuf::from(std::env::var_os("TBTM_GATED_RELEASE").unwrap());
    let root = PathBuf::from(std::env::var_os("TBTM_GATED_ROOT").unwrap());
    let arguments: Vec<String> =
        serde_json::from_str(&std::env::var("TBTM_GATED_ARGS").unwrap()).unwrap();
    fs::write(&ready, []).unwrap();
    wait_until("parent release", || release.exists());
    let output = Command::new(env!("CARGO_BIN_EXE_tbtm"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap();
    println!(
        "{CHILD_MARKER}{}",
        json!({
            "status": output.status.code().unwrap_or(-1),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        })
    );
}

#[test]
fn repeated_specified_claim_races_preserve_the_exact_winner() {
    let fixture = SharedWorktrees::new();
    let gates = tempdir().unwrap();
    for (pairing, roots) in fixture.pairings() {
        for repetition in 0..REPETITIONS {
            let label = format!("specified-{pairing}-{repetition}");
            let task = fixture.create(&label, &label, 50);
            let arguments = fixture.agents.clone().map(|agent| {
                vec![
                    "task".into(),
                    "claim".into(),
                    task.clone(),
                    "--agent".into(),
                    agent,
                    "--json".into(),
                ]
            });
            let results = gated_race(&gates, &label, roots, arguments);
            let winners: Vec<_> = results.iter().filter(|result| result.status == 0).collect();
            let losers: Vec<_> = results.iter().filter(|result| result.status == 4).collect();
            assert_eq!(winners.len(), 1, "{label}: {results:#?}");
            assert_eq!(losers.len(), 1, "{label}: {results:#?}");
            let winner = winners[0].json();
            let winner_agent = winner["data"]["claim"]["agent"]["id"].as_str().unwrap();
            let claimed_at = winner["data"]["claim"]["claimedAt"].as_str().unwrap();
            let loser = losers[0].json();
            assert_eq!(loser["error"]["code"], "CLAIM_CONFLICT", "{label}");
            assert_eq!(loser["error"]["details"]["agent"]["id"], winner_agent);
            assert_eq!(loser["error"]["details"]["claimedAt"], claimed_at);
            let connection = Connection::open(&fixture.database).unwrap();
            let persisted: (String, String, i64) = connection
                .query_row(
                    "SELECT agent_id, claimed_at, COUNT(*) FROM task_claims WHERE task_id = ?1",
                    [&task],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap();
            assert_eq!(
                persisted,
                (winner_agent.to_owned(), claimed_at.to_owned(), 1)
            );
            fixture.assert_storage_and_integrity();
        }
    }
}

#[test]
fn repeated_one_candidate_claim_next_races_return_claim_and_empty_success() {
    let fixture = SharedWorktrees::new();
    let gates = tempdir().unwrap();
    for (pairing, roots) in fixture.pairings() {
        for repetition in 0..REPETITIONS {
            let label = format!("next-one-{pairing}-{repetition}");
            let task = fixture.create(&label, &label, 50);
            let arguments = fixture.agents.clone().map(|agent| {
                vec![
                    "task".into(),
                    "claim-next".into(),
                    "--agent".into(),
                    agent,
                    "--tag".into(),
                    label.clone(),
                    "--json".into(),
                ]
            });
            let results = gated_race(&gates, &label, roots, arguments);
            assert!(
                results.iter().all(|result| result.status == 0),
                "{results:#?}"
            );
            let values: Vec<_> = results.iter().map(GatedResult::json).collect();
            assert_eq!(
                values
                    .iter()
                    .filter(|value| value["data"]["id"] == task)
                    .count(),
                1
            );
            assert_eq!(
                values
                    .iter()
                    .filter(|value| value["data"].is_null())
                    .count(),
                1
            );
            assert!(values.iter().all(|value| value["error"].is_null()));
            let connection = Connection::open(&fixture.database).unwrap();
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM task_claims WHERE task_id = ?1",
                    [&task],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "{label}: {results:#?}");
            fixture.assert_storage_and_integrity();
        }
    }
}

#[test]
fn repeated_multi_candidate_claim_next_races_claim_distinct_ordered_candidates() {
    let fixture = SharedWorktrees::new();
    let gates = tempdir().unwrap();
    for (pairing, roots) in fixture.pairings() {
        for repetition in 0..REPETITIONS {
            let label = format!("next-many-{pairing}-{repetition}");
            let expected = [
                fixture.create(&format!("{label}-high"), &label, 90),
                fixture.create(&format!("{label}-low"), &label, 80),
            ];
            let arguments = fixture.agents.clone().map(|agent| {
                vec![
                    "task".into(),
                    "claim-next".into(),
                    "--agent".into(),
                    agent,
                    "--tag".into(),
                    label.clone(),
                    "--json".into(),
                ]
            });
            let results = gated_race(&gates, &label, roots, arguments);
            assert!(
                results.iter().all(|result| result.status == 0),
                "{results:#?}"
            );
            let values: Vec<_> = results.iter().map(GatedResult::json).collect();
            let mut actual: Vec<_> = values
                .iter()
                .map(|value| value["data"]["id"].as_str().unwrap().to_owned())
                .collect();
            actual.sort();
            let mut expected = expected.to_vec();
            expected.sort();
            assert_eq!(actual, expected, "{label}: {results:#?}");
            assert!(values.iter().all(|value| value["error"].is_null()));
            let connection = Connection::open(&fixture.database).unwrap();
            for task in &expected {
                let count: i64 = connection
                    .query_row(
                        "SELECT COUNT(*) FROM task_claims WHERE task_id = ?1",
                        [task],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(count, 1, "{label}: {results:#?}");
            }
            fixture.assert_storage_and_integrity();
        }
    }
}
