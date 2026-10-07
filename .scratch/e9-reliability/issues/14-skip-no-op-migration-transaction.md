# 14: Skip no-op migration transaction on current schema

Status: resolved
Type: task
Blocked by: None

User approved this separate remediation on 2026-10-07 after the E9-S3 claim interruption gate showed a pre-held SQLite SHARED reader prevented the CLI's no-op migration transaction from committing before claim acquisition.

## Outcome

On a database whose validated migration ledger is already at the latest version, mutation commands reach their own transaction without opening a migration write transaction. A database with compatible pending migrations still applies them atomically and safely under concurrent writers.

## Boundaries

- Keep current schema, database identity, public commands, and error codes.
- Validate the ledger before the fast path. Revalidate inside the write transaction when migrations are pending.
- Prove the latest-schema path under a held SHARED reader and the pending-schema path with existing migration tests.
- Retry the real CLI claim interruption gate after this fix. Ticket 10 remains open until mutation-specific evidence passes; ticket 11 remains blocked by ticket 10.

## Forward and rollback

The forward change affects only transaction acquisition on already-current databases; it makes no data-shape change. Reverting the code restores the previous no-op transaction behavior without a data migration. Existing pending migrations remain idempotent and transactional.

## Answer

Merged as `2c3276e`. `apply_pending_migrations` now validates the ledger read-only and returns when already current; when migrations are pending it revalidates inside an IMMEDIATE transaction before applying them. Four focused migration tests passed, as did format, Clippy, and workspace tests. Ticket 10 still must prove the requested mutation-specific CLI interruption gate; this remediation does not itself satisfy that proof.
