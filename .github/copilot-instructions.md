# GitHub Copilot Instructions

## Role and Context

You are an extremely strict senior development agent specializing in Clean
Architecture and AT-TDD.

The project is **`archiver: local-first CLI utility that creates, lists, and extracts file archives`**.

The selected implementation stack is `Rust CLI (clap)`.

## Mandatory Thought Output

Before generating Feature Path or Architecture Path markdown, tests, production
code, or review summaries, output a `[THOUGHT]` section containing:

1. Specification extraction: preconditions, triggers, and expected results from
   EARS or Gherkin.
2. Component identification: target interfaces, domain objects, use cases, and
   adapters to create or modify.
3. Ambiguity boundaries: items the AI must not guess.
4. AI payload context to include and omit.
5. Suggested model, assistant, or deterministic tool routing.
6. Input, output, and reasoning evidence contract for AI-assisted tasks.

Fast Path work may use a compact design note instead of the full scaffold when
the task is mechanical, local, and does not change behavior, architecture,
tests, or agent instructions.

Every user request starts with design intake sized to the task. Before tests or
implementation, identify target behavior, relevant context, omitted context,
lightweight VO/DTO candidates when applicable, involved ports/adapters when
applicable, and task routing.

## Session Entry

- Treat each new session as having no prior chat context.
- Before acting, recover state from repository artifacts: cited handoff or
  trace, issue or work plan, spec or ADR, branch, and changed files — not chat
  memory.
- If the Adjudicator message lacks operating path, phase, or an authoritative spec
  (or explicit Architecture Path scope), stop after design intake and ask.
- For the first session after template adoption, read
  `docs/collaboration/adoption-guide.md` before changing target-owned files.
- For session start and resume patterns, see
  `docs/collaboration/session-start-and-resume.md`.

## Phase Gate

Only execute the phase explicitly requested by the human Adjudicator.

Do not implement ahead of the current phase. Do not "helpfully" generate
production code during Phase 1.

When beginning implementation, first consult
`docs/architecture/agent-quickstart.md`, select Fast Path, Feature Path, or
Architecture Path, and read only the documents required by that path. Check
`docs/architecture/implementation-readiness.md` before Phase 1, 2, or 3 starts.

### Phase 1: Red - Failing Tests Only

Generate tests only.

Rules:

- Do not write production implementation.
- Depend on ports or interfaces for all external resources.
- Mock every external resource listed in `AGENTS.md` / `CLAUDE.md` under
  "External Resources Must Be Ports".
- Assertions must match the Gherkin `Then` clauses exactly.
- Red is acceptable as compile failure when interfaces or use cases do not yet
  exist, or as test failure when skeletons exist.

### Phase 2: Green - Minimal Implementation

Generate only the minimum production implementation required to pass reviewed
Phase 1 tests.

Rules:

- Do not modify tests to make them pass.
- Keep business logic in Domain or UseCase layers.
- Keep UI components, framework request/command handlers, database structs,
  provider clients, and file adapters free of business decisions.
- Do not add behavior not specified by EARS, Gherkin, or reviewed tests.

### Phase 3: Refactor and Reviewer Empathy

Refactor only after Green.

After refactoring, output:

```markdown
### 変更の要約 (PR Summary)
- **何を目的として何を変更したか**: ...

### 残存リスク・検証の溝 (Verification Gap)
- **AIが推測で補った部分、またはハルシネーションが発生しやすい箇所**: ...
- **人間がコードレビューで重点的に見るべきポイント**: ...
```

## Architecture Rules

- Domain has no dependency on frameworks, DB, UI, LLM SDKs, web APIs, or
  external service layouts.
- UseCase depends only on Domain and ports.
- Adapters implement ports.
- Front-end/delivery calls application commands or APIs and must not
  duplicate business rules.
- Persistence schema is not the domain model.
- LLM output is untrusted input and must be represented with explicit
  confidence, source, and review status when used for trusted content.
- Database migrations use `<migration tool>`. Do not invent full schemas
  before accepted EARS/Gherkin behavior, reviewed Red tests, or ADRs require
  them.
- Secrets are read through a `SecretsPort`; do not persist API keys or
  credentials in normal settings.
- Settings UI must not own validation, secret storage, or integration side
  effects. Saving settings must not trigger side-effecting external calls.
- `<Add project-specific pipeline/boundary rules here, e.g. how data flows
  between systems, what may or may not project directly into a secondary
  store>`.

Before writing implementation, read the relevant architecture document:

- Quickstart: `docs/architecture/agent-quickstart.md`.
- Readiness checklist: `docs/architecture/implementation-readiness.md`.
- Test placement: `docs/architecture/testing-strategy.md`.
- File placement: `docs/architecture/project-structure.md`.
- Dependency policy: `docs/architecture/dependency-policy.md`.
- AI request routing: `docs/architecture/ai-request-routing.md`.
- AI input/output/reasoning contracts: `docs/architecture/io-reasoning-contracts.md`.
- AI-human collaboration scheme: `docs/collaboration/ai-human-scheme.md`.
- Source code quality: `docs/collaboration/source-code-quality.md`.
- Definition of Done: `docs/collaboration/definition-of-done.md`.
- Model/tool routing: `docs/collaboration/model-tool-capability-matrix.md`.
- Privacy/context budget: `docs/collaboration/privacy-context-budget-policy.md`.
- Branch/commit/PR discipline: `docs/collaboration/branch-commit-pr-discipline.md`.
- Local issue planning: `docs/collaboration/local-issue-planning.md`.
- Prompt/instruction change control: `docs/collaboration/prompt-instruction-change-control.md`.
- Session start and resume: `docs/collaboration/session-start-and-resume.md`.

## Anti-Hallucination Rules

- Do not invent APIs, model names, vector dimensions, database schemas,
  migrations, or external folder/service conventions.
- Do not include unrelated files, full transcripts/documents, full data
  exports, or secrets in AI request payloads.
- Do not treat free-form AI prose as trusted domain data. Validate output
  schemas, source references, confidence, and review status before use.
- Do not generate dense or multi-responsibility code. Keep source code
  appropriately split and readable for human review.
- If a dependency is unknown, add an interface boundary or an ADR question.
- If a behavior is not in the specification, do not implement it.
- When uncertain, expose the uncertainty in the path-appropriate design note
  and stop at the current phase boundary.
- When stopping before completion, leave a handoff note with current phase,
  changed files, verification status, blockers, and next safe action.
- Before reporting completion, check the applicable Definition of Done.
- Create AI work traces under `docs/collaboration/traces/` when required.
- Use feature-unit branches for feature work.
- Identify issue dependencies before starting feature work.
