# Why Use This Template?

This document explains what this collaboration template is good for. It is not
a promise of automatic productivity or cost reduction. The benefits come from
using the process consistently.

## Primary Benefit

The template turns AI-assisted development from an ad hoc chat workflow into a
reviewable engineering workflow.

It gives humans and AI agents shared rules for:

- what must be specified before implementation.
- which phase is allowed now.
- what context may be sent to AI.
- when strong reasoning is justified.
- when deterministic tools should replace model judgment.
- where architecture decisions and handoff evidence are recorded.

## What Problems It Reduces

### Speculation Before Specification

AI agents often fill missing requirements with plausible implementation. This
template forces target behavior into EARS/Gherkin specs and design intake
before production code.

Expected result:

- fewer guessed features.
- less rework from misunderstood behavior.
- clearer Adjudicator review points.

### Phase Skipping

AI agents can jump from vague intent directly to implementation. This template
separates Phase 1 Red, Phase 2 Green, and Phase 3 Refactor.

Expected result:

- tests are reviewed before implementation.
- minimal implementation stays tied to accepted behavior.
- refactoring has a clear point in the workflow.

### Hidden Architecture Drift

Adapters, UI handlers, persistence structs, and provider clients can quietly
accumulate business policy. This template repeats Clean Architecture boundaries
and requires external resources to be represented as ports.

Expected result:

- domain and use-case logic remain easier to inspect.
- provider and datastore choices are easier to replace.
- architecture decisions become ADRs instead of hidden assumptions.

### Excessive LLM Reasoning

High-capability coding models are useful, but expensive reasoning is wasteful
for mechanical work. This template separates Fast Path, Feature Path, and
Architecture Path.

Expected result:

- local mechanical work can use compact design notes.
- deterministic tools handle facts, formatting, parsing, and verification.
- stronger reasoning is reserved for ambiguity, boundary decisions, privacy,
  and cross-module design.

### Unbounded Context Sharing

Large prompts can increase cost, leak private data, and confuse the task. This
template defines context budgets and traceable included/omitted context.

Expected result:

- smaller AI payloads.
- clearer privacy posture.
- easier continuation by another agent.

### Unsafe Dependency Adoption

Libraries can look attractive in examples but fail on version mismatch,
security advisories, poor troubleshooting evidence, or inability to run a
small real-file test. This template adds a dependency adoption checklist.

Expected result:

- fewer fragile dependency choices.
- clearer POC expectations.
- dependency decisions that are easier to review.

## Who Benefits

### Adjudicator

The Adjudicator gets explicit review gates, ADR hooks, handoff records, and a clear
place to reject assumptions.

### AI Coding Agents

Agents get enough structure to avoid guessing, over-reading, over-building, or
using strong reasoning when a deterministic tool would do.

### Future Maintainers

Maintainers get specs, traces, ADRs, and readable phase history instead of only
final code and chat fragments.

### Existing Projects

Existing projects can adopt the collaboration layer without replacing product
README files, accepted architecture, or target-owned specifications.

## What It Does Not Do

This template does not:

- choose the target domain model.
- choose the target stack, datastore, provider, or LLM.
- guarantee lower cost without disciplined use.
- replace human architectural judgment.
- remove the need for tests, CI, dependency checks, or code review.

## How to Tell It Is Working

Look for these signs:

- features start from target-owned specs.
- agents stop at phase gates.
- Fast Path is used for mechanical work.
- Architecture Path is used only when decisions really affect boundaries or
  policy.
- traces explain what context was included and omitted.
- dependency adoption notes include security, version, troubleshooting, test,
  and POC evidence.
- review comments focus on real design tradeoffs instead of missing process.

## Warning Signs

Investigate if:

- agents repeatedly implement without accepted specs.
- Phase 1 tests are changed during Phase 2 just to pass.
- every task becomes Architecture Path.
- traces are long but do not explain decisions.
- AI prompts include broad unrelated source or private context.
- dependency choices lack version-specific evidence or minimal real-file tests.
