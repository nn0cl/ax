# ADR 0001: Design-First AI Request Routing

## Status

Accepted

## Context

The project uses AI agents and code assistants for design, tests, and
implementation. Sending every available document, schema, and record to an
AI request increases hallucination risk, cost, latency, and privacy exposure.

Some tasks are lightweight and should be clarified during design before
implementation begins. Examples include value object definitions, DTO names,
port boundaries, test scope, and whether a task can be handled by a smaller
model or a code-completion assistant.

## Decision

Every user request must start with a design step before tests or implementation.

The design step must decide:

- acceptance behavior being targeted.
- value objects and DTOs that are lightweight enough to define before coding.
- ports and adapters involved.
- files and documents that are relevant to the task.
- AI request payload slices to include or omit.
- model or assistant class suitable for each subtask.
- ambiguity boundaries that must not be guessed.

AI request payloads must be selected by need. Do not include the whole
repository, all private documents, all schemas, or unrelated ADRs when a
smaller payload is enough.

## Payload Selection Rules

Include:

- the target EARS/Gherkin scenario.
- relevant ADRs.
- relevant architecture document sections.
- touched code or test files.
- minimal examples needed to preserve local style.

Exclude unless explicitly required:

- unrelated specifications.
- unrelated source files.
- full private documents or records when a fragment is enough.
- full data exports.
- secrets and API keys.
- provider responses not needed by the task.

## Model Routing Guidance

Use smaller or faster models, code completion, or deterministic tooling for:

- value object naming and field review.
- DTO shape drafting.
- import-boundary checks.
- formatting, linting, and mechanical edits.
- narrow test generation from clear Gherkin.

Use stronger reasoning models or an agentic assistant for:

- architecture boundary decisions.
- cross-cutting refactors.
- ambiguous domain modeling.
- persistence and migration design.
- privacy-sensitive provider routing.

Use deterministic tools instead of AI when possible for:

- dependency graph checks.
- formatting.
- linting.
- test execution.
- infrastructure config validation.
- migration ordering checks.

## Consequences

Positive:

- Reduces hallucination risk and accidental overreach.
- Keeps AI prompts smaller and more reviewable.
- Lets simple VO and DTO work be handled cheaply.
- Makes design intent visible before Phase 1 Red.

Negative:

- Adds a short design step before coding.
- Requires agents to explicitly choose context instead of forwarding everything.

## Enforcement

Code review should reject:

- implementation started without a design step.
- AI prompts or task notes that include unrelated large context.
- VO, DTO, port, or adapter choices introduced during implementation without
  being identified in the design step.
- tasks routed to external AI providers when local-only or privacy settings
  forbid it.
