# Evaluation and Golden Examples

This document defines how to evaluate AI-TDD collaboration artifacts without
entering application-internal design.

## Purpose

Golden examples keep AI outputs from drifting while still sounding plausible.
They provide stable examples for design intake, Phase 1 tests, reviewer
summaries, handoffs, and instruction changes.

## Locations

Use:

```text
docs/evaluation/golden-examples/
docs/evaluation/criteria/
```

Keep `.gitkeep` files in both folders until concrete examples are added.

## What Needs Golden Examples

Create golden examples for:

- design intake notes.
- Adjudicator review requests.
- agent handoff notes.
- Phase 1 Red test requests.
- Phase 3 reviewer empathy summaries.
- prompt or instruction change proposals.

Do not create golden examples that depend on private user data, real secrets,
or full private document exports.

## Evaluation Criteria

AI collaboration artifacts should be evaluated for:

- phase correctness.
- visible assumptions.
- small and relevant payload selection.
- omitted context explanation.
- Adjudicator decision points.
- deterministic verification summary.
- low cognitive load.
- absence of secrets and unrelated large context.

## Comparison Rules

Prefer deterministic comparison where possible:

- required headings present.
- required checklist items present.
- forbidden terms or paths absent.
- changed files listed.
- phase stated.

Use human review for:

- clarity.
- sufficiency of assumptions.
- quality of reviewer empathy summary.
- whether the artifact helps or burdens the Adjudicator.
