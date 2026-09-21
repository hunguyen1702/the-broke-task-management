# Constitution lookup

The manifest pins framework and ruleset version `1.0.0`. Its `frameworkDigest`
is SHA-256 over every framework-tree file sorted by UTF-8 relative path. For
each file, hash its relative path bytes, a NUL byte, its exact file bytes, and
another NUL byte. The framework tree is immutable until a separately planned
upgrade. The installation approval record identifies the pinned ruleset; the
validator checks consistency and cannot prove human intent.

Run `.harness/scripts/validate-constitution validate` before mutating harness
work. An invalid Constitution blocks that work; read-only diagnosis is allowed.
Run `.harness/scripts/validate-constitution inspect-effective path/one path/two`
with **all** relevant repository-relative paths, then read each returned rule
file. Re-query if the target path set grows. With no paths, inspection returns
all effective rules. Output is stable YAML. An active rule is effective unless
an active rule explicitly supersedes its exact ID and revision. `extends` keeps
the referenced rule effective. Exclusions win over inclusions.

Scope patterns use `/` and are relative to the repository root. `*` matches
zero or more characters in one path segment; `**` matches zero or more whole
segments. For example, `docs/**/*.md` includes Markdown at any depth under
`docs`. The validator also accepts `repository: true` for repository-wide
rules. Path specificity does not override a framework rule.

To propose a project rule, add `project/<category>/<id>-r<revision>.md` with
`status: draft`, required frontmatter and the three required body sections.
References use exact `{id, revision}` pairs. Drafts appear in the index but
never in effective inspection. The future rule-management workflow owns
approval and activation; do not set `active` or approval metadata without
explicit user authorization. After any canonical change, run `rebuild-index`
and `validate`. Rebuild validates inputs first and replaces only the index.
`validate` and `inspect-effective` never write files. Exit codes are 0 for
success, 1 for invalid Constitution or stale index, and 2 for invocation or IO
failure. Body words are whitespace-delimited tokens; frontmatter is excluded
from body limits and has its own byte cap.
