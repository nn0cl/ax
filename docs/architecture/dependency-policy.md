# Dependency Policy

Dependency relationships must be checked by tools as the implementation grows.

This document covers software packages, libraries, provider SDKs, CLI
tools, and test helpers. For AI-generated artifacts or human-sourced
external content/data resources (not software dependencies), see the
optional `docs/architecture/external-resource-adoption-contract.md`
instead.

Pick and configure the tools that match your stack. See
`docs/templates/examples/` for two worked examples (`cargo-deny` for Rust,
`dependency-cruiser` for a TypeScript front-end) — copy the pattern for
whatever language and package ecosystem this project actually uses.

## Package-Level Dependency Checks

Use a package-dependency auditing tool (e.g. `cargo-deny`, `npm audit` /
`pip-audit` / `osv-scanner`) once the corresponding project manifest exists.

Initial responsibilities:

- detect vulnerable dependencies.
- enforce license policy.
- detect duplicate or banned packages when configured.
- make dependency decisions visible in CI.

These tools do not replace Clean Architecture review. They check package-level
dependency policy, not whether a module imported the wrong local module.

## Dependency Adoption Checklist

Before adopting a new runtime library, framework package, provider SDK, CLI, or
test helper, record a lightweight adoption note in the design note, ADR, local
issue, or work trace.

The note should cover:

- **Security posture**: check whether vulnerability advisories exist for the
  library and the version under consideration. Prefer package-manager audit
  tools, official advisory databases, and maintained release notes.
- **Version-specific examples**: confirm that implementation examples,
  documentation, and community usage match the version intended for adoption.
  Avoid relying on examples for incompatible major versions.
- **Troubleshooting depth**: confirm that errors, migration notes, known
  issues, and common failure modes are searchable enough for the team to debug
  without broad model speculation.
- **Minimal real-file test**: verify that the smallest useful behavior can be
  exercised against actual project files or representative fixtures, not only
  mocked prose or README snippets.
- **POC feasibility**: when the dependency affects architecture, external I/O,
  persistence, build tooling, or generated artifacts, prefer a small
  throwaway POC before committing to the dependency.
- **Boundary fit**: confirm whether the dependency belongs in an adapter,
  delivery layer, build tooling, or test-only support. It must not leak into
  domain code.

If any item is unknown, list it as an ambiguity and stop for Adjudicator decision
or an ADR when the decision affects architecture.

## Import-Boundary Checks

Use an import-boundary tool (e.g. `dependency-cruiser` for TypeScript,
`import-linter` for Python) once the corresponding front-end or backend
project exists.

Initial responsibilities:

- prevent UI components from importing backend transport/provider SDKs
  directly.
- keep feature, entity, and shared boundaries visible.
- detect circular imports.

## CI Rule

Dependency policy checks should be conditional until the relevant
implementation exists — gate each job on the presence of the corresponding
manifest or config file (see the commented-out example jobs in
`.github/workflows/ci.yml`).

## Not a Substitute

These tools do not decide architecture. ADRs and AT-TDD specifications remain
the source of design decisions.
