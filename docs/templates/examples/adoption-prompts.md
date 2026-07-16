# Adoption Prompt Examples

These examples intentionally avoid project-specific paths, product names,
domain models, providers, databases, and implementation stacks. Replace the
angle-bracket placeholders with target-project information before use.

## Initial Assessment Prompt

Use this first after copying the template into a target repository.

```markdown
You are working in this repository:

<TARGET_REPOSITORY_PATH>

This repository has just adopted the AI-human collaboration template.

Before implementing anything:

1. Read `AGENTS.md`.
2. Read `docs/architecture/agent-quickstart.md`.
3. Read `docs/collaboration/adoption-guide.md`.
4. Read `docs/collaboration/project-start-guide.md`.
5. Read `docs/collaboration/template-benefits.md`.

Task:

- Determine whether this request should use Fast Path, Feature Path, or
  Architecture Path.
- Inspect the existing README, source layout, docs, and configuration only as
  needed.
- Identify template placeholders that should be filled for this target project.
- Identify target-owned architecture or specification documents that already
  exist and must remain authoritative.
- Do not implement, refactor, delete, or overwrite existing project behavior.
- Do not infer the target domain model, datastore, provider, external API, or
  LLM model without an accepted specification, ADR, or Adjudicator decision.

Output:

1. Selected operating path and why.
2. Files read.
3. Existing project facts discovered.
4. Placeholders to fill.
5. Missing specs or architecture documents to create later.
6. Adjudicator decisions needed.
7. Next safe action.
```

## Initial Template-Fill Prompt

Use this after the initial assessment has been reviewed by the Adjudicator.

```markdown
Adjudicator decision:

- Operating path: Architecture Path.
- Scope: initial template adoption cleanup only.
- Do not implement application behavior.
- Keep existing target README, accepted specs, and architecture documents
  authoritative.
- Fill only generic placeholders in:
  - `AGENTS.md`
  - `CLAUDE.md`
  - `.github/copilot-instructions.md`
  - `docs/architecture/README.md`
- Add or update collaboration trace if required by
  `docs/collaboration/prompt-instruction-change-control.md`.

Target project facts approved by the Adjudicator:

- Project name: `archiver`
- Domain summary: `<ONE_LINE_DOMAIN_SUMMARY>`
- Selected stack: `<STACK_SUMMARY_OR_CURRENT_NON_DECISION>`
- Known external resources: `<EXTERNAL_RESOURCE_LIST_OR_CURRENT_NON_DECISION>`
- Current non-decisions: `<NON_DECISION_LIST>`

Task:

Apply only the approved placeholder updates and report verification.
```

## First Feature Phase Prompt

Use this only after a target-owned specification exists.

```markdown
Target specification:

- `docs/specs/<FEATURE_SPEC>.md`

Current phase:

- Phase 1: Red

Task:

Write failing tests only for the accepted behavior in the target specification.
Do not write production implementation.
Do not add provider, datastore, UI, or framework behavior beyond the reviewed
specification.
Represent external resources as ports or interfaces.

Before editing:

- Read `docs/architecture/agent-quickstart.md`.
- Use Feature Path.
- Read the target specification.
- Read the relevant architecture document for the touched area.
- Check `docs/architecture/implementation-readiness.md`.
```

## Dependency Adoption Prompt

Use this before introducing a new dependency or upgrading a major version.

```markdown
Dependency candidate:

- Name: `<DEPENDENCY_NAME>`
- Intended version or range: `<VERSION>`
- Intended use: `<WHY_THIS_DEPENDENCY_IS_NEEDED>`
- Expected layer: `<adapter|delivery|build tooling|test-only|other>`

Task:

Evaluate whether this dependency is ready for adoption.

Check:

- known vulnerability advisories for the intended version.
- version-specific documentation and implementation examples.
- troubleshooting depth, known issues, and migration notes.
- whether a minimal real-file test or representative fixture can verify the
  intended behavior.
- whether a small POC is needed before adoption.
- whether the dependency respects Clean Architecture boundaries.

Do not add the dependency yet. Report findings, risks, and Adjudicator decisions
needed.
```
