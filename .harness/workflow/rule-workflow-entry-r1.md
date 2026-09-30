---
schemaVersion: 1
kind: constitution-rule
id: rule-workflow-entry
revision: 1
title: Enter the harness workflow
category: workflow
status: active
scope: {repository: true}
contextLoading: always
createdOn: 2026-09-21
createdBy: framework
origin: framework
installationApproval: {approvedBy: user, approvedOn: 2026-09-21, rulesetVersion: 1.11.0}
---
## Rule
After loading the always-context Constitution rules, invoke the bootstrap
Intent workflow for a new work intent that has no confirmed interpretation.
Intent loads Current Truth only after confirmed `ready`; Current Truth loads
Risk Router only after `ready` or a routable `unresolved`. Continue an
established intent without restarting it. Re-enter Intent for a new goal or
material goal/scope change; return Current Truth `insufficient_query` to the
same Intent.

## Rationale
The entry boundary must be discoverable from the Constitution while keeping
the repository entry file short and ordinary conversation free of repeated
startup work.

## Application

```mermaid
flowchart TD
    A[User input after rule lookup] --> B{New work intent without confirmation?}
    B -- Yes --> I[Invoke Intent workflow]
    B -- No --> C{New goal or material goal/scope change?}
    C -- Yes --> I
    C -- No --> D{Current Truth returned insufficient_query?}
    D -- Yes --> R[Return to same Intent for clarification]
    D -- No --> K[Continue established conversation]
```

Use `inspect-context` for repository startup; it loads Workflow Entry and
Intent, but not Current Truth, Risk Router, or routed workflows. Intent and
Current Truth own their successor lookups. After Router selects an on-demand
workflow, use `inspect-workflow` for that stable rule ID and the current target
paths before entering it. Use the request and conversation state to distinguish new work from an answer,
correction, status request, or continuation. A confirmation reply completes the
pending Intent; it is not a new request. Reinspect effective rules when target
paths or material rules change, not on every message.

**Trigger bad:** Treat “yes” in response to Intent's confirmation as a new
intent and ask for another interpretation. **Trigger good:** Confirm the
existing interpretation and let it proceed.

**Handoff bad:** Send a new goal straight to Current Truth using the previous
goal's scope. **Handoff good:** Invoke Intent for the new goal; only its
confirmed `ready` result supplies Current Truth's information need and scope.
---
