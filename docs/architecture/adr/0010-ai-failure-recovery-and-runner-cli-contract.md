# ADR 0010: AI Failure Recovery and Slow-Job Runner CLI Contract

## Status

Accepted

## Context

Gap Register #7 (`docs/collaboration/process-gap-register.md`) has tracked
"AI Failure and Recovery Procedure" as `Status: future` since the template's
initial gap survey: the process defines when to stop, but not how to recover
from a bad or interrupted AI turn.

External feedback from real usage of this template (Adjudicator-supplied,
2026-07-13, based on a Studio integration running long, detachable AI
generation jobs) confirmed the gap in practice and surfaced two related
problems that the existing documents do not cover:

- `docs/collaboration/model-tool-capability-matrix.md` routes tasks by
  capability class, but does not record whether a specific provider/model/
  tool configuration was actually verified to work, as opposed to assumed
  from a model card or generic documentation.
- `docs/collaboration/llm-cost-reduction.md` states the goal of reducing
  avoidable LLM cost and reasoning overhead, but has no guidance at all for
  slow external AI jobs specifically — no CLI contract, and no safe resume
  behavior. Without one, resuming a slow job risks either duplicate local
  candidate records or duplicate provider-side submissions (and cost).

## Decision

1. Add `docs/collaboration/ai-failure-recovery.md`, resolving Gap Register
   #7. It classifies AI-assisted job failures into five categories: input
   contract violation, provider connection failure, workflow/tool
   compatibility failure, output retrieval failure, and human review
   rejection. Each category includes one short example failure pattern. The
   boundary between input contract violation and workflow/tool compatibility
   failure is resolved by a stated tie-break rule: if the caller's input is
   unchanged but the provider/tool's acceptance criteria changed, classify as
   workflow/tool compatibility; if the caller's input violates its own
   declared input contract, classify as input contract violation.
2. Recovery is defined around: a last-trusted-artifact concept, a resumable
   experiment/run identifier, an explicit record of what changed between
   attempts, and stated resume conditions. None of this may be copied from,
   or reference, any specific commercial provider, model, or generation tool.
3. Resume/dedupe behavior distinguishes two separate guarantees:
   - Local record-level dedup: re-running a job adds only unreviewed
     candidates; records that already carry a human verdict (accepted or
     rejected) are never deleted or silently overwritten.
   - Provider-level request idempotency: the same job is not resubmitted to
     the provider a second time when the first submission's outcome is
     unconfirmed. A run whose last submission's outcome cannot be confirmed
     enters an explicit `unconfirmed` state, distinct from `completed` and
     `failed`.
4. Automatic resume from `unconfirmed` is allowed only when the provider's
   idempotency-key or safe-retry support is `verified` (see item 5). When it
   is `inferred` or `unknown`, the run pauses for a human decision instead of
   auto-resubmitting.
5. `docs/collaboration/model-tool-capability-matrix.md` gains a
   verified/inferred/unknown compatibility state, applied to the existing
   capability classes and routing table rather than as a parallel table.
   "Verified" means the specific configuration was actually exercised and
   confirmed; "inferred" means it is assumed from documentation or a model
   card without direct confirmation; "unknown" means it has not been checked.
6. Add `docs/collaboration/runner-cli-contract.md` as its own document,
   defining an optional CLI contract (plan/detach/status/resume/dedupe) for
   any slow external AI job, not limited to image generation. It is
   referenced from, but not merged into, `llm-cost-reduction.md`, because the
   CLI contract changes for reasons unrelated to cost-reduction policy and
   coupling the two would make either document's edit history noisy for the
   other concern.
7. All of the above is additive and optional: a project with no slow
   external AI job and no unverified-provider workload is not required to
   adopt the runner CLI contract or the compatibility-state tracking, and
   nothing in this ADR changes any existing mandatory phase gate.

## Consequences

Positive:

- Gap Register #7 is closed with a concrete, vendor-neutral procedure.
- Resume behavior after a detached or interrupted job cannot silently double
  a provider charge/side effect or destroy a human-reviewed verdict.
- Capability assumptions become visible (verified/inferred/unknown) instead
  of implicit, reducing silent misrouting to an unverified configuration.
- The runner CLI contract gives agents a documented reason to detach instead
  of polling, reinforcing the existing cost-reduction goal.

Negative:

- Adds a new document (`ai-failure-recovery.md`) and a new document
  (`runner-cli-contract.md`) for adopters to be aware of, even though both
  are optional.
- Projects that adopt the runner CLI contract must implement the
  plan/detach/status/resume/dedupe surface themselves; the ADR defines the
  contract, not an implementation.
- Requires providers/tools to be explicitly marked verified/inferred/unknown,
  which is extra bookkeeping compared to assuming compatibility.

## Enforcement

Code review should reject:

- automatic resume of an `unconfirmed` run when the provider's idempotency/
  safe-retry support is not `verified`.
- resume logic that deletes, overwrites, or silently merges a candidate
  record that already carries a human verdict.
- new runner CLI contract features added directly to `llm-cost-reduction.md`
  instead of `runner-cli-contract.md`.
- failure-recovery documentation that names a specific commercial provider,
  model, or generation tool as part of the general procedure.
- capability routing decisions that treat an `inferred` or `unknown`
  compatibility state as if it were `verified`.
