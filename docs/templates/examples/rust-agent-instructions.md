# Rust Agent Instructions (example)

Place content like this in `<backend-dir>/AGENTS.md` when the Rust project is
created.

## Scope

This directory owns Rust application core, command handlers, ports, and
adapters.

## Required Reading

- `../docs/architecture/rust-clean-architecture.md`
- `../docs/architecture/persistence.md`
- `../docs/architecture/testing-strategy.md`

## Rules

- Domain must not import framework, DB, file-system, network, or third-party
  provider SDKs.
- Use cases depend on domain and ports.
- Command/request handlers call use cases and convert DTOs only.
- Database adapters implement persistence ports.
- Unit tests mock ports and do not require a running database or external
  service.
