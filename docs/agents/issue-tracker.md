# Issue tracker: Local Markdown

Engineering skill issues and specs live in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`.
- Spec: `.scratch/<feature-slug>/spec.md`.
- Tickets: `.scratch/<feature-slug>/issues/<NN>-<slug>.md`,
  numbered from `01`, one file per ticket.
- Triage role: a `Status:` line near the top, using the strings
  in `docs/agents/triage-labels.md`.
- Conversation: append under `## Comments`.

Existing behavior contracts remain in `docs/epics/` and `docs/tasks/`,
with project status recorded in `docs/STATUS.md`.

## Publish and fetch

To publish, create the spec or ticket at the corresponding path.
To fetch, read the referenced file. Resolve numeric ticket references
within their feature directory; ask for the feature if ambiguous.

## Wayfinding operations

- Map: `.scratch/<effort>/map.md`, containing Notes,
  Decisions-so-far, and Fog.
- Child: `.scratch/<effort>/issues/NN-<slug>.md`.
- Type: `Type: research`, `prototype`, `grilling`, or `task`.
- Lifecycle: `Status: open`, `claimed`, or `resolved`.
  These are wayfinding states; triage tickets use the triage roles.
- Blocking: `Blocked by: NN, NN`; unblocked once all listed
  tickets are resolved.
- Frontier: first open, unblocked ticket by number.
- Claim: save `Status: claimed` before work.
- Resolve: append `## Answer`, set `Status: resolved`, and add
  a gist plus file link to the map's Decisions-so-far.
