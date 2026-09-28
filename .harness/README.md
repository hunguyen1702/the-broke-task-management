# Constitution lookup

The manifest, schema, and derived index live directly in `.harness/`.
Canonical rules live in `.harness/<category>/`, including active workflows in
`.harness/workflow/`. Each rule's `origin` is `framework` or `project`; the
directory layout does not encode origin. The pinned `frameworkDigest` hashes
only framework-origin rules, sorted by their path relative to `.harness/`.
For each file, hash its relative path bytes, NUL, exact file bytes, NUL.
Framework-origin rules are pinned by a ruleset version. During initial H1
construction, the user's bounded authorization allows additions needed for
approved workflow nodes. Each addition creates a new pinned snapshot with
matching provenance, digest, index, and validation; do not edit a snapshot
under its old version. General upgrade/reinstall remains deferred.
Installation approval identifies the pinned ruleset; validation checks
consistency and cannot prove human intent.

Run `.harness/scripts/validate-constitution validate` before mutating harness
work. An invalid Constitution blocks that work; read-only diagnosis is allowed.
Run `.harness/scripts/validate-constitution inspect-context path/one path/two`
with all relevant repository-relative paths, then read each returned bootstrap
rule. Bootstrap contains every non-workflow rule, Workflow Entry, and Intent;
it excludes Current Truth, Risk Router, and routed workflows. After Intent
returns confirmed `ready`, run `inspect-workflow rule-current-truth` with the
same paths and read its returned chain. After Current Truth returns `ready` or
a routable `unresolved`, run `inspect-workflow rule-risk-router` with those
paths and read its returned chain. After Risk Router selects a route, run
`inspect-workflow <workflow-id> path/one path/two` and read only that selected
canonical chain. `inspect-workflow` is a deterministic lookup, not a lifecycle
engine. Re-query affected context when target paths, rules, or route change.
`inspect-effective` remains the complete diagnostic view: with no paths it
returns every effective rule. Output is stable YAML. An active rule is effective
unless another active rule explicitly supersedes its exact ID and revision.
`extends` keeps its referenced rule effective. Exclusions win over inclusions.

Scope patterns use `/` relative to the repository root. `*` matches zero or
more characters in one segment; `**` matches zero or more whole segments.
`repository: true` applies everywhere. Path specificity does not override a
rule by itself.

To propose a project rule, add `.harness/<category>/<id>-r<revision>.md` with
`origin: project`, `status: draft`, required frontmatter, and the three required
body sections. Drafts appear in the index but not effective inspection. The
future rule-management workflow owns approval and activation; do not set
`active` or approval metadata without explicit user authorization. After any
canonical change, run `rebuild-index` and `validate`. Rebuild validates inputs
first and replaces only the index. `validate`, `inspect-effective`,
`inspect-context`, and `inspect-workflow` never write files. Exit codes are 0
for success, 1 for invalid Constitution or stale index, and 2 for invocation,
selection, or IO failure.
