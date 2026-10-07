# Offline workspace backup and restore

Stop every TBTM client and writer in the main and all linked worktrees before running these steps. Keep them stopped through verification. Use a durable destination **outside** the repository for both the backup and displaced state. In particular, do not use `.tbtm.backup-*` or `.tbtm.staging-*` under the repository root; uninstall removes those paths. Do not manually remove SQLite journal files. If any identity or health check fails, stop and preserve all artifacts for investigation; do not force-initialize or replace them.

Save the following exact script as `recovery.py`, then run `python3 recovery.py backup /path/to/any/worktree /durable/external/backup.tbtm` or `python3 recovery.py restore /path/to/any/worktree /durable/external/backup.tbtm /durable/external/displaced.tbtm`. The destination parent directories must already exist. `tbtm` must be on `PATH` (or set `TBTM_BIN` to its executable). An offline backup is a complete copy of the canonical `.tbtm` directory, including its matching `config.json` and `tbtm.db`.

```python
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import sys


def run(*args, cwd):
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{args!r}: {result.stderr or result.stdout}")
    return result.stdout


def canonical_root(worktree):
    lines = run("git", "worktree", "list", "--porcelain", cwd=worktree).splitlines()
    if not lines or not lines[0].startswith("worktree "):
        raise RuntimeError("cannot resolve canonical Git worktree")
    return pathlib.Path(lines[0][9:]).resolve(strict=True)


def check_workspace(workspace):
    config = json.loads((workspace / "config.json").read_text())
    if config.get("database") != "tbtm.db":
        raise RuntimeError("unexpected database path; preserve workspace")
    database = workspace / "tbtm.db"
    if not database.is_file() or database.is_symlink():
        raise RuntimeError("database missing or unsafe; preserve workspace")
    # A writable open lets SQLite recover an existing journal. Close before copying.
    with sqlite3.connect(f"file:{database}?mode=rw", uri=True) as connection:
        identity = connection.execute(
            "SELECT repository_id, prefix, created_at FROM repository_metadata WHERE singleton = 1"
        ).fetchone()
        expected = (config["repositoryId"], config["prefix"], config["createdAt"])
        if identity != expected:
            raise RuntimeError("configuration/database identity mismatch; preserve workspace")
        if connection.execute("PRAGMA integrity_check").fetchone() != ("ok",):
            raise RuntimeError("SQLite integrity check failed; preserve workspace")
        if connection.execute("PRAGMA foreign_key_check").fetchone() is not None:
            raise RuntimeError("SQLite foreign key check failed; preserve workspace")
    return config["repositoryId"]


def health(worktree, root, identity):
    data = json.loads(run(os.environ.get("TBTM_BIN", "tbtm"), "repo", "status", "--json", cwd=worktree))["data"]
    if data["health"] != "healthy" or pathlib.Path(data["repositoryRoot"]).resolve() != root or data["repositoryId"] != identity:
        raise RuntimeError("repository health or canonical identity mismatch; preserve workspace")


def outside_repo(path, root):
    path = path.resolve()
    if path == root or root in path.parents:
        raise RuntimeError("destination must be outside the repository")
    if path.exists() or path.is_symlink():
        raise RuntimeError(f"destination already exists: {path}")
    if not path.parent.is_dir():
        raise RuntimeError(f"destination parent missing: {path.parent}")
    return path


def main():
    if len(sys.argv) not in (4, 5) or sys.argv[1] not in ("backup", "restore"):
        raise RuntimeError("usage: recovery.py backup WORKTREE EXTERNAL_BACKUP | restore WORKTREE EXTERNAL_BACKUP EXTERNAL_DISPLACED")
    action, worktree = sys.argv[1], pathlib.Path(sys.argv[2]).resolve(strict=True)
    root = canonical_root(worktree)
    live = root / ".tbtm"
    backup = pathlib.Path(sys.argv[3]).resolve()
    if action == "backup":
        destination = outside_repo(backup, root)
        identity = check_workspace(live)
        health(worktree, root, identity)
        shutil.copytree(live, destination)
        if check_workspace(destination) != identity:
            raise RuntimeError("copied identity differs; retain both workspaces")
    else:
        if len(sys.argv) != 5 or not backup.is_dir() or backup.is_symlink():
            raise RuntimeError("existing external backup and displaced destination required")
        if backup == live or root in backup.parents:
            raise RuntimeError("backup must be outside the repository")
        displaced = outside_repo(pathlib.Path(sys.argv[4]), root)
        if displaced == backup or backup in displaced.parents or displaced in backup.parents:
            raise RuntimeError("backup and displaced destinations must be separate")
        identity = check_workspace(backup)
        if not live.is_dir() or live.is_symlink():
            raise RuntimeError("live workspace missing or unsafe; preserve artifacts")
        live.rename(displaced)
        shutil.copytree(backup, live)
        if check_workspace(live) != identity:
            raise RuntimeError("restored identity differs; retain backup and displaced state")
        health(worktree, root, identity)
    print(f"{action} verified: {identity}")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        sys.exit(f"Recovery stopped: {error}")
```

After restore, verify representative saved agents, task details, claims, and relationships with `tbtm agent list --json` and `tbtm task view ID --json` from both main and linked worktrees before resuming writers. Keep the backup and displaced directory until the recovered state has been accepted. If restore reports failure, leave both directories in place and stop; inspect the copied live directory before any further recovery attempt.
