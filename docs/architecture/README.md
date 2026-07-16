# Architecture Overview

The project uses Clean Architecture with **local-first** runtime assumptions.

The selected stack is **Rust CLI (clap + Cargo)**. The application core lives
in `src/core/` (domain, use cases, ports). Delivery is the CLI in
`src/delivery/`. Format engines and filesystem access live in `src/adapters/`.

## Layers

### Domain

Pure archive concepts: format identity, entry metadata, path safety rules for
archive member names, and domain errors.

Must not depend on:

- clap or any CLI framework.
- `std::fs` path operations that perform I/O (path *policy* is allowed as pure
  functions over string paths).
- ZIP/TAR SDK crates (`zip`, `tar`, `flate2`).

### UseCase

Coordinates domain behavior through ports.

- Create archive from input paths.
- List archive members.
- Extract archive into a destination directory (with path-safety checks).

### Ports

Interfaces owned by the application core.

- `ArchiveEngine`: create, list, extract for a given format.
- `FileSystem`: list input files for create operations when needed by adapters.

Ports isolate every external resource named in `CLAUDE.md` / `AGENTS.md`
under "External Resources Must Be Ports".

### Adapters

Framework and infrastructure implementations.

- ZIP / TAR / TAR.GZ engines using crate SDKs.
- Local filesystem adapter.

Adapters must not define business policy (e.g. path-traversal rejection
belongs in domain or use case).

### Delivery

The CLI presents commands (`create`, `list`, `extract`), parses arguments,
maps exit codes, and prints human-readable results.

It must not own:

- path-safety policy for extract.
- format selection rules beyond flag/extension mapping approved in the spec.
- any policy that belongs in a use case.

## Runtime Direction

Runs as a single local process on the user's machine. No optional cloud
providers. Network access is not required for core behavior.

## Selected Technology

- Runtime/shell: local OS process / shell.
- Application language: Rust (edition 2021).
- UI framework: none (CLI only).
- Package manager: Cargo.
- Distribution goal: local binary via `cargo install` / `cargo build --release`.

## Detailed Rules

- `project-structure.md`: where files belong.
- `testing-strategy.md`: AT-TDD test placement.
- `implementation-readiness.md`: checklist before coding.
- `dependency-policy.md`: package dependency checking policy.
- `rust-cli-architecture.md`: Rust layering and module layout.
- `ai-request-routing.md`: AI payload selection and task routing.
- `io-reasoning-contracts.md`: AI input/output/reasoning contracts.
- `external-resource-adoption-contract.md`: optional contract for adopting
  external resources as ports.
