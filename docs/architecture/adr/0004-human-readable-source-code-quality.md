# ADR 0004: Human-Readable Source Code Quality

## Status

Accepted

## Context

The repository is designed for AI-TDD plus human Adjudicator review. AI agents can
generate large amounts of code quickly, but large files, mixed responsibilities,
deep nesting, and clever abstractions increase human cognitive load.

Clean Architecture defines dependency direction, but the collaboration scheme
also needs a quality bar for readability and reviewability of generated source
code.

This ADR defines source-code quality expectations without deciding application
internals.

## Decision

Generated source code must optimize for human review and low cognitive load.

AI agents must prefer:

- small, purpose-named modules.
- small functions with one reason to change.
- explicit types and boundaries over clever implicit behavior.
- shallow control flow.
- domain and application code that can be read without provider or framework
  knowledge.
- tests that explain behavior through clear Given/When/Then structure.

AI agents must avoid:

- large multi-responsibility files.
- speculative abstractions.
- deeply nested control flow.
- hidden side effects.
- framework or provider details leaking into core code.
- generated code that is correct only because a model remembers unstated
  context.

## Reviewability Heuristics

Code should usually be split when:

- a file mixes domain, application, adapter, and delivery concerns.
- a function requires scrolling to understand its whole behavior.
- a reader must keep more than one business decision in mind at once.
- tests need unrelated setup to verify a single behavior.
- names explain implementation technology instead of domain or capability.

Code should usually stay together when:

- splitting would create indirection without reducing responsibility.
- the behavior is small and only used in one place.
- a test would become harder to read because setup is scattered.

## Phase Behavior

Phase 1:

- Tests should be small, behavior-focused, and named from the Gherkin intent.

Phase 2:

- Implementation should be minimal, but not compressed into unreadable code.
- Passing tests with dense, clever, or multi-responsibility code is not
  acceptable Green.

Phase 3:

- Refactor should reduce human cognitive load.
- The reviewer empathy summary should mention any remaining readability risk.

## Consequences

Positive:

- Human reviewers can validate AI work faster.
- Future agents can resume work without rediscovering hidden intent.
- Refactors have a clear target beyond merely passing tests.

Negative:

- Some code may use more files or names than the smallest possible
  implementation.
- Agents must balance YAGNI against readability instead of compressing code.

## Enforcement

Code review should reject:

- source files that mix architectural concerns.
- large functions with multiple business decisions.
- speculative abstractions that are not required by current tests.
- tests whose setup hides the behavior being asserted.
- code that increases Adjudicator cognitive load without a documented reason.
