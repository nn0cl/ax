# AI Request Routing and Payload Policy

AI assistance is allowed, but context must be intentionally selected.

## Design First

After any user request, agents must begin with a design step before writing
tests, implementation, migrations, or UI. The design step should be as small as
the selected operating path allows.

The design step must produce:

- target behavior or question.
- relevant EARS/Gherkin scenarios.
- value objects, DTOs, and ports that are clear enough to name now.
- implementation phase to run next.
- files and documents required as AI payload context.
- files and documents intentionally omitted.
- model or assistant class for each subtask.
- ambiguity boundaries.

This design step does not replace AT-TDD. It prepares the payload and task
routing before Phase 1 Red, Phase 2 Green, or Phase 3 Refactor.

## Payload Budgeting

Use the smallest useful payload.

Preferred payload order:

1. Current user request.
2. Target EARS/Gherkin scenario.
3. Relevant ADRs.
4. Relevant architecture document.
5. Touched files.
6. Minimal neighboring examples.

Avoid payloads that include:

- unrelated source directories.
- full private documents or records when only a fragment is needed.
- full data exports.
- secrets, API keys, or `.env` values.
- provider SDK documentation unless the adapter is being implemented.
- generated files or build artifacts.

## Reasoning Budgeting

Use the smallest useful reasoning budget.

### Mechanical

Use when the task is deterministic and local.

- inspect only directly touched files.
- prefer shell checks, formatters, linters, and tests over model judgment.
- do not compare alternatives unless a command fails or the user asks.

### Local Change

Use when behavior changes within one area and the target spec is clear.

- inspect the target spec, touched files, and minimal neighboring examples.
- compare at most two implementation options unless a third is forced by an
  existing architecture rule.
- stop when the requested phase is complete.

### Feature Phase

Use for AT-TDD Phase 1, 2, or 3 work.

- inspect the target spec, phase rules, relevant architecture document, and
  touched files.
- keep reasoning tied to the current phase gate.
- do not pre-design later phases beyond naming known risks.

### Architecture Decision

Use for ADRs, instruction changes, boundary conflicts, privacy-sensitive
routing, or provider choices.

- inspect relevant ADRs, contract files, and architecture rules.
- state rejected alternatives only when they materially affect the decision.
- stop for Adjudicator approval when the decision changes project policy.

## Value Object Design

Lightweight value objects should be clarified in the design step when the
requirements are already explicit.

Examples (replace with your project's actual domain concepts):

- `<DomainEntityId>`
- `<SourceReference>`
- `<ReviewStatus>`
- `<ConfidenceScore>`

Design may name a VO and its invariants, but implementation still follows the
requested AT-TDD phase.

Do not invent persistence columns, provider schemas, vector dimensions, or
external layout conventions just because a VO was named.

## Model Assignment

Route tasks by risk and complexity:

- Simple VO/DTO naming: lightweight model or code assistant.
- Clear Gherkin-to-test conversion: code assistant or smaller coding model.
- Architecture boundary choices: stronger reasoning model.
- Cross-module refactor: stronger reasoning model plus deterministic tests.
- Formatting/lint/import checks: deterministic tools.
- Privacy-sensitive provider choice: stronger reasoning model and Adjudicator
  review.

Escalate to a stronger reasoning agent only when ambiguity, cross-boundary
change, privacy risk, or instruction conflict is present. Downgrade to a
deterministic tool or code assistant when the task is mechanical, local, and
covered by an accepted spec.

## Required Design Note Format

```markdown
## Design Note

- Target behavior:
- Phase to execute next:
- Context to include:
- Context to omit:
- VO/DTO candidates:
- Ports/adapters involved:
- Suggested task routing:
- Ambiguities:
```

If the Adjudicator already provided enough detail, the design note should be short
and decisive. If the request is ambiguous, list the missing decisions and stop
before implementation.
