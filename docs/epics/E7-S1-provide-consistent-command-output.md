---
id: E7-S1
kind: epic
planning_status: done
implementation_status: ready
contract_depends_on:
  - E1-S1
---

# E7-S1: Provide consistent command output

## Outcome

Coding agents can invoke TBTM through one predictable machine-readable boundary, while humans retain concise terminal output and actionable errors by default.

## User story

As a coding agent, I want stable JSON and exit behavior so that I can invoke commands programmatically.

## Product decisions

### Output modes

- Human-readable output remains the default.
- `--json` is one global option inherited by every current and future subcommand. It is accepted before, between, or after nested subcommands wherever Clap accepts global arguments.
- Repeating `--json`, including when both a wrapper and its caller add it, is valid and idempotent.
- E7-S1 standardizes the transport boundary; owning command contracts continue to define their successful `data` payloads and human presentation.

### JSON envelope and streams

- Every JSON-mode command result or parse failure emits exactly one compact JSON document to stdout and no other stdout or stderr text. Help and version requests are the explicit exception defined below.
- Successful query, mutation, empty-result, and valid no-op responses use:

  ```json
  {"ok":true,"data":{},"error":null}
  ```

- Errors use:

  ```json
  {
    "ok": false,
    "data": null,
    "error": {
      "code": "STABLE_ERROR_CODE",
      "message": "Human-readable explanation",
      "details": {}
    }
  }
  ```

- JSON errors are written to stdout so an agent has one response stream to parse. Human success and query output use stdout; human errors and recovery guidance use stderr.
- Empty-result shapes remain command-specific: for example, a list may return an empty array while an atomic select-and-claim operation may return `null`. Empty results are successful rather than not-found errors unless the owning command says otherwise.
- E7-S1 must preserve existing stable command payloads, error codes, error details, remediation guidance, and human output except where this contract explicitly resolves an inconsistency.

### Argument-parse failures

- The exact argv token `--json` requests JSON-mode parse errors even when the remainder of the command line is invalid. A typo such as `--jsoon` is not inferred as JSON mode.
- In a recognized JSON request, unknown commands or flags, missing required arguments, invalid values, and invalid flag placement return one normal error envelope on stdout, no stderr text, and exit code `2`.
- Help and version requests always retain Clap's ordinary human-readable text, streams, and exit `0`, even when an exact `--json` token is also present. This applies at the root and nested-command levels because help/version are CLI metadata requests rather than query, mutation, or error responses.
- Without an exact `--json` token, usage and parse errors also retain their ordinary human-readable behavior and streams.
- Repeated exact `--json` tokens do not themselves cause a parse failure.

### Confirmation and non-interactive operation

- JSON mode never prompts or reads an answer from stdin.
- A non-terminal invocation never prompts. An operation requiring confirmation must receive its documented explicit option such as `--yes`; otherwise it returns its existing confirmation-required error with exit code `2`.
- Human interactive cancellation remains a successful outcome with exit code `0` and its command-specific human message.
- Preview and dry-run operations remain non-interactive when their owning contract already permits them without confirmation.

### Exit behavior

E7-S1 preserves the repository-wide exit categories established by E1-S1 and later owning stories:

| Exit | Category |
|---:|---|
| 0 | Success, empty result, valid no-op, dry-run, or declined human confirmation |
| 1 | Operational or unexpected failure |
| 2 | Validation/state error, CLI parse error, or required confirmation missing |
| 3 | Not found |
| 4 | Conflict |
| 5 | Permission or forbidden operation |

Stable domain error codes continue to disambiguate cases within these categories. E7-S1 audits and centralizes mapping but does not redefine an owning story's error semantics.

### Partial uninstall

- A partially successful uninstall is one error result, never a success envelope followed by an error envelope.
- It keeps the existing `UNINSTALL_FAILED` or `UNINSTALL_PERMISSION_FAILED` code and exit precedence.
- `error.details` contains the complete existing `UninstallResult` projection so the user can see committed cleanup effects:

  ```json
  {
    "dryRun": false,
    "planned": [],
    "removed": [],
    "failed": [],
    "stealthLinesPlanned": 0,
    "stealthLinesRemoved": 0
  }
  ```

