# Definition of Done for AI-TDD Collaboration

This document defines completion criteria for collaboration phases. It does not
define application internals.

## Universal Done

A task is not done unless:

- the current phase is explicit.
- local issue, GitHub issue, work plan, or explicit no-issue reason is stated
  when the task is more than a tiny documentation edit.
- planned feature or bug work states its planning size, or explains why size is
  not applicable.
- size `M`, `L`, or `XL` work has a linked AI planning record.
- second-attempt bug fixes have a linked trace and updated planning size.
- changed files are listed in the final response or handoff.
- assumptions and open decisions are visible.
- deterministic verification was run or explicitly marked not applicable.
- issue status, phase, applicable work-plan row, and completion evidence are synchronized in the same reviewable unit when issue work changes status.
- no unrelated context, secrets, or full private data exports were used
  without Adjudicator approval.
- generated code, if any, is readable and appropriately split.

## Phase 0 Done: Design Intake

Done when:

- target behavior or question is stated.
- issue dependencies are identified or marked not applicable.
- included context is listed.
- omitted context is listed.
- AI payload and model/tool routing are stated.
- input/output/reasoning contract is stated when AI output is involved.
- Adjudicator decisions needed are identified.
- next phase is proposed, not silently executed.

## Phase 1 Done: Red

Done when:

- only tests or accepted test-only scaffolding were changed.
- tests map to accepted EARS/Gherkin behavior.
- external dependencies are mocked through ports or interfaces.
- expected Red state is reported.
- Phase 2 is not started before Adjudicator review.

## Phase 2 Done: Green

Done when:

- reviewed tests were not modified to pass.
- implementation is the smallest readable code needed to pass.
- business logic remains out of adapters, UI, commands, provider clients, and
  persistence structs.
- deterministic verification is run where available.
- any unreadable minimal code is either refactored immediately or marked as a
  Phase 3 readability risk.

## Phase 3 Done: Refactor

Done when:

- behavior and assertions are unchanged.
- separation of concerns is improved or preserved.
- readability is improved or preserved.
- deterministic verification is run where available.
- reviewer empathy summary is included.

## Documentation-Only Done

Done when:

- the document scope is explicit.
- internal application design is not introduced accidentally.
- new ADRs are listed in README and CI checks when accepted.
- instruction files are updated when agent behavior changes.
- YAML or other structured files are validated when touched.

## Issue Status Synchronization

Issue status drift is a process failure, not an optional documentation task. When an issue reaches
`done`, `review`, `blocked`, or `wont_do`, update all of the following before reporting completion:

1. `docs/issues/LISS-*.md` metadata and completion/current-status evidence.
2. The corresponding row and `Current Next Issue` in the active work plan, when a work plan exists.
3. Any accepted-spec or ADR references whose decision boundary changed.

The implementation commit and the status/documentation update should be the same reviewable unit,
or the handoff must explicitly identify the pending synchronization. A status is not considered
complete from code and tests alone.

## Handoff Done

Done when:

- current phase is stated.
- completed artifacts are listed.
- changed files are listed.
- verification status is stated.
- blockers are stated.
- next safe action is stated.
