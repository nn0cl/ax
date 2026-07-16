# LLM Cost and Reasoning Control

This document defines lightweight practices for reducing unnecessary use of
high-capability LLM coding services. It does not define billing integration,
model pricing, or telemetry infrastructure.

## Goal

Reduce avoidable LLM cost by making agents:

- choose the smallest safe operating path.
- prefer deterministic tools for facts and verification.
- avoid broad context loading.
- record why stronger reasoning was needed.
- make rework visible when AI output caused it.

## Non-Goals

- Billing-grade exact token accounting.
- Guessing unavailable token usage.
- Choosing specific commercial models.
- Building a centralized cost dashboard.
- Requiring private prompts, provider logs, or billing exports in the repo.

## Slow External AI Jobs

Projects that run slow external AI jobs (generation, batch inference, long
training or evaluation runs) should avoid having an agent poll for
completion, since polling burns reasoning cycles on waiting rather than
working. See `docs/collaboration/runner-cli-contract.md` for an optional,
concrete CLI contract (plan/detach/status/resume/dedupe) covering this case.

## Cost Control Signals

Each substantial trace should record:

- `Operating path`: Fast Path, Feature Path, or Architecture Path.
- `Files read`: approximate count or short list.
- `Context intentionally omitted`: important context classes that were not
  loaded.
- `Deterministic checks used`: commands, tests, formatters, linters, or search.
- `Escalation reason`: why a stronger model or deeper reasoning was needed.
- `Avoided LLM work`: work handled by tools, existing specs, or narrow context.
- `Rework caused by AI output`: none, minor, or a short description.

Keep these entries short. The goal is trend visibility, not accounting
precision.

## AI Planning Estimates

For planned work with size `M`, `L`, or `XL`, record a vendor-neutral planning
estimate in the canonical local issue or work plan.

Each planning record should include:

- record id and status.
- authoring agent and environment.
- displayed model name and reasoning setting, or `N/A` with reason.
- planned size and intended route.
- estimated token range and midpoint, or another available usage metric.
- estimation basis, assumptions, and confidence.
- revision links when another agent updates the estimate.

Use the model and reasoning names shown by the executing environment. Do not
normalize names across vendors or overwrite another agent's estimate without a
new revision record.

## AI Execution Records

For trace-required work, record actual AI usage by attempt in the AI work
trace. Use `N/A` with a reason when the environment does not expose a stable
usage value.

Each attempt should keep the accepted planning estimate beside the actual
usage fields. When actual usage is unavailable, do not infer a count. If the
work clearly expanded or contracted from the estimate, record the variance
reason without calculating an exact variance.

Do not combine token values across attempts as the primary record. A reference
total is allowed only when the token metric, source, and attribution boundary
are compatible and stated.

These records are planning and review evidence. They are not billing records.

## Review Questions

During review, ask:

- Could this have used Fast Path?
- Did the agent read more files than the task needed?
- Was a deterministic tool available for a fact the model reasoned about?
- Was escalation to a stronger reasoning agent justified?
- Did missing specification or unclear phase cause avoidable rework?

## Healthy Trends

Over time, a project should see:

- more mechanical work completed through Fast Path.
- fewer architecture-path escalations for local edits.
- traces that clearly explain why strong reasoning was used.
- fewer broad context dumps.
- fewer changes caused by guessed requirements.

## Warning Signs

Investigate when:

- many traces say the operating path was Architecture Path for small edits.
- agents repeatedly read whole directories for local changes.
- verification is described in prose but no deterministic check was run.
- AI output causes repeated rework or test changes.
- Adjudicator review repeatedly rejects work for phase or scope drift.
