# Project Start and Development Guide

Use this guide after the collaboration template has been copied into a target
repository. It explains how to start a project and continue development without
letting the template decide target-specific architecture or domain behavior in
advance.

For the rationale behind adopting this template, see
`docs/collaboration/template-benefits.md`.

## Core Distinction

The template distribution must not predefine the target project's domain model.

After adoption, the target project may and should design its domain model
inside this process, based on:

- accepted EARS/Gherkin specifications.
- Adjudicator decisions.
- ADRs when the decision affects architecture or long-term policy.
- Phase 1 tests reviewed before Phase 2 implementation.

In other words, the template must not ship a domain model. The adopted project
uses the template workflow to discover and implement its own domain model.

## Starting a New Project

1. Copy the template into the target repository.
2. Fill project identity placeholders in agent instruction files.
3. Write a short project boundary note in `docs/architecture/README.md`.
4. List current non-decisions in `CLAUDE.md` or the relevant architecture
   document.
5. Create the first target specification under `docs/specs/`.
6. Identify external resources that must become ports.
7. List early dependency candidates and apply the adoption checklist in
   `docs/architecture/dependency-policy.md`.
8. Start with Feature Path Phase 0 design intake.
9. Ask the Adjudicator to approve Phase 1 Red before writing tests.

Do not create a full domain layer, persistence schema, provider adapter, or UI
structure before a specification, ADR, or reviewed test requires it.

## Continuing an Existing Project

1. Run the copy script without `--force`.
2. Keep existing target architecture and accepted specifications authoritative.
3. Add missing collaboration files around the existing project shape.
4. Create or update only the architecture documents needed for touched areas.
5. Record conflicts between existing process and this template as process gaps
   or ADR questions.
6. Use Fast Path for mechanical adoption cleanup.
7. Use Feature Path only for work backed by accepted target specifications.
8. Use Architecture Path for boundary, prompt, privacy, or process decisions.
9. Re-check dependency adoption notes before introducing new libraries or
   upgrading major versions.

## Domain Modeling Flow

Domain modeling belongs in the target project, not in the reusable template.

Recommended flow:

1. Write the behavior in EARS or Gherkin.
2. During design intake, name candidate entities, value objects, use cases, and
   ports only when the specification makes them clear.
3. In Phase 1, express the expected behavior as tests.
4. In Phase 2, implement only the domain concepts required by reviewed tests.
5. In Phase 3, refine names and boundaries while preserving behavior.
6. Capture durable architecture decisions in ADRs.

Avoid guessing domain concepts from generic examples. Placeholder names and
example architecture documents are teaching aids, not target-project decisions.

## Development Loop

For each feature:

1. Confirm the issue or explicit no-issue reason.
2. Read `docs/architecture/agent-quickstart.md`.
3. Select Fast Path, Feature Path, or Architecture Path.
4. For Feature Path, read the target spec and relevant architecture document.
5. Produce the path-appropriate design note.
6. For new dependencies, check vulnerability reports, version-matched examples,
   troubleshooting depth, minimal real-file tests, and POC feasibility.
7. Execute only the Adjudicator-approved phase.
8. Run deterministic verification when available.
9. Record trace and cost/reasoning control signals when required.
10. Stop at phase gates for Adjudicator review.

## Stop and Ask

Stop for Adjudicator decision when:

- the target behavior is not specified.
- the requested phase is unclear.
- a domain concept is plausible but not supported by the spec.
- persistence, provider, model, or external layout choices are needed.
- dependency security, version suitability, troubleshooting evidence, or POC
  feasibility cannot be confirmed.
- existing target architecture conflicts with this template.
- the agent would need broad private context.

## Good Outcome

A healthy project started from this template should have:

- target-owned specifications under `docs/specs/`.
- project-specific architecture documents added only when needed.
- a domain model that emerges from accepted behavior.
- ports for external resources before adapters.
- traces that show when strong reasoning was actually necessary.
