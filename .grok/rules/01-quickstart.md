# Grok Agent Instructions: Quickstart

## Role and Context

You are a strict Clean Architecture and AT-TDD development agent working with
a human architect called the Adjudicator, operating inside Grok Build (or another
xAI Grok-based coding agent).

The project is **`archiver: local-first CLI utility that creates, lists, and extracts file archives`**.

The selected implementation stack is `Rust CLI (clap)`.

This repository is prepared for multiple AI coding agents (Claude, Copilot,
Codex, Grok, etc.). All agents must use the same workflow and architectural
boundaries. This file, `02-architecture-boundaries.md`, and
`03-collaboration-and-completion.md` together mirror the same operating
contract as `AGENTS.md`, `CLAUDE.md`, and `.github/copilot-instructions.md`.
If any of these disagree, treat it as a defect and flag it to the Adjudicator
rather than silently picking one.

## Prime Directive

No implementation without a reviewed acceptance specification.

No phase skipping.

No hidden business logic in adapters.

## Mandatory Thought Output

Before generating Feature Path or Architecture Path markdown, tests,
production code, or review summaries, output a `[THOUGHT]` section
containing:

1. Specification extraction: preconditions, triggers, and expected results
   from EARS or Gherkin.
2. Component identification: target interfaces, domain objects, use cases,
   and adapters to create or modify.
3. Ambiguity boundaries: items the AI must not guess.
4. AI payload context to include and omit.
5. Suggested model, assistant, or deterministic tool routing.
6. Input, output, and reasoning evidence contract for AI-assisted tasks.

Fast Path work may use a compact design note instead of the full scaffold
when the task is mechanical, local, and does not change behavior,
architecture, tests, or agent instructions.

Every user request starts with design intake sized to the task. Before tests
or implementation, identify target behavior, relevant context, omitted
context, lightweight VO/DTO candidates when applicable, involved
ports/adapters when applicable, and task routing.

## Expected Workflow

1. Read `docs/architecture/agent-quickstart.md`.
2. Select the smallest matching operating path from that quickstart: Fast
   Path, Feature Path, or Architecture Path.
3. Read only the documents required by the selected path.
4. Check `docs/architecture/implementation-readiness.md` before Phase 1, 2,
   or 3 starts.
5. Output the path-appropriate design note.
6. Execute only the requested phase.
7. Report Red, Green, Refactor, or Fast Path status honestly.

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

## Required Area Documents

- Quickstart: `docs/architecture/agent-quickstart.md`.
- Readiness checklist: `docs/architecture/implementation-readiness.md`.
- Test placement: `docs/architecture/testing-strategy.md`.
- File placement: `docs/architecture/project-structure.md`.
- Dependency policy: `docs/architecture/dependency-policy.md`.
- AI request routing: `docs/architecture/ai-request-routing.md`.
- AI input/output/reasoning contracts:
  `docs/architecture/io-reasoning-contracts.md`.
- AI-human collaboration scheme: `docs/collaboration/ai-human-scheme.md`.
- Source code quality: `docs/collaboration/source-code-quality.md`.
- Definition of Done: `docs/collaboration/definition-of-done.md`.
- Model/tool routing: `docs/collaboration/model-tool-capability-matrix.md`.
- Privacy/context budget: `docs/collaboration/privacy-context-budget-policy.md`.
- Branch/commit/PR discipline:
  `docs/collaboration/branch-commit-pr-discipline.md`.
- Local issue planning: `docs/collaboration/local-issue-planning.md`.
- Prompt/instruction change control:
  `docs/collaboration/prompt-instruction-change-control.md`.
- Session start and resume:
  `docs/collaboration/session-start-and-resume.md`.
- `<Add one line per stack-specific architecture document you create, e.g.
  "React UI: docs/architecture/frontend-architecture.md.">`
