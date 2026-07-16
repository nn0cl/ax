# Input, Output, and Reasoning Contracts

Use this document when a task sends context to AI, cloud providers, local
models, semantic extraction, or code agents.

The goal is not to expose hidden model chain-of-thought. The goal is to make
inputs compact, outputs structured, and reasoning evidence auditable.

## Missing Design Areas This Covers

Clean Architecture, AT-TDD, payload routing, and provider boundaries define
the rest of the process. The remaining design gaps this document covers are:

- normalized task input envelopes.
- output schemas per task type.
- source span and citation representation.
- confidence and review-state rules.
- rationale and assumption summaries.
- rejection and repair behavior for malformed AI output.
- evaluation datasets and golden examples.
- token and privacy budgets for prompts.
- UI presentation rules for uncertain or unreviewed knowledge.

For adopting AI-generated or human-sourced external resources (not software
dependencies) into trusted use, see the optional `docs/architecture/
external-resource-adoption-contract.md`, which extends this document's
Output Contract with an explicit adoption-check lifecycle.

## Input Contract

Every AI-assisted task should receive an input envelope instead of ad hoc text.

Recommended fields:

- `task_type`: what the model is being asked to do.
- `user_intent`: the user or developer intent.
- `constraints`: local-only, privacy, allowed providers, phase boundary.
- `context_items`: selected snippets with source identifiers.
- `excluded_context`: intentionally omitted context classes.
- `expected_output`: output schema or DTO name.
- `rejection_rules`: conditions that make the response unusable.

Examples of `task_type` (replace with your project's actual AI-assisted
tasks):

- `<summarize_fragment>`
- `<extract_candidate_terms>`
- `<explain_term>`
- `generate_phase_1_tests`
- `review_architecture_boundary`

## Output Contract

Outputs should be parsed as structured data before use-case code accepts them.

Recommended fields:

- `result_type`
- `items`
- `source_refs`
- `confidence`
- `review_status`
- `warnings`
- `blocked_reason`
- `requires_human_review`

Rules:

- Free-form prose may be displayed as explanation, but it must not become
  trusted domain data by itself.
- AI-derived knowledge defaults to unreviewed unless the user explicitly
  approves it or an accepted spec says otherwise.
- Output that omits required source references must not be written as
  approved knowledge to any trusted store.

## Reasoning Evidence Contract

Do not ask models to expose hidden chain-of-thought. Ask for concise, auditable
decision metadata.

Recommended fields:

- `evidence_refs`: source snippets, spans, record IDs, or fact IDs.
- `assumptions`: assumptions that affected the result.
- `constraints_applied`: privacy, phase, trust, provider, or schema constraints.
- `rationale_summary`: short explanation of why the result was chosen.
- `rejected_alternatives`: optional list when alternatives materially mattered.
- `review_points`: what a human should verify.

Reasoning evidence belongs in application output, review UI, or trace logs when
it affects user-visible content or persisted knowledge.

## Source Reference Shape

Use a stable source reference object for traceability.

Recommended fields:

- `source_id`
- `source_type`: e.g. document, record, provider response, or test
  specification.
- `span`: optional start/end offsets or time range.
- `quote`: short excerpt when allowed.
- `uri_or_path`: optional location when safe to expose.
- `captured_at`: timestamp when available.

## Optimization Goals

Optimize inputs for:

- relevance.
- privacy.
- phase boundary.
- minimal token use.
- enough examples to preserve style.

Optimize outputs for:

- schema validation.
- human review.
- traceability.
- deterministic downstream processing.
- graceful rejection.

Optimize reasoning evidence for:

- short review time.
- source-backed claims.
- visible uncertainty.
- clear handoff between AI and deterministic code.

## Validation Rules

Provider adapters must validate output before returning it to use cases.

Reject or mark blocked when:

- required fields are missing.
- source references are absent for knowledge claims.
- confidence is outside the accepted range.
- review status is missing.
- the response violates local-only or provider constraints.
- the response asks to perform work outside the current AT-TDD phase.

## AT-TDD Guidance

Phase 1 tests should assert contracts at the boundary:

- input envelope contains only selected context.
- secrets and omitted context are absent.
- output is rejected when required fields are missing.
- unreviewed knowledge is not persisted as trusted data.
- evidence references are preserved.

Phase 2 implementation should keep validation minimal and close to the tested
contract.

Phase 3 refactor may extract reusable validators, DTO mappers, and trace
formatters without changing behavior.
