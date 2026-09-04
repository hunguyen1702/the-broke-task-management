---
id: AT-E6-S1-001
story: E6-S1
status: approved
---

# Add and list task comments

## Setup

Use a temporary repository with one task and one registered agent.

## Test cases

| Case | Input | Expected output |
|---|---|---|
| Add a comment | Add comment `Looks good` to the task as the registered agent with `--json` | Exit `0`; output contains the comment text, author, task ID, and generated comment ID |
| List comments | List comments for the task with `--json` | Exit `0`; output includes `Looks good` with its author |
| Empty comment | Try to add an empty comment | Exit `2`; output reports invalid comment content and no comment is added |
| Unknown agent | Add a comment using a nonexistent agent ID | Exit `3`; error code is `AGENT_NOT_FOUND` |
| Unknown task | List comments for a nonexistent task ID | Exit `3`; error code is `TASK_NOT_FOUND` |

## References

- [E6-S1 contract](../../epics/E6-S1-add-and-view-comments.md)
- [Common test setup](../test-setup.md)
