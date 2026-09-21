---
id: H1-T1
kind: implementation_task
planning_status: done
implementation_status: done
depends_on:
  - H1-T0
---

# H1-T1: Implement Constitution

## Parent and readiness

[H1 adaptive repository harness](../epics/H1-build-adaptive-repository-harness.md).
H1-T0 is complete. This task implements the Constitution node alone; it does
not implement Intent, Router, Current Truth, workflow guides, or a common data
model for other nodes. H1 does not change the `tbtm` product contract or CLI.

## Approved layout revision (2026-09-21)

The user revised the installed Constitution after the original H1-T1 implementation.
Canonical rules now share `.harness/<category>/` regardless of origin; `origin`
frontmatter distinguishes framework and project rules. Schema and derived index
live at `.harness/schema.yaml` and `.harness/index.yaml`. The pinned digest
covers framework-origin rule files using their paths relative to `.harness/`.
The user also authorized the generic Current Truth rule in `.harness/workflow/`.
All future workflows belong there. Historical statements below about H1-T1's
initial empty workflow category describe the original 1.0.0 install, not the
current 1.1.0 state. See
[TD-0003](../decisions/TD-0003-flatten-constitution-rules.md).

## Initial H1 construction authorization (2026-09-21)

The user subsequently authorized adding framework workflow rules needed for
approved H1 nodes during initial harness construction without a separate
ruleset approval round for each node. This is a bounded revision to the
specified-ruleset approval process below, not permission to mutate the 1.1.0
snapshot in place. Each installed snapshot still receives a new
`rulesetVersion`, matching framework-rule installation provenance, a
recomputed `frameworkDigest` and derived index, and successful validation.
Only approved H1 node work is covered. Project-rule activation, unrelated
governance changes, and general upgrade/reinstall semantics remain outside
this authorization; H1-T24 retains the project-rule lifecycle. The approval
metadata records this user-authorized construction process, not a claim that
the user reviewed every final rule byte.

## Outcome

A newly installed harness has a usable, version-pinned, locally materialized
Constitution: canonical short Markdown rules with YAML frontmatter, one derived
query index, a manifest, and a deterministic local validator. Project rules
can be added in the same layout without changing framework-owned files.

## Five resolved design decisions

1. **Schema:** Keep one versioned, machine-readable
   `.harness/schema.yaml` describing the allowed frontmatter,
   required sections and per-category constraints. The validator implements
   deterministic cross-file checks not expressible there. Schema changes are
   framework-owned and versioned; do not silently introduce an unversioned
   second source of validation policy.
2. **References:** A relationship references an exact `{id, revision}` pair,
   never a moving logical ID. Revision supersession is explicit in the next
   revision's `supersedes`; project replacement likewise references the exact
   framework or project revision. `extends` preserves the referenced rule's
   effect; `supersedes` removes it from the effective set only after the new
   rule is active. No ambiguous or dangling references, cycles, or multiple
   effective replacements of one revision. A new framework ruleset version
   may require explicit review of project references; upgrade is out of scope.
3. **Framework approval:** User authorization to install a *specified pinned
   ruleset* approves its bundled framework rules as a set. An unattended
   bootstrap without that authorization must not claim user approval or
   activate governance on the user's behalf. The installer may mark rules
   active and record provenance only for the explicitly authorized ruleset;
   the script cannot itself prove the user's intent. Do not fabricate a
   per-rule user conversation. Later project rule activation requires
   separate explicit user approval. Validator checks metadata consistency,
   not whether a human genuinely approved it.
4. **Scope:** Require one `scope` per rule: `repository: true` or nonempty
   `paths.include`, optionally `paths.exclude`. Patterns are relative to the
   repository root, use `/` separators and documented `*` (one segment) and
   `**` (zero or more segments) glob behavior. Exclusions win over inclusions;
   match all relevant target paths and re-query if the target set expands.
   No workflow, role or risk scope. An active project rule may extend or
   supersede a framework rule, but path specificity alone never overrides it.
5. **Script interface:** The executable named by manifest supports
   `validate`, `rebuild-index`, and `inspect-effective`. `validate` and
   `inspect-effective` are read-only; `rebuild-index` validates canonical
   inputs before replacing only the derived index atomically. Exit 0 means
   success, 1 invalid Constitution or stale index, 2 invocation/IO failure.
   Diagnostics are deterministic and identify file, rule and violation on
   stderr; effective inspection prints stable YAML on stdout. Missing or
   unsupported manifest/schema and missing rule tree are errors. Missing or
   stale index fails `validate` but may be rebuilt from valid canonical files.

## Layout and canonicality

```text
.harness/
  manifest.yaml
  scripts/validate-constitution
  schema.yaml
  index.yaml
  {governance,authority,safety,workflow,repository,verification}/
```

Rule files are canonical; one index is a derived, sorted projection of all
categories, including IDs, revisions, origin, category, status, effective
state, path and file digest. Origin is explicit in frontmatter. Framework-origin
files are immutable outside a later upgrade
mechanism. Manifest pins framework version and ruleset version and a SHA-256
digest of framework-origin rule files using paths relative to `.harness/` and
documented file-byte ordering. Validator rejects digest drift. No SQLite. Entry-point
`AGENTS.md` or equivalent gets only a short harness declaration and lookup
instructions; preserve existing contents and never replace user guidance.

