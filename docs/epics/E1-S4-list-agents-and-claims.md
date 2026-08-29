---
id: E1-S4
kind: epic
planning_status: done
implementation_status: ready
depends_on:
  - E1-S3
  - E5-S1
---

# E1-S4: List agents and claims

## Outcome

A user can inspect every registered agent and see the tasks that agent currently claims, including agents that own no work.

## User story

As a user, I want to list agents and their claims so that I can understand current ownership.

## Command

```text
tbtm agent list [--json]
```

## Product decisions

### Result contents

- Return every registered agent in the repository, including agents with no active claims.
- Each agent contains `id`, normalized `baseName`, `displayName`, `createdAt`, and a `claims` collection.
- Each current claim contains `taskId`, `title`, `status`, and `claimedAt`.
- `status` is the task's current status name. Claim ownership remains independent of task status.
- Only active claims appear. This story does not introduce claim history.
- An empty agent registry is a successful result with an empty collection and exit code 0.

### Deterministic ordering

- Order agents by `createdAt` ascending, then `id` ascending.
- Order each agent's claims by `claimedAt` ascending, then `taskId` ascending.
- Ordering is identical in human and JSON output.

### Consistent read behavior

- Use the E1-S2 resolver with read-only, no-create database access.
- Read agents, active claims, tasks, and current statuses from one SQLite read transaction so one response represents one consistent database snapshot.
- Listing never applies migrations, repairs state, or changes repository or database content.
- The implementation consumes the authoritative task, status, and active-claim model established by prerequisite stories. E1-S4 does not define a competing claim schema.

### MVP scope

- The command has no filters, pagination, sorting flags, or agent-detail mode.
- The complete local registry is returned. Personal-repository scale makes pagination unnecessary for the MVP.

### Output and errors

Human output presents each agent and its current claimed tasks. Agents without claims remain visible and are explicitly recognizable as having no claims.

JSON reuses the shared envelope and stable camelCase fields:

```json
{
  "ok": true,
  "data": {
    "agents": [
      {
        "id": "UUID",
        "baseName": "claude",
        "displayName": "claude-a1b2c3d4",
        "createdAt": "RFC 3339 UTC timestamp",
        "claims": [
          {
            "taskId": "project-story-a1b2c3",
            "title": "Implement parser",
            "status": "In progress",
            "claimedAt": "RFC 3339 UTC timestamp"
          }
        ]
      }
    ]
  },
  "error": null
}
```

Failures reuse the shared `error.code`, `error.message`, and `error.details` shape and E1-S2 exit-code mapping:

- Success, including an empty registry: exit code 0.
- Database or unexpected operational failure: exit code 1.
- Invalid repository configuration: exit code 2.
- Repository not initialized: exit code 3.
- Permission denied: exit code 5.

## Functional acceptance criteria

1. `tbtm agent list` returns every registered repository-local agent with identity, base name, display name, creation time, and current claims.
2. An agent with no active claims remains in the result with an empty `claims` collection.
3. Each claim shows the claimed task ID, title, current status name, and preserved claim timestamp.
4. Released claims and claim history do not appear as current ownership.
5. Agents and claims use the documented deterministic ordering in both human and JSON output.
6. An empty registry returns success, an empty agent collection, and exit code 0.
7. Human output clearly distinguishes agents and their tasks, including agents with no claims.
8. JSON uses the shared envelope and documented camelCase shape.
9. Repository, configuration, database, permission, and unexpected failures retain stable shared error and exit behavior.
10. The complete result is read from one consistent SQLite snapshot without changing repository state.

## Non-functional acceptance criteria

1. Listing is read-only, creates no SQLite side files through application writes, applies no migrations, and performs no network access.
2. The command feels immediate for a local personal repository and avoids one database query per agent.
3. Output ordering and JSON field names remain stable across repeated reads of unchanged data.
4. Concurrent local mutations produce either the snapshot before or after a committed mutation, never a mixed ownership view.
5. Tests cover Linux, macOS, and Windows behavior at a practical level.

## Verification

- List an empty registry and confirm exit code 0 and an empty collection.
- Register multiple agents, leave at least one without a claim, and confirm every identity remains visible.
- Give agents multiple active claims and verify claim fields and deterministic ordering.
- Release a claim and confirm it no longer appears while the agent remains listed.
- Compare the response with direct SQLite reads from the same fixture.
- Exercise a controlled concurrent claim change and verify each response is a consistent snapshot.
- Compare repository config and database bytes before and after listing where the platform permits, and verify no application-created journal or WAL artifact remains.
- Snapshot human and JSON output and assert inherited error and exit-code categories.
- Run formatting, lint, and tests through `mise` after prerequisite stories are implemented.

## Out of scope

- Registering, renaming, deleting, disabling, or selecting an agent.
- Claiming, unclaiming, force-unclaiming, or transferring tasks.
- Claim history, claim age formatting, or automatic claim expiry.
- Filtering, pagination, alternate sorting, or a separate agent-detail command.
- Defining task, status, or active-claim persistence independently of their prerequisite stories.
- Visual Studio Code agent-registry views.

## Dependencies

- E1-S3 provides the registered agent identity model.
- E5-S1 provides authoritative active-claim persistence and depends on E2-S1, E3-S1, and E4-S2 for task availability.
- Implementation must integrate with those completed models rather than anticipating their database layout.

## Implementation task

See [E1-S4-T1: Implement agent and claim listing](../tasks/E1-S4-T1-implement-agent-and-claim-listing.md).
