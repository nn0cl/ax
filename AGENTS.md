# Agent Operating Contract

This repository is prepared for multiple AI coding agents. All agents must use
the same workflow and architectural boundaries.

## Prime Directive

No implementation without a reviewed acceptance specification.

No phase skipping.

No hidden business logic in adapters.

## Expected Workflow

1. Read `docs/architecture/agent-quickstart.md`.
2. Select the smallest matching operating path from that quickstart:
   Fast Path, Feature Path, or Architecture Path.
3. Read only the documents required by the selected path.
4. Check `docs/architecture/implementation-readiness.md` before Phase 1, 2, or
   3 starts.
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

Relevant architecture documents:

- Quickstart: `docs/architecture/agent-quickstart.md`.
- File placement: `docs/architecture/project-structure.md`.
- Readiness checklist: `docs/architecture/implementation-readiness.md`.
- Test placement: `docs/architecture/testing-strategy.md`.
- Dependency policy: `docs/architecture/dependency-policy.md`.
- AI request routing: `docs/architecture/ai-request-routing.md`.
- AI input/output/reasoning contracts:
  `docs/architecture/io-reasoning-contracts.md`.
- AI-human collaboration scheme:
  `docs/collaboration/ai-human-scheme.md`.
- Source code quality for AI-TDD:
  `docs/collaboration/source-code-quality.md`.
- Definition of Done:
  `docs/collaboration/definition-of-done.md`.
- Model/tool routing:
  `docs/collaboration/model-tool-capability-matrix.md`.
- Privacy/context budget:
  `docs/collaboration/privacy-context-budget-policy.md`.
- Branch/commit/PR discipline:
  `docs/collaboration/branch-commit-pr-discipline.md`.
- Local issue planning:
  `docs/collaboration/local-issue-planning.md`.
- Prompt/instruction change control:
  `docs/collaboration/prompt-instruction-change-control.md`.
- Session start and resume:
  `docs/collaboration/session-start-and-resume.md`.
- Rust CLI / Clean Architecture: `docs/architecture/rust-cli-architecture.md`.

## Clean Architecture Dependency Rule

Allowed dependencies:

- Domain -> nothing project-specific.
- UseCase -> Domain and Ports.
- Adapter -> UseCase, Ports, framework SDKs, DB SDKs, file system, network.
- UI/Delivery -> application command/query contracts and presentation state.

Forbidden dependencies:

- Domain -> Adapter.
- Domain -> Framework.
- UseCase -> DB schema.
- UseCase -> migration files.
- UseCase -> UI component.
- UseCase -> framework request/command handler.
- UI -> DB.
- UI -> external provider SDK.
- Adapter -> business policy not present in UseCase or Domain.

## External Resources Must Be Ports

Represent these as ports before using concrete implementations:

- Local filesystem (walk, read, write, create directories).
- Archive format engines (ZIP, TAR, TAR.GZ create/list/extract).

## Adjudicator Interaction

When a decision affects architecture, capture it as an ADR. When a decision is
unknown, list it in the path-appropriate design note as an ambiguity boundary.

Every request starts from design intake. Select only the AI payload context
needed for the task, define lightweight VO or DTO candidates when clear, and
route subtasks to an appropriate model, code assistant, or deterministic tool.
When AI or model output is involved, define input, output, and reasoning
evidence contracts before implementation.

Use the full `[THOUGHT]` scaffold only for Feature Path and Architecture Path
work. For Fast Path work, use a compact design note that states scope, omitted
context, deterministic checks, and why the full scaffold is unnecessary.

When handing off or stopping before completion, use
`docs/templates/agent-handoff.md`. When asking the Adjudicator for approval, use the
review points from `docs/templates/adjudicator-review.md`.

Generated source code must minimize human cognitive load. Prefer clear
responsibility boundaries, small functions, straightforward names, and
reviewable tests. Do not compress implementation into dense code just to be
minimal.

Before reporting completion, check `docs/collaboration/definition-of-done.md`.
Create AI work traces under `docs/collaboration/traces/` when the trace policy
requires it. Use feature-unit branches for feature work.
For feature work, identify local issue or GitHub issue dependencies before
creating the branch.
