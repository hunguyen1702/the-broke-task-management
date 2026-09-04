# Technical decisions

Technical decision records explain durable implementation or compatibility choices. They supplement, but never replace, PRD and epic product contracts.

Use a record when a choice materially affects architecture, persistence, compatibility, concurrency, security, migration, public output, or future maintenance. Observable product changes must also update the relevant PRD/epic through its normal planning workflow.

File naming:

```text
<searchable-decision-name>-TD-<four-digits>.md
```

Copy `TD-template.md`. IDs are sequential and never reused. Accepted records are immutable in meaning; a later record supersedes them.

Classify acceptance impact as `revalidate`, `add`, or `supersede` only when a
decision changes a common user workflow. Use `none` for internal behavior that
is adequately covered by Rust tests.
