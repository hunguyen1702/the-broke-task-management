# H1 deferred-work register

These items were deliberately left out of H1-T0/H1-T1, not forgotten. A row
becomes an implementation contract only after its own planning session and
dependency review. Do not treat this register as permission to implement it
inside H1-T1.

| Item | Current tracking | Why deferred | Planning boundary |
|---|---|---|---|
| Constitution rule-management workflow | H1-T24, `needed / not_planned` | H1-T1 establishes states, authority and validation, but not the steps for proposing, obtaining user approval, activating, revising, superseding or retiring rules. | Plan as a workflow after Constitution is available; define input, decisions, user interaction, atomicity, failure recovery and verification then. |
| Framework ruleset upgrade/reinstall | deferred, no task ID yet | The first install pins a version and detects drift; safe upgrade semantics require real installed-state experience. | Create a task when an upgrade is needed; preserve project rules and review exact revision references. |
| Router trigger and route-decision contract | H1-T13 placeholder | Workflow definitions do not decide when they are called. | Define when planning Risk Router, not in Constitution. |
| Workflow guide layout and detailed node instructions | owning node placeholders | No physical layout for other nodes was approved by H1-T0. | Decide per node when its workflow is planned. |
| Shared schemas or components across nodes | deferred, no task ID yet | H1-T0 explicitly rejects standardizing speculative common outputs. | Extract only after concrete nodes demonstrate the same requirement. |

This register can be updated during later planning, but no entry here changes
the active [H1 roadmap](../epics/H1-build-adaptive-repository-harness.md) by
itself.
