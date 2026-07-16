# Project Structure

This document defines the target repository shape for implementation. Create
directories only when a Phase 1 or Phase 2 task needs them.

## Target Layout

```text
.
├── docs/
├── Cargo.toml
├── src/
│   ├── main.rs                 # binary entry; wires adapters + delivery
│   ├── lib.rs                  # library root for tests and reuse
│   ├── delivery/               # CLI (clap), DTO mapping, exit codes
│   ├── core/
│   │   ├── domain/             # pure types, path safety, errors
│   │   ├── application/        # use cases
│   │   └── ports/              # ArchiveEngine, FileSystem traits
│   └── adapters/
│       ├── archive/            # zip / tar / tar.gz engines
│       └── fs/                 # local filesystem
└── tests/                      # integration tests (optional)
```

## Ownership

`src/core/domain/` owns pure domain types and domain behavior.

`src/core/application/` owns use cases and application orchestration.

`src/core/ports/` owns interfaces for external resources.

`src/adapters/` owns concrete implementations of ports.

`src/delivery/` owns CLI handlers and argument/result conversion.

## Forbidden Placement

Do not put business rules in:

- delivery/CLI handlers beyond mapping.
- adapters.
- archive crate call sites that invent policy.

Do not put framework, file-system I/O, network, or archive SDK imports in:

- `core/domain/`.
- `core/application/`, except through core-owned ports.

## Creation Rule

Create the smallest directory and module set needed by the current AT-TDD phase.
Do not scaffold unused adapters or formats.

Smallest does not mean densest. Split files and modules when it reduces human
review cost or separates architectural responsibilities.
