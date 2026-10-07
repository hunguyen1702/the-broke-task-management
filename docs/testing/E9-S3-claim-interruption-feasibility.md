# E9-S3 claim interruption feasibility

Ticket 10 is incomplete. On macOS, a pre-established SQLite SHARED reader lock prevents the real CLI writer from reaching claim insertion. The claim command calls `apply_pending_migrations` before its claim transaction, even on a fully migrated fixture. The migration phase times out under the reader lock.

Reproduce with `rtk proxy cargo test -p tbtm --test interrupted_claim killed_claim_writer -- --ignored --nocapture`. The ignored test uses a fully initialized temporary repository, a populated `task_claims` table with a unique journal marker, a reader transaction that reads that table, a real `task claim --json` child, and a seven-second gate. Observed on 2026-10-07: after the five-second busy timeout, the child exited 1 with JSON code `DATABASE_UNAVAILABLE`, details check `database migration`, and message `database is locked`. No claim rollback journal existed. The test kills and reaps a still-running child before releasing the reader on failure.

The passing `claim_post_write_trigger_failure_rolls_back_claim_and_metadata` test uses a public core claim call and a test-owned SQLite trigger. The trigger performs a write and then raises `FAIL`; the operation rolls back the claim, the trigger write, and task metadata. This proves that rollback boundary but does not satisfy CLI termination evidence.

Reopen the interruption-gate design before ticket 10 or dependent ticket 11 can close. One possible approved production change is to validate the latest migration ledger without beginning a write transaction when no migration is pending, then repeat the same reader-lock experiment. A different gate would need a reliable, mutation-specific boundary without production hooks. Neither option is established by this evidence.
