# AI Work Trace Log

AI work traces record the important process choices made during AI-assisted
development. They are not application logs and must not contain secrets or
full exports of private data.

## Location

Store task traces under:

```text
docs/collaboration/traces/
```

Use one Markdown file per substantial task:

```text
YYYY-MM-DD-short-task-name.md
```

Keep `docs/collaboration/traces/.gitkeep` so the folder exists before the first
trace is created.

## When Required

Create or update a trace when:

- a task changes agent instructions, templates, ADRs, or collaboration rules.
- a planned task is size `M`, `L`, or `XL`.
- a bug fix reaches a second execution attempt, even if it originally looked
  like size `S`.
- a task spans more than one phase.
- a task uses external AI, cloud providers, or non-default model routing.
- a task is paused and another agent may resume it.
- the Adjudicator asks for an audit trail.

Trace is optional for tiny documentation-only changes when the final response
already includes enough context.

An execution attempt is one cohesive run against the current plan. Start a new
attempt when deterministic verification fails and replanning is needed, work is
stopped unresolved and later resumed, the executing environment changes, or a
materially different plan replaces the previous one. Individual chat messages,
tool calls, formatting, linting, or tests within the same run are not separate
attempts.

## Trace Contents

Each trace should include:

- user request.
- current phase.
- canonical issue or work plan.
- AI planning record reference, when one exists.
- included context.
- omitted context.
- model, assistant, or deterministic tool routing.
- execution records by attempt, including scope and result.
- operating path and cost/reasoning control signals.
- Adjudicator decisions.
- assumptions.
- open decisions.
- verification run.
- changed files.
- next safe action.

See `docs/collaboration/llm-cost-reduction.md` for the lightweight cost and
reasoning control fields used in traces. A trace that adopts an
AI-generated or externally-sourced resource may also reference
`docs/architecture/external-resource-adoption-contract.md`'s check-record
fields.

For AI execution records:

- Record the environment, displayed model name, and displayed reasoning
  setting when available.
- If token usage is unavailable, record `N/A` and the reason. Do not estimate
  actual usage.
- Do not replace per-attempt records with a combined total.
- If a reference total is useful, include it only when the metric, source, and
  attribution boundary are compatible and clearly stated.
- Distinguish issue-only work from combined work in the attempt scope.

## Privacy Rules

Do not put these in a trace:

- secrets or API keys.
- full `.env` values.
- full exports of private user or business data.
- provider raw responses unless explicitly approved.

Use short excerpts and source references instead.
