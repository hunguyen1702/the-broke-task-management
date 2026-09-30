# H1 deferred-work register

These items were deliberately left out of H1-T0/H1-T1, not forgotten. A row
becomes an implementation contract only after its own planning session and
dependency review. Do not treat this register as permission to implement it
inside H1-T1.

| Item | Current tracking | Why deferred | Planning boundary |
|---|---|---|---|
| Commitment Gate | H1-T19, removed from the active H1 flow ([TD-0008](../decisions/TD-0008-defer-h1-commitment-gate.md)) | Plan already defines observable acceptance and verification; the user chose Plan → Execute without a separate gate. Constitution snapshot 1.11.0 aligns the active Plan and Router rules; prior revisions retain the earlier flow as history. | Reintroduce only after a new user decision and dedicated planning. Do not treat H1-T19 as the next active task. |
| Constitution rule-management workflow | H1-T24, `needed / not_planned` | H1-T1 establishes states, authority and validation, but not the steps for proposing, obtaining user approval, activating, revising, superseding or retiring rules. | Plan as a workflow after Constitution is available; define input, decisions, user interaction, atomicity, failure recovery and verification then. |
| General framework ruleset upgrade/reinstall | deferred, no task ID yet | The first install pins a version and detects drift; safe general upgrade semantics require real installed-state experience. The user separately authorized versioned additions needed for approved H1 workflow nodes during initial construction (H1-T1 addendum). | Create a task for general upgrade/reinstall; preserve project rules and review exact revision references. The bounded H1 construction additions keep manifest, digest, index, provenance, and validation consistent. |
| Router trigger and route-decision contract | H1-T13 placeholder | Workflow definitions do not decide when they are called. | Define when planning Risk Router, not in Constitution. |
| Workflow guide layout and detailed node instructions | owning node placeholders | No physical layout for other nodes was approved by H1-T0. | Decide per node when its workflow is planned. |
| Shared schemas or components across nodes | deferred, no task ID yet | H1-T0 explicitly rejects standardizing speculative common outputs. | Extract only after concrete nodes demonstrate the same requirement. |

This register can be updated during later planning, but no entry here changes
the active [H1 roadmap](../epics/H1-build-adaptive-repository-harness.md) by
itself.