## Rule contract

- Frontmatter: `schemaVersion`, `kind: constitution-rule`, stable `id`, positive
  `revision`, `title`, one category, status (`draft`, `active`, `superseded`,
  `retired`), scope, creation date/creator, optional exact `extends` and
  `supersedes` references, and required `origin`. Active project-origin rules require approval date and
  `approvedBy: user`; installed framework rules carry ruleset installation
  approval provenance. Dates use ISO 8601 date format. Rule filename embeds
  ID and revision; category matches the directory.
- Body: nonempty `## Rule`, `## Rationale`, `## Application` in order;
  additional category-specific sections are allowed. No `triggers` field.
  Workflow rules may define actions or refer to a guide, but route selection
  belongs to Intent/Risk Router; install no workflow rule until planned.
- Manifest body limits: default 1,000 words and 16,384 UTF-8 bytes;
  `workflow` 2,500 words and 40,960 bytes. Count Markdown body only, using a
  documented whitespace-token definition; also cap frontmatter at 8,192
  bytes independently. Reject on either exceeded limit. Limits are
  configurable only through a validated, governed manifest change.
- One active revision per logical ID. Semantic edits produce a new revision;
  small nonsemantic corrections may update a project rule in place. An
  inactive revision remains auditable. Do not hard-delete published history.
  This task validates states and effective relationships but does not build
the future operational rule-management workflow.

The operational rule-management workflow is tracked separately as H1-T24.
This task must not implement its user-interaction, approval, transition, or
rollback procedure. See the [deferred-work register](../handoff/H1-deferred-work.md).

## Baseline framework rules

Write concise rules with the agreed common body structure:

- Governance: rule format and bounded body; agents may propose scoped project
  rules from project evidence but never activate them; every Constitution
  change requires rebuilding the index when needed and running validation.
- Authority: user decides and delegates; agent executes ordinary in-scope
  work and asks for scope-expanding decisions; only user approves activation
  or semantic governance changes; validator has no semantic authority.
- Safety: bounded destructive actions; preserve pre-existing work; protect
  sensitive information.
- Repository: follow applicable local instructions; keep one canonical source
  for each fact and derive projections.
- Verification: require proportional proof; report unverified limitations.
- Workflow: H1-T1 originally left this category empty. H1-T11 now supplies
  the generic Current Truth rule; later nodes add their own workflows here.

## Implementation sequence

1. Implement manifest/schema parser and canonical rule scan with path
   containment checks, duplicate detection, size/structure validation and
   actionable diagnostics. Pin deterministic digest conventions.
2. Implement exact-reference graph resolution, active/effective computation,
   index generation and pure comparison. Validator never edits canonical rule
   content, status, approval or semantic conflicts.
3. Add baseline framework rules and manifest; calculate pinned framework
   digest, generate index, and verify clean installation. Preserve existing
   `AGENTS.md` (or equivalent) with only a short entry instruction.
4. Add script modes with atomic index replacement; ensure failure leaves
   previous index untouched. Integrate with existing repository tooling only
   where it does not change product runtime.
5. Document how an agent reads the manifest, validates, obtains effective
   rule file paths and reads the canonical files. Invalid Constitution blocks
   mutating harness work, but allows read-only diagnosis. Describe how to add
   draft project rules without creating the future approval workflow.

## Verification

Exercise clean install, effective Current Truth workflow, project draft exclusion,
approved active addition, extension and replacement, revision replacement,
path include/exclude, missing/invalid schema or manifest, invalid YAML,
missing sections, limits (including Unicode and frontmatter exclusion),
duplicate IDs/revisions, dangling/cyclic relationships, conflicting
replacements, stale/missing index, digest drift, framework edits, and failed
atomic rebuild. Ensure attempts to spoof approval cannot be claimed as
cryptographically verified. Check idempotent validate/rebuild and stable
ordering/digest across repeated runs. Use temporary isolated repositories;
preserve existing guidance. Run:

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

## Completion and exclusions

Implementation is complete only when a freshly installed pinned Constitution
validates; project drafts can be indexed but never become effective; active
changes require explicit user approval and revalidation; the script reports
structural failures deterministically without semantic decisions; framework
drift fails; and all three repository verification tasks pass. Record
acceptance impact (`none`, `add`, `revalidate`, or `supersede`) at implementation
completion based on actual user-visible changes, without modifying
`docs/testing/` in this task.

Do not implement an installer upgrade, approval UI, rule-management workflow,
Router, other nodes, a shared schema across nodes, or product CLI changes.

## Dependency cross-check

Re-read H1-T0 before approval: its flow makes Constitution an input to Router
and explicitly defers other node schemas; this task supplies governance and
validation only. No conflicting inherited persistence or output contract.
NO CONFLICT.

## Implementation result

The pinned Constitution is installed under `.harness/` with five active
framework rules and a Current Truth workflow. Canonical rule files, the
schema, manifest, derived index, and validator are covered by isolated
temporary-repository tests. The rule-management workflow remains H1-T24.

Verification: `rtk mise run format`, `rtk mise run lint`, and
`rtk mise run test` passed on 2026-09-21. Acceptance impact: `none`, because
the harness does not change the product CLI or an existing user scenario.
