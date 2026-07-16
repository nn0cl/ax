# AI-TDD + Human Collaboration Scheme

This document defines how humans and AI agents collaborate in this repository.
It does not define application internals.

## Roles

### Adjudicator

The human architect and final decision maker.

Responsibilities:

- approve or reject phase transitions.
- accept or reject ADRs.
- review generated tests before implementation.
- decide ambiguous architecture or product questions.
- confirm when AI may use broader context, external providers, or stronger
  models.

### Agent

An AI coding or design assistant.

Responsibilities:

- start from design intake.
- expose assumptions and ambiguity boundaries.
- execute only the requested phase.
- keep payloads minimal.
- produce reviewable artifacts.
- stop when a Adjudicator decision is required.

### Deterministic Tool

Non-AI tool such as formatter, linter, dependency checker, test runner,
container orchestration validator, migration checker, or import-boundary
checker.

Responsibilities:

- verify facts that should not depend on model judgment.
- provide repeatable signals for CI and review.

## Collaboration Loop

```text
User request
  -> Phase 0 Design Intake
  -> Adjudicator review or approval
  -> Phase 1 Red
  -> Adjudicator reviews tests
  -> Phase 2 Green
  -> deterministic verification
  -> Phase 3 Refactor
  -> reviewer empathy summary
  -> Adjudicator final review
```

The loop can stop at any point when the Adjudicator asks for clarification, changes
scope, rejects an assumption, or requests a new ADR.

## Required Artifacts

Every task should leave enough evidence for another human or agent to continue.

Required for design-only work:

- design note.
- local issue or work plan reference when planning feature work.
- included and omitted context.
- open decisions.
- proposed next phase.

Required for Phase 1:

- failing tests only.
- explanation of expected Red state.
- mocked ports or interfaces for external dependencies.

Required for Phase 2:

- minimal implementation.
- verification output summary.
- unchanged reviewed tests.

Required for Phase 3:

- refactor summary.
- verification output summary.
- reviewer empathy summary.

## Decision Gates

Agents must stop for Adjudicator decision when:

- phase is not explicitly selected.
- issue dependencies are unclear or unresolved.
- requirements imply a new architecture decision.
- a payload would need unrelated large context.
- a task requires secrets, full source documents, or full private data
  exports.
- an external provider, SDK, model, DB product, or schema convention must be
  chosen.
- a change would alter accepted tests.
- deterministic verification contradicts AI assumptions.

## Context Ledger

Each substantial task should maintain a short context ledger in the design note
or final answer:

- `Included`: files, specs, ADRs, and snippets used.
- `Omitted`: relevant-looking context intentionally excluded.
- `Assumptions`: assumptions made for this phase.
- `Open decisions`: questions for Adjudicator or future ADR.
- `Verification`: deterministic checks run or not run.
- `Issue links`: local issue IDs, GitHub issue links, and work plan links.

## Handoff Rule

When stopping before completion, the agent must state:

- current phase.
- completed artifacts.
- next safe action.
- blockers.
- files changed.
- verification status.

This keeps work resumable by another agent without rereading the entire
repository.

## Quality Bar

Acceptable AI work is:

- phase-correct.
- reviewable in small pieces.
- readable with low human cognitive load.
- traceable to specs, ADRs, or Adjudicator instructions.
- verified by deterministic tools when possible.
- honest about ambiguity and unverified claims.

Unacceptable AI work is:

- implementation before design intake.
- implementation before reviewed tests when Phase 1 is required.
- broad context dumping.
- hidden assumptions.
- modifying tests to make implementation pass.
- turning AI prose into accepted design without Adjudicator or ADR review.
- generating dense or multi-responsibility source code that is difficult for a
  human to review.
