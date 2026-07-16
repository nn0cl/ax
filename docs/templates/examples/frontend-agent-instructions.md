# Front-End Agent Instructions (example)

Place content like this in `<frontend-dir>/AGENTS.md` when the front-end
project is created.

## Scope

This directory owns UI rendering and presentation state.

## Required Reading

- `../docs/architecture/frontend-architecture.md`
- `../docs/architecture/testing-strategy.md`

## Rules

- Do not call backend transport APIs directly from components; use a shared
  client boundary.
- Do not implement confidence, trust, merge, or validation policy in the UI
  layer.
- Mock shared clients in tests.
- Keep UI text and layout consistent with the target workflow.
