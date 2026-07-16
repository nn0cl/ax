# AI Failure and Recovery Procedure

This document resolves Gap Register #7
(`docs/collaboration/process-gap-register.md`): the process defines when to
stop, but not how to recover from a bad, interrupted, or unconfirmed AI turn.

It applies to any AI-assisted task, not only code generation — including
long-running external AI jobs (see `docs/collaboration/
runner-cli-contract.md` for the optional CLI contract that uses this
document's states). It does not name any specific commercial provider,
model, or tool.

## Failure Categories

Classify a failure into exactly one of these five categories. Each includes
one short example pattern.

### 1. Input Contract Violation

The request sent to the provider or tool violates its own declared input
contract (see `docs/architecture/io-reasoning-contracts.md`).

Example: a required field in the input envelope is missing or malformed
before the request is even sent.

### 2. Provider Connection Failure

The request could not reach the provider, or the provider could not be
reached long enough to get a definitive result.

Example: a transient network error, timeout, or an unreachable endpoint.

### 3. Workflow/Tool Compatibility Failure

The request reached the provider, but the provider or tool rejected it, or
produced a result inconsistent with the workflow that submitted it, because
of a mismatch between what the workflow assumed and what the provider/tool
actually supports.

Example: a workflow built for one configuration is run against a provider or
tool version that no longer accepts it, or accepts it but interprets it
differently.

### 4. Output Retrieval Failure

The provider completed the work, but the result could not be retrieved
correctly — the wrong artifact was fetched, retrieval was partial, or a
derived/secondary output was confused with the primary one.

Example: fetching an intermediate or derived result instead of the final
declared output.

### 5. Human Review Rejection

The output was retrieved successfully but a human reviewer rejected it.

Example: the result is well-formed and retrievable but does not meet the
review criteria for its task.

## Tie-Break Rule: Input Contract vs. Workflow/Tool Compatibility

These two categories can look similar when a request that used to succeed
starts failing. Use this rule:

- If the caller's input is unchanged, and the provider's or tool's
  acceptance criteria changed underneath it, classify as **workflow/tool
  compatibility failure**.
- If the caller's input violates its own declared input contract (including
  when it always did, or the contract itself changed on the caller's side),
  classify as **input contract violation**.

## Recovery Concepts

### Last Trusted Artifact

The most recent artifact, record, or state that a human or a deterministic
check has already confirmed as valid. Recovery always starts from here, not
from an assumed midpoint.

### Resumable Run Identifier

Every recoverable unit of work (a run, an experiment, a job) has a stable
identifier that recovery, resume, and dedup logic can reference. This
identifier is distinct from any provider-assigned request ID, since a
provider may not expose one, or may expose one that changes across retries.

### What Changed Between Attempts

Record what changed between the last trusted state and the current attempt:
inputs, provider/tool configuration, and workflow version. An attempt that
changes nothing should not fail differently from the one before it; if it
does, the difference is itself diagnostic information and must be recorded,
not discarded.

### Resume Conditions

A run may resume only when:

- the last trusted artifact is identified.
- the resumable run identifier is known.
- what changed since the last trusted state is recorded (or explicitly
  "nothing changed").
- the local-dedup and provider-idempotency rules below are satisfied.

## Resume Guarantees: Local Dedup vs. Provider Idempotency

Resuming an interrupted or detached run must satisfy two distinct
guarantees. Meeting one does not satisfy the other.

### Local Record-Level Dedup

Re-running a job adds only unreviewed candidate records. A record that
already carries a human verdict (accepted or rejected) is never deleted,
duplicated, or silently overwritten by a resume.

### Provider-Level Request Idempotency

The same job is not resubmitted to the provider a second time when the
outcome of the first submission cannot be confirmed. A run whose last
submission's outcome is unknown enters an explicit `unconfirmed` state,
distinct from `completed` and `failed`.

Automatic resume from `unconfirmed` is allowed only when the provider's
idempotency-key or safe-retry support is `verified` in
`docs/collaboration/model-tool-capability-matrix.md`. When it is `inferred`
or `unknown`, the run pauses and waits for a human decision instead of
auto-resubmitting. This prevents a resume from silently duplicating a
provider-side charge or side effect.

## Documenting Rejected AI Output

When a human rejects AI output (category 5), record:

- which resumable run identifier produced it.
- the rejection reason.
- whether the rejection implies the input, the provider/tool configuration,
  or the workflow needs to change before the next attempt.

Do not silently discard a rejected attempt's record — the next attempt must
be able to see what was already tried and rejected, so it does not repeat
the same failure.
