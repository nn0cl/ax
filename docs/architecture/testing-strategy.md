# Testing Strategy

Testing follows AT-TDD phase gates.

## Test Levels

### Acceptance Tests

Purpose:

- prove Gherkin scenarios.
- drive Phase 1 Red.

Placement:

- CLI acceptance / integration: `tests/archive_cli.rs`, `tests/pack_restore.rs`
  (assert_cmd + tempfile).
- Domain unit tests: `src/core/domain/*` via `#[cfg(test)]` modules.
- No UI layer in this project.

### Product-path coverage (archiver)

| Area | Where | Notes |
|------|--------|------|
| Pack / restore / verify | `tests/pack_restore.rs` | Primary path |
| Resume strict (no plan) | `pack_resume_without_plan_fails` | ADR 0015 |
| Config `.inf` | `pack_from_config_inf` | ADR 0017 |
| `verify_stage` | `pack_with_verify_stage_succeeds` | ADR 0016 |
| Content × algo estimate | domain `content_class` + dry-run `size_estimate` | ADR 0018 |
| Concurrent same output | best-effort assertion (unsupported) | ADR 0019 |
| Huge trees / flaky USB / Windows | **not** automated | manual `docs/QA.md` |

Manual pre-release: [`docs/QA.md`](../QA.md).

### Domain Unit Tests

Purpose:

- prove pure domain behavior.

Rules:

- no mocks needed for pure logic.
- no framework, DB, network, file-system, or SDK imports.

### Application Use Case Tests

Purpose:

- prove orchestration through ports.

Rules:

- use fake or mock port implementations.
- assert outputs and port interactions specified by Gherkin.
- no real adapters.

### Adapter Integration Tests

Purpose:

- prove concrete provider integration.

Rules:

- must be explicitly requested or covered by an ADR.
- must be separable from normal unit tests.
- must not be required for Phase 1 Red of core behavior.

### Dependency Policy Checks

Purpose:

- catch package dependency, license, advisory, and import-boundary drift.

Rules:

- run the project's chosen dependency-policy tool(s) once configured (see
  `docs/architecture/dependency-policy.md`).
- do not treat these checks as substitutes for Clean Architecture review.

### Front-End Tests

Purpose:

- prove UI behavior and user interaction.

Rules:

- Not applicable: this project has no GUI. CLI behavior is covered by
  `tests/archive_cli.rs`.

### E2E Tests

Purpose:

- prove the assembled CLI flow (create → list → extract).

Rules:

- Covered by Cargo integration tests with real temp directories (no network).
- Do not depend on real external providers.

## Phase Mapping

Phase 1 Red:

- add failing tests only.
- prefer use-case tests for core behavior.
- prefer UI tests for presentation behavior.

Phase 2 Green:

- add minimum implementation.
- do not edit reviewed tests to pass.

Phase 3 Refactor:

- improve structure.
- keep behavior and assertions stable.

## Mocking Rule

Mock ports, not concrete providers.

Examples:

- mock `ArchiveEngine`, not the `zip` / `tar` crate clients.
