# Agent approval for user-only commands

## Status

Future improvement; not required for the MVP or E5-S4.

## Context

The logical `user` actor is a product convention, not an authentication boundary. An agent process with the same shell and filesystem permissions as the human user can invoke a user-only CLI path such as force-unclaim. Flags and interactive confirmation make destructive intent explicit, but they cannot prove that a human initiated the command.

## Proposed improvement

Introduce an agent-execution rule that requires explicit human approval before an agent may run commands classified as user-only or privileged. Force-unclaim should be included in that class.

The approval mechanism should eventually define:

- How the CLI or agent runtime determines that a command was initiated by an agent.
- Which commands require approval and how that policy is configured.
- How the exact command, repository, task, current claim owner, and claim timestamp are shown to the approver.
- How approval is bound to the observed state so that it cannot authorize a replacement claim or a different command.
- How non-interactive execution, denial, timeout, logging, and audit history behave.
- Whether enforcement belongs in `tbtm`, the agent runtime, or an OS-level permission boundary.

## Constraints

- Do not treat the current logical actor model as security enforcement.
- Preserve the MVP's explicit `--force` and confirmation safeguards even if agent approval is added later.
- Approval should be narrowly scoped and single-use; it must not grant a general ability to run future user-only commands.

