---
id: E7-S1-T1
kind: implementation_task
planning_status: done
implementation_status: ready
depends_on:
  - E1-S1-T1
implements:
  - E7-S1
---

# E7-S1-T1: Standardize the CLI output boundary

## Epic

[E7-S1: Provide consistent command output](../epics/E7-S1-provide-consistent-command-output.md)

## Objective

Consolidate CLI output-mode selection, parsing failures, JSON envelope emission, stream discipline, and exit handling so every current command presents one predictable programmatic boundary without changing its domain behavior or successful payload.

## Readiness

E1-S1-T1 is complete and supplies the authoritative JSON envelope, initial exit categories, confirmation conventions, and partial-uninstall result. The current CLI already renders domain-specific human and JSON results; this task standardizes the shared transport around them. Later E7 stories can extend command coverage on the resulting boundary.

## Deliverables

- One inherited global, idempotent `--json` option for the full Clap command tree.
- A fallible, centralized response renderer for success and error envelopes with explicit stdout/stderr routing.
- JSON-aware handling of Clap parse failures when the exact argv token `--json` is present.
- One-envelope partial-uninstall failure reporting with the full existing cleanup result in `error.details`.
- Cross-cutting integration tests covering every current top-level command family, parse failures, streams, exit categories, confirmation behavior, and duplicate-output regressions.
- Preservation of existing command-specific payloads, typed errors, human renderers, and recovery guidance.

No domain command, migration, persistent model, or acceptance-scenario change is part of this task.

## Proposed structure

Keep the core/CLI boundary and extract transport concerns within the CLI crate as needed:

```text
crates/tbtm-cli/src/main.rs
crates/tbtm-cli/src/output.rs          # optional shared response boundary
crates/tbtm-cli/tests/command_output.rs
```

The exact module split is implementation-owned. Avoid moving domain operations or result construction out of `tbtm-core`; avoid duplicating every owning story's payload tests in the new cross-cutting suite.

## Technical choices

- Parse with a fallible Clap entry point rather than allowing `Cli::parse()` to terminate before the selected output mode can be applied.
- Detect a JSON parse-error request from the exact raw argv token `--json`. Do not infer typos or substring matches.
- Declare `--json` as inherited/global and configure repeated occurrences as idempotent. Remove per-leaf mode plumbing once the global mode is authoritative.
- Represent a completed invocation as one response outcome and render it once. Do not print an optimistic result before final exit/error classification.
- Keep JSON errors on stdout and keep stderr empty in JSON mode. Preserve human success on stdout and human errors/recovery guidance on stderr.
- Make envelope serialization fallible. Serialize the complete document before writing it so a serialization error cannot leave a partial JSON prefix on stdout.
- Keep the established `ok/data/error` shape. Successful `data` stays strongly typed/serializable; command-specific empty and no-op shapes remain unchanged.
- Preserve the core `exit_code` categories and stable domain codes. CLI parse errors map to a stable validation-style error response and exit `2` without entering domain execution.
- Model partial uninstall as a typed CLI result/error carrying the complete `UninstallResult`, or an equivalent structured boundary. Do not reconstruct cleanup state from rendered text.

## Implementation flow

### 1. Centralize output-mode selection

- Add one global `--json` option inherited throughout the Clap tree.
- Ensure it works before, between, and after nested subcommands and accepts repeated exact occurrences.
- Replace the exhaustive per-command `command_uses_json` traversal and leaf-local JSON flags with one authoritative mode value.
- Preserve root and nested help/version as ordinary Clap text with exit `0`, even when `--json` is also present.

### 2. Own parse failures

- Capture raw argv and determine whether it contains an exact `--json` token.
- Use fallible parsing and route parse failures through either the JSON response boundary or ordinary Clap human output.
- Normalize recognized JSON parse failures to one error envelope, empty stderr, and exit `2` for unknown commands/flags, missing required arguments, invalid values, and invalid placement.
- Treat Clap's help/version display outcomes as successful CLI metadata requests rather than parse failures; render their normal text and exit `0` regardless of `--json` placement.
- Do not execute repository discovery or domain operations after a parse failure.

### 3. Render one finalized response

- Separate domain execution from final transport rendering enough to determine the response and exit status before emitting bytes.
- Centralize envelope construction and stream selection while retaining command-specific human render functions.
- Ensure JSON success and failure each produce one compact document plus the normal trailing newline, with no prompts, previews, warnings, or diagnostics outside it.
- Preserve error `details` and human recovery guidance supplied by typed core errors.

### 4. Correct partial uninstall output

- Retain E1-S1 best-effort cleanup and failure precedence.
- When `UninstallResult.failed` is non-empty, produce one `UNINSTALL_FAILED` or `UNINSTALL_PERMISSION_FAILED` response rather than rendering success before returning an error.
- Put the serialized existing fields `dryRun`, `planned`, `removed`, `failed`, `stealthLinesPlanned`, and `stealthLinesRemoved` in `error.details`.
- Preserve equivalent human visibility into removed and remaining artifacts and suggested recovery.

### 5. Enforce non-interactive behavior

- Route every confirmation path through the selected invocation mode and terminal check.
- JSON and non-terminal paths must never call a prompt/read path. Missing explicit confirmation returns the owning confirmation-required error and exit `2`.
- Preserve human TTY confirmation, cancellation exit `0`, and owning preview/dry-run exceptions.

