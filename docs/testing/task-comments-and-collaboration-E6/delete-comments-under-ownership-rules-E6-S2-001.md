---
id: AT-E6-S2-001
story: E6-S2
status: draft
---

# Delete comments under ownership rules

## Setup

Use a temporary repository with one active task, one archived task, two registered
agents, and comments authored by the logical user and each agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Agent deletes own comment | Delete an agent-authored comment using that agent's UUID with `--json` | Exit `0`; output returns the deleted comment with its unchanged ID, task ID, content, author, and creation time; listing no longer includes it |
| Agent cannot delete another actor's comment | Delete a user- or other-agent-authored comment using the foreign agent UUID with `--json` | Exit `5`; error code is `COMMENT_DELETE_FORBIDDEN`; the comment remains listed |
| User deletes any comment | Delete an agent-authored comment without `--agent` | Exit `0`; human output identifies the deleted comment ID and task ID |
| Archived-task parity | Delete a comment belonging to an archived task as its permitted actor | Exit `0`; the comment is removed while the task remains archived |
| Unknown comment | Delete a nonexistent comment UUID from an existing task with `--json` | Exit `3`; error code is `COMMENT_NOT_FOUND` |
| Comment belongs to another task | Delete an existing comment UUID while naming a different existing task with `--json` | Exit `3`; error code is `COMMENT_NOT_FOUND`; the original comment remains listed on its task |
| Unknown agent | Delete a comment using an unregistered agent UUID with `--json` | Exit `3`; error code is `AGENT_NOT_FOUND`; the comment remains listed |
| Unknown task | Delete a comment from a nonexistent task ID with `--json` | Exit `3`; error code is `TASK_NOT_FOUND`; no comment is removed |

## References

- [E6-S2 contract](../../epics/E6-S2-delete-comments-under-ownership-rules.md)
- [Common test setup](../test-setup.md)
