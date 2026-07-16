# Implementation Readiness Checklist

Use this checklist before starting a coding task.

## Required Inputs

- A target EARS or Gherkin specification exists under `docs/specs/` for
  Feature Path work.
- A design note identifies target behavior, included context, omitted context,
  task routing, and any applicable VO/DTO candidates or ports/adapters.
- AI-assisted tasks identify input envelope, output schema, and reasoning
  evidence contract.
- The Adjudicator explicitly selected the current phase.
- The touched area has a matching architecture rule document.
- Unknown provider, DB, model, or folder decisions are listed as ambiguities or
  captured in an ADR.
- New dependencies have a lightweight adoption note covering vulnerability
  posture, version-specific examples, troubleshooting depth, minimal real-file
  testing, POC feasibility, and Clean Architecture boundary fit.
- Settings tasks separate normal settings, validation, and secrets.
- Optional local infrastructure tasks (e.g. containerized services) keep that
  infrastructure outside domain and use-case unit test requirements.

## Ready for Phase 1 Red

- The scenario has clear `Given`, `When`, and `Then` clauses.
- External resources can be represented as ports.
- Expected results are observable.
- The test location is selected from `docs/architecture/testing-strategy.md`.

## Ready for Phase 2 Green

- Phase 1 tests were reviewed.
- The implementation location follows `docs/architecture/project-structure.md`.
- Business logic belongs in domain or application modules.
- Delivery handlers, UI components, and adapters remain thin.
- Source code remains readable, appropriately split, and reviewable by a human
  Adjudicator.

## Ready for Phase 3 Refactor

- Tests are green.
- Refactoring does not change assertions or behavior.
- Remaining risks can be summarized for the Adjudicator.

## Not Ready If

- The task starts tests, implementation, migrations, or UI without the
  path-appropriate design intake.
- The proposed AI request payload includes unrelated files, full private
  documents, secrets, or provider data not required by the task.
- AI output is accepted as trusted data without structured validation, source
  evidence, confidence or uncertainty, and review status.
- The task requires choosing a datastore, vector DB, embedding model, external
  layout, or provider API without an ADR or explicit Adjudicator instruction.
- A new dependency is adopted without checking known vulnerability reports for
  the intended version, version-matched examples, troubleshooting evidence, a
  minimal real-file test path, and POC feasibility when architecture risk is
  present.
- Saving settings triggers side-effecting external calls (writes, provider
  calls, projections) that the feature does not require.
- Domain or use-case unit tests require optional infrastructure (containers,
  external services) to be installed.
- The proposed code puts business policy in UI components, delivery handlers,
  or adapters.
- Tests require a real external service, network call, or provider for core
  behavior.
- The proposed code is dense, multi-responsibility, or split into speculative
  abstractions that increase human cognitive load.
