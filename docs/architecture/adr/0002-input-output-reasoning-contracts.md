# ADR 0002: Input, Output, and Reasoning Contracts

## Status

Accepted

## Context

The application sends selected context to AI providers, local models, or coding
assistants. ADR 0001 defines design-first payload routing, but the project also
needs stable contracts for the shape of inputs, outputs, and reasoning evidence.

Without these contracts, agents may return useful-looking prose that cannot be
validated, traced to source evidence, reviewed by a human, or safely written
back into any trusted store.

## Decision

Define explicit input, output, and reasoning contracts for AI-assisted tasks.

Inputs must be normalized before they are sent to an AI or model adapter.

Outputs must be structured enough for the application to validate, reject,
review, or persist them without parsing free-form prose as business truth.

Reasoning must be represented as auditable evidence and decision metadata, not
as hidden chain-of-thought. The system should capture concise rationales,
citations, source spans, assumptions, rejected alternatives, confidence, and
review status when they affect user-visible content or persisted state.

## Contract Layers

Input contracts define:

- task type.
- user intent.
- allowed sources.
- selected context snippets.
- privacy and local-only constraints.
- expected output schema.
- rejection criteria.

Output contracts define:

- machine-readable result fields.
- source references.
- confidence or uncertainty.
- review status.
- warnings or blocked reasons.
- follow-up questions when required.

Reasoning contracts define:

- evidence used.
- assumptions made.
- constraints applied.
- concise decision rationale.
- alternatives rejected when relevant.
- fields that require human review.

## Consequences

Positive:

- AI output becomes testable and reviewable.
- The application can reject malformed or unsupported output.
- Persisted writes can preserve evidence.
- The UI can show uncertainty without exposing private model chain-of-thought.

Negative:

- Requires DTO and validation design before provider adapters are implemented.
- Some model responses may need repair or rejection instead of direct display.

## Enforcement

Code review should reject:

- use cases that treat free-form AI prose as trusted domain data.
- persisted AI-derived knowledge without source evidence or review status.
- provider adapters that return SDK-native responses directly to domain or
  use-case code.
- outputs that cannot be validated against a declared schema or contract.
