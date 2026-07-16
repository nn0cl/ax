# Runner CLI Contract for Slow External AI Jobs

This document is optional. It applies only to projects that run slow
external AI jobs (generation, batch inference, long training or evaluation
runs, and similar) where an agent would otherwise have to poll for
completion. It is net-new guidance motivated by the cost-reduction goal in
`docs/collaboration/llm-cost-reduction.md` ("avoid unnecessary use of
high-capability LLM coding services", "avoid broad context loading") —
`llm-cost-reduction.md` itself does not define a job runner shape.

A project with no such workload does not need this document.

## Why Detach Instead of Poll

An agent that polls a slow job burns reasoning cycles and context on a task
that is waiting, not thinking. A runner that can be told to detach lets the
agent (or the human) come back later instead of holding a turn open.

## CLI Surface

A runner satisfying this contract exposes:

- **plan**: given a set of inputs, print what would be submitted — item
  count, target provider/tool, task role, and any declared output contract —
  without submitting anything. Lets the caller check scope and cost before
  committing.
- **detach**: submit the job and return immediately with a resumable run
  identifier (see `docs/collaboration/ai-failure-recovery.md`) and wherever
  progress/events will be recorded, instead of blocking until completion.
- **status**: given a run identifier, report a short summary — do not
  require the caller to re-fetch full provider output just to check whether
  a run is still in progress.
- **resume**: given a run identifier, continue an interrupted or detached
  run. Resume must satisfy both guarantees defined in
  `ai-failure-recovery.md`: local record-level dedup, and provider-level
  request idempotency. A run in the `unconfirmed` state resumes
  automatically only when the provider's idempotency/safe-retry support is
  `verified` in `docs/collaboration/model-tool-capability-matrix.md`;
  otherwise resume pauses for a human decision.
- **dedupe**: given a run identifier, reconcile local candidate records
  against what the provider actually holds, without ever deleting or
  overwriting a record that already carries a human verdict.

## Non-Negotiable Rules

- A resumed run never deletes, duplicates, or silently overwrites a record
  that already carries a human verdict (accepted or rejected).
- A resumed run never auto-resubmits an `unconfirmed` job to the provider
  unless idempotency/safe-retry support for that provider is `verified`.
- `detach` must return a resumable run identifier immediately; it must not
  require the caller to keep a connection open.
- `status` must be cheap enough that an agent can call it without triggering
  the same cost/reasoning concerns this contract exists to avoid.

## What This Document Does Not Define

- The specific provider, SDK, or job type — those are project-specific.
- A billing or token-accounting mechanism — see the non-goals in
  `docs/collaboration/llm-cost-reduction.md`.
- The concrete storage format for run records — only that a resumable run
  identifier and the two resume guarantees above exist.
