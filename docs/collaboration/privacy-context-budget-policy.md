# Privacy and Context Budget Policy

This policy controls how much context can be sent to AI tools during AI-TDD
collaboration. It does not define application runtime privacy behavior.

## Budget Classes

### Public Project Context

Examples:

- README.
- ADRs.
- architecture docs.
- templates.
- CI configuration.

Default:

- allowed in AI payloads when relevant.

### Local Project Source

Examples:

- source files.
- tests.
- config files.

Default:

- include only touched files and minimal neighboring examples.

### Private User or Business Content

Examples:

- user-generated documents or transcripts.
- customer records.
- private notes or excerpts from an external knowledge store.

Default:

- do not include unless the task explicitly requires it.
- prefer short excerpts.
- record inclusion in the context ledger.

### Secrets and Credentials

Examples:

- `.env` values.
- API keys.
- tokens.
- database passwords.

Default:

- never include in AI payloads, traces, golden examples, or review summaries.

## Context Budgets

Use the smallest useful context.

Default maximums unless the Adjudicator approves more:

- one target EARS/Gherkin scenario.
- only directly relevant ADRs.
- only directly touched files.
- excerpts instead of full documents.
- summaries instead of provider raw responses.

## Reasoning Budgets

Use the smallest useful reasoning effort that can preserve correctness.
Record the selected operating path and any escalation reason in the work trace
when a trace is required.

### Fast Path Budget

Default for mechanical, local, and deterministic work.

- read only the touched files and one directly relevant rule document.
- use compact design notes.
- avoid speculative alternatives.
- verify with deterministic tools where available.

### Feature Path Budget

Default for AT-TDD Phase 1, 2, or 3 work.

- read the target specification, current phase rules, touched files, and the
  relevant architecture document.
- reason only about the current phase.
- leave future-phase ideas as risks or next actions, not implementation.

### Architecture Path Budget

Default for ADRs, process changes, prompt changes, privacy-sensitive routing,
or cross-boundary design.

- read the relevant contract files, ADRs, and policy documents.
- summarize only decision-relevant alternatives.
- stop for Adjudicator approval when policy changes.

## Required Ledger

When private or large context is considered, record:

- what was included.
- what was omitted.
- why the included context was necessary.
- whether Adjudicator approval was needed.

For substantial tasks, also record the cost/reasoning control signals defined
in `docs/collaboration/llm-cost-reduction.md`.

## Forbidden Payloads

Do not send:

- full exports of private or customer data.
- raw secrets.
- full `.env` files.
- unrelated source directories.
- provider logs containing credentials or personal data.

## Review Rule

If a task cannot be done without forbidden or large private context, stop and
ask the Adjudicator for a narrower excerpt or explicit approval.