### 6. Add cross-cutting regressions

- Add a compact command-family matrix rather than reproducing full domain suites.
- Retain existing focused tests as the authority for payload fields, ordering, validation precedence, and domain effects.
- Add direct assertions for exact JSON document count, stdout/stderr separation, mode placement/repetition, parse errors, exit category coverage, and absence of prompt text.

## Output and error contract

### Success

```json
{"ok":true,"data":{},"error":null}
```

- Exactly one document on stdout; stderr empty.
- `data` is the unchanged owning-command payload, including its empty or no-op representation.
- Exit `0`.

### Error

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

- Exactly one document on stdout; stderr empty.
- Exit is selected from the stable `1`–`5` taxonomy.
- Parse errors requested with exact `--json` use this shape and exit `2`.
- Partial uninstall uses its existing failure code and puts the complete `UninstallResult` projection in `details`.

### Human mode

- Success, empty/no-op, previews, and interactive prompts use stdout as defined by their command renderers.
- Errors and actionable recovery guidance use stderr.
- Root and nested Clap help/version always retain human-readable behavior and exit `0`, including when exact `--json` is present. Ordinary parse failures do so only when exact `--json` is absent.

## Test plan

### Parser and mode tests

- Exercise global `--json` before the top-level command, between nested subcommands where supported, after the leaf arguments, and in repeated combinations.
- Cover every current top-level family: init/uninstall, repo, agent, status, and task, with representative nested task operations.
- Cover unknown command, unknown flag, missing required argument, invalid value, and malformed placement with exact `--json`; assert one envelope, empty stderr, and exit `2`.
- Repeat representative parse failures with no flag and with `--jsoon`; assert ordinary human Clap output rather than a JSON envelope.
- Cover root and nested `--help`/`--version` combinations with `--json` before and after the metadata flag; assert ordinary Clap text and exit `0` rather than an envelope.

### Envelope and stream tests

- Parse stdout as one and only one complete JSON value for representative query, mutation, empty result, valid no-op, and errors at exits `1` through `5`.
- Assert the common `ok/data/error` invariants and no extra stdout lines or stderr bytes in JSON mode.
- Assert human success stays on stdout and human errors plus existing remediation stay on stderr.
- Verify domain payloads and error details against representative existing integration assertions rather than redefining them.

### Confirmation and failure tests

- Exercise each high-impact confirmation family with JSON, piped/non-terminal stdin, human TTY where practical, `--yes`, cancellation, dry-run, and preview.
- Assert JSON/non-terminal requests never block or consume stdin and missing confirmation exits `2`.
- Inject partial uninstall with mixed removed/failed paths and `.gitignore` outcomes; assert one error document contains the exact committed cleanup result and proper exit precedence.
- Exercise representative operational, permission/forbidden, conflict, not-found, validation, and unexpected/serialization paths without partial output.

### Regression tests

- Keep existing command-family human/JSON snapshots and exit assertions passing unless this epic explicitly changes the transport.
- Verify no repository or database mutation occurs for parser failures.
- Verify the refactor does not change domain transactions, ordering, filters, identities, or concurrency behavior.

## Verification commands

```bash
rtk mise run format
rtk mise run lint
rtk mise run test
```

Smoke-test global/repeated JSON flags, JSON parse errors, one representative command in every top-level family, non-interactive confirmation, and partial uninstall in isolated temporary repositories.

## Acceptance scenario impact

Classify as `none`, `revalidate`, `add`, or `supersede` only after implementation from the actual public behavior changed. A non-`none` result is a handoff to the separate acceptance-scenario workflow; do not edit or execute scenario files here.

## Definition of done

- All E7-S1 functional and non-functional acceptance criteria pass.
- Every current command inherits one idempotent global `--json` option.
- Recognized JSON parse failures and domain results emit exactly one envelope on stdout with empty stderr.
- Root and nested help/version retain ordinary Clap text and exit `0`, even when `--json` is present.
- Human mode preserves readable success, errors, and recovery guidance on the correct streams.
- JSON and non-terminal operation never prompt; confirmation and cancellation behavior retains owning contracts.
- Partial uninstall reports one failure envelope with complete cleanup effects and correct exit precedence.
- Existing domain payloads, error codes/details, state changes, transactions, ordering, and concurrency behavior remain stable.
- No migration, persistent model, new domain command, or acceptance-scenario edit is introduced.
- Formatting, Clippy with warnings denied, and all workspace tests pass.

## References

- [E7-S1 epic](../epics/E7-S1-provide-consistent-command-output.md)
- [PRD product goals](../PRD.md#41-primary-goals)
- [PRD CLI requirements](../PRD.md#fr-11-cli-experience)
- [PRD usability and observability](../PRD.md#124-usability-and-observability)
- [PRD MVP acceptance criteria](../PRD.md#13-mvp-acceptance-criteria)
- [PRD E7-S1 story](../PRD.md#story-e7-s1-provide-consistent-command-output)
- [E1-S1 repository initialization contract](../epics/E1-S1-initialize-repository.md)
- [E1-S1 implementation task](E1-S1-T1-implement-repository-initialization.md)
