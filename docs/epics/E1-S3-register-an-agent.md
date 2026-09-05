---
id: E1-S3
kind: epic
planning_status: done
implementation_status: done
contract_depends_on:
  - E1-S1
---

# E1-S3: Register an agent

## Outcome

A coding agent can create a repository-local identity and receive a stable UUID suitable for later agent-scoped task, claim, and comment operations.

## User story

As a coding agent, I want to register a base name so that my actions and claims have a stable identity.

## Command

```text
tbtm agent register <base-name> [--json]
```

## Product decisions

### Registration behavior

- Every successful invocation creates a new agent record with a UUID v4.
- Registration never searches for or reuses an existing identity, even when the normalized base name matches an existing agent.
- The record stores `id`, normalized `baseName`, unique `displayName`, and `createdAt`.
- Registration is repository-local and uses the shared E1-S2 resolver with read-write, no-create database access.

### Base-name normalization

- Apply the E1-S1 prefix normalization contract: transliterate Latin text to lowercase ASCII, replace runs outside `[a-z0-9]` with `-`, and trim leading and trailing `-`.
- Validate after normalization and require length `1–48`.
- Never truncate silently.
- Store and return the normalized value; the original input is not retained as a separate field.
- Invalid or empty normalized input returns `INVALID_AGENT_NAME`.

### UUID and display name

- Generate a UUID v4 for each registration attempt.
- Build the display name as `<normalized-base-name>-<suffix>`.
- The suffix is the first eight lowercase hexadecimal characters of the UUID without hyphens, for example `claude-a1b2c3d4`.
- UUID and display name are independently unique within the repository database.
- If the derived display name collides, generate a new UUID and retry the insert in the same registration operation. Never reuse or overwrite the existing agent.

### Agent identity for later commands

- The UUID is the authoritative identity for agent-scoped operations.
- Later commands provide it explicitly as `--agent <uuid>`.
- TBTM does not store a repository-wide or machine-wide current-agent selection. This avoids shared mutable selection when several coding agents use the same repository concurrently.
- Display name is for human recognition and is not accepted as the authoritative actor identifier.

### Persistence and atomicity

- Add a versioned SQLite migration for the agent registry.
- `schemaVersion` is the repository/config compatibility version and remains `1` for this MVP addition.
- Internal migrations are tracked separately in `schema_migrations`; the agent registry is migration `0002`.
- A newly initialized repository applies all bundled schema-version-1 migrations. Registration on an older schema-version-1 repository applies pending compatible migrations before inserting the agent.
- Migration application and agent insertion use separate SQLite transactions: committed migrations remain applied if the later registration insert fails, while no partial agent record is left behind.
- Insert each agent record atomically with its generated identity, display name, and UTC creation timestamp.
- A failed registration leaves no partial agent record.
- Registration does not modify repository configuration or existing agent records. Applying a pending internal migration may change database schema and migration metadata.

### Output and errors

Human output shows the registered UUID, normalized base name, display name, and creation time. JSON reuses the E1-S1 envelope:

```json
{
  "ok": true,
  "data": {
    "id": "UUID",
    "baseName": "claude",
    "displayName": "claude-a1b2c3d4",
    "createdAt": "RFC 3339 UTC timestamp"
  },
  "error": null
}
```

Failures use the shared `error.code`, `error.message`, and `error.details` shape. Relevant behavior:

- `INVALID_AGENT_NAME`, exit code 2, for invalid normalized base names.
- E1-S2 repository, configuration, database, and permission errors retain their existing codes and exit codes.
- An unexpected failure after bounded identity-generation retries is an operational failure with exit code 1.
- Success returns exit code 0 and writes one JSON object to stdout in JSON mode.

## Functional acceptance criteria

1. `tbtm agent register <base-name>` creates one repository-local agent with a UUID v4, normalized base name, unique display name, and UTC creation time.
2. Repeating registration with the same base-name input creates a distinct UUID and display name without changing the earlier record.
3. Base names follow the E1-S1 normalization and `1–48` character validation rules; invalid input creates no record.
4. Display names use the exact normalized-base plus eight-character UUID-derived suffix format.
5. UUID and display-name uniqueness are enforced by the database; a suffix collision regenerates identity rather than overwriting or reusing data.
6. Registration is atomic and leaves no partial record on failure.
7. Human and JSON success output contains `id`, `baseName`, `displayName`, and `createdAt`.
8. Validation, repository, database, permission, and unexpected failures use stable documented error and exit behavior.
9. The returned UUID can be supplied through the documented `--agent <uuid>` contract in later agent-scoped stories; no implicit current-agent state is created.
10. Adding the agent registry keeps repository/config `schemaVersion: 1`; internal migration `0002` is tracked separately and can upgrade an older compatible repository without rewriting config.

## Non-functional acceptance criteria

1. Registration performs no network access and feels immediate for a local personal repository.
2. Concurrent registrations cannot create duplicate UUIDs or display names and cannot corrupt the database.
3. Persisted timestamps use RFC 3339 UTC and JSON field names remain stable camelCase.
4. Existing repository configuration and agent records are not modified by a new registration.
5. Tests cover Linux, macOS, and Windows behavior at a practical level.

## Verification

- Register the same base name repeatedly and query SQLite to verify distinct records and uniqueness constraints.
- Exercise normalization, invalid-name boundaries, and a controlled display-suffix collision.
- Inject an insert failure and verify no partial agent record remains.
- Open an older schema-version-1 repository containing only migration `0001`, register an agent, and verify migration `0002` is recorded without changing config.
- Run concurrent registrations and verify all successful identities are distinct.
- Snapshot human and JSON output and assert exact exit-code categories.
- Run formatting, lint, and tests through `mise`.

## Out of scope

- Listing agents or their current claims; covered by E1-S4.
- Renaming, deleting, disabling, or automatically cleaning up agents.
- Authentication, authorization, secrets, or access tokens.
- Persisting or selecting a global current agent.
- Implementing downstream task, claim, or comment commands that consume `--agent`.

## Implementation task

See [E1-S3-T1: Implement agent registration](../tasks/E1-S3-T1-implement-agent-registration.md).
