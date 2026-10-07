# Domain Docs

## Layout and reading rules

This repo uses a single domain context across its Rust crates.

Before exploring:
- Read root `GLOSSARY.md` if present.
- Read relevant records in `docs/decisions/` and its README.
- Read the PRD and approved epic/task contracts required by
  the repository workflow.

If the glossary is absent, proceed silently. Create it lazily through
domain-modeling when terms are resolved.

## Vocabulary

Use glossary terms in tickets, proposals, hypotheses, and tests.
For a missing concept, reconsider the wording or note the gap for
domain-modeling.

## Decisions

Use `docs/decisions/README.md` and `TD-template.md` for durable decisions.
Keep decision records in the existing directory and naming convention.
They supplement the PRD and approved contracts.

Surface conflicts with accepted decisions explicitly, identifying the
record and why reopening it is warranted. Follow the repository's
supersession rules.
