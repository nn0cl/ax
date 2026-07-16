# ADR 0009: Bug Planning and AI Usage Records

## Status

Accepted

## Context

Experience from adopting projects showed that difficult bug fixes can require
multiple AI attempts, yet the existing planning rules focus on feature work.
Without durable bug planning, later agents cannot reliably distinguish a local
correction from a problem with uncertain scope, dependencies, or verification.

The existing cost policy deliberately avoids billing telemetry and exact token
accounting. It does not, however, provide a vendor-neutral way to preserve
best-effort planning estimates, execution records, or revisions made when one
AI agent hands work to another.

## Decision

1. Record a discovered bug in a local issue or an existing work plan before
   fixing it, except when it is a one-attempt, planning-size `S` bug already
   covered by the current approved scope.
2. The `S` exception waives only a separate issue or work plan. It never
   waives design intake, acceptance evidence, phase gates, or deterministic
   verification.
3. Add vendor-neutral planning sizes `S`, `M`, `L`, `XL`, and `TBD`. The size
   represents scope, uncertainty, dependencies, and verification effort, not
   elapsed-time commitment.
4. Require versioned AI planning records for `M`, `L`, and `XL` work. A record
   identifies its authoring environment and displayed model/reasoning names,
   intended execution route and scope, token estimate range and midpoint when
   available, basis, assumptions, and confidence. Revisions append a new record
   and link to the superseded record rather than overwriting it.
5. Require an AI work trace for planning-size `M` or larger, a second or later
   execution attempt, non-default model routing or an AI handoff that changes
   the plan, and all existing trace-required cases.
6. Preserve execution usage per attempt. Each attempt should reference the
   planning estimate, record actual usage when exposed by the environment, and
   use `N/A` with a reason when actual usage is unavailable. Never replace
   attempt records with an aggregate. A reference total may be shown only when
   token metric, source, and attribution are compatible.
7. Use `N/A` with a reason whenever a model, reasoning setting, or token value
   is unavailable. Do not infer or fabricate provider usage.
8. When exact actual usage is unavailable but meaningful variance from the
   estimate is observed or suspected, record the variance reason without
   inventing a token count.
9. Keep the policy vendor-neutral. Claude, Codex, Cursor, Copilot, Grok, or a
   future environment use the same record shape.

An attempt is a cohesive AI execution toward resolving one problem. A failed
deterministic verification followed by replanning, resumption after an
unresolved stop, a change of executing environment, or material replacement
of the plan starts a new attempt. Individual messages, tool calls, formatting,
linting, and test commands within the same execution do not.

## Consequences

Positive:

- Difficult bugs and failed approaches remain visible to later agents.
- Planning estimates can be revised by another AI without erasing provenance.
- Size and usage records remain comparable without assuming a specific vendor.
- Small, one-attempt fixes do not require a separate planning artifact.

Negative:

- `M` or larger bugs add planning and trace overhead.
- Token values from different environments may not be comparable.
- Agents must maintain record links and re-triage work when attempts fail.

## Enforcement

Code review should reject:

- bug fixes that use the planning exception to skip a phase gate.
- `M`, `L`, or `XL` work without a versioned AI planning record and trace.
- second or later attempts that are not split into attempt records.
- AI planning revisions that silently overwrite prior estimates.
- guessed model, reasoning, or token values where the environment did not
  expose them.
- omitted variance reasons when the task clearly expanded beyond or shrank
  below the planning estimate.
- totals that combine incompatible token metrics, sources, or attribution.