- These fields retain their E1-S1 meanings. Moving them into the single error envelope is a transport correction, not a change to cleanup behavior or filesystem atomicity.
- Human partial-failure output must likewise report what was planned, removed, and left unresolved, with actionable recovery guidance.

## Functional acceptance criteria

1. Every current query and mutation command accepts the inherited global `--json` option before or after its nested command path and returns its existing successful payload inside the common envelope.
2. Repeated `--json` tokens are accepted idempotently.
3. Every JSON-mode command result or parse failure emits exactly one parseable JSON document on stdout and emits no additional stdout or stderr text.
4. Success, command-specific empty result, valid no-op, validation/state error, not found, conflict, permission/forbidden, operational failure, and unexpected failure are distinguishable through the envelope, stable error code, and exit status.
5. An exact `--json` token causes recognizable command-line parse failures to use the JSON error envelope and exit `2`; a mistyped token receives ordinary human parsing behavior. Root and nested help/version requests remain human-readable and exit `0` with or without `--json`.
6. Human-readable output remains the default, with successful output on stdout and errors plus recovery guidance on stderr.
7. JSON and non-terminal operation never prompt; high-impact operations without their required explicit confirmation option fail with the existing confirmation-required behavior and exit `2`.
8. Interactive human cancellation remains successful with exit `0`; dry-run and preview behavior remains governed by the owning command contract.
9. Partial uninstall emits one error envelope whose details contain `dryRun`, `planned`, `removed`, `failed`, `stealthLinesPlanned`, and `stealthLinesRemoved`, without losing committed-effect or recovery information.
10. Existing domain payload shapes, stable error codes/details, human projections, and exit meanings remain unchanged except for the explicitly standardized global flag, parse-error, stream, and partial-uninstall behavior.

## Non-functional acceptance criteria

1. Output-mode selection, envelope serialization, stream routing, and process-exit mapping are centralized enough that a new command does not need an independent transport implementation.
2. JSON output remains deterministic and machine-parseable without scraping human text.
3. JSON serialization does not panic for supported response and error values; a serialization failure follows the operational/unexpected error boundary without emitting a partial document.
4. Agent-oriented operation performs no prompt or hidden stdin dependency when JSON mode or a non-terminal stdin is used.
5. Core continues to own domain results and typed errors; CLI continues to own argument parsing, rendering, stream selection, and process exit status.
6. The standardization adds no database migration, persistent state, network dependency, or concurrency behavior change.

## Verification

- Exercise one representative query and mutation from every current top-level command family with `--json` before and after nested subcommands and with repeated flags.
- Parse stdout as exactly one JSON value; assert stderr is empty and verify envelope invariants for success, empty/no-op, and every exit category from `0` through `5`.
- Exercise unknown command/flag, missing argument, invalid value, malformed flag placement, exact `--json`, and mistyped JSON flags.
- Exercise root and nested help/version requests with `--json` before and after the metadata flag; verify ordinary Clap text and exit `0`.
- Run high-impact commands with terminal and non-terminal stdin, JSON mode, missing confirmation, explicit confirmation, cancellation, preview, and dry-run.
- Inject or reproduce partial uninstall, operational, permission, and unexpected/serialization failures and verify no partial or duplicate JSON document is emitted.
- Preserve representative existing human output, error-details, remediation, and domain-payload snapshots.
- Run formatting, lint, and workspace tests through `mise`.

## Out of scope

- Adding repository, agent, task, status, relationship, claim, or comment capabilities; E7-S2 through E7-S5 own complete CLI exposure.
- Changing successful domain payloads, ordering, filtering, identity, persistence, or transaction semantics owned by prior stories.
- Introducing a new database schema, migration, configuration field, network protocol, logging format, or authentication boundary.
- Turning root or nested help/version output into the business-response envelope, including when `--json` is present.
- Implementing the Visual Studio Code extension.
- Creating, editing, reviewing, approving, or executing acceptance scenarios during planning or implementation.

## Implementation task

See [E7-S1-T1: Standardize the CLI output boundary](../tasks/E7-S1-T1-standardize-cli-output-boundary.md).
