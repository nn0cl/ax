# ADR 0017: Config file (`.inf` / `.ini`) and CLI precedence

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: global `--config`, `[pack]` section, `examples/archiver.inf`

## Context

Repeated pack invocations (USB jobs, cron-like scripts) need stable defaults
without long flag lines. External deps for full TOML/YAML were unnecessary
for a flat key=value surface.

## Dependency Adoption Evidence

Not applicable (in-tree INI parser, no new crate).

## Decision

1. Support **INI-style** files: `[section]`, `key=value`, `#`/`;` comments,
   optional quotes. Extensions: `.inf`, `.ini`, or any path of that content.
2. Global CLI: `--config PATH` (applies to subcommands that merge it; pack is
   primary consumer).
3. Pack keys live under **`[pack]`** (case-insensitive section/keys).
4. **Precedence**: CLI flag wins over config; config wins over built-in
   default. For boolean “on” flags (`--verify-stage`, `--resume`, …), CLI
   true or config true enables; no separate `--no-*` required for v0.x
   unless later demanded.
5. `source` / `output` may come from config alone so
   `archiver pack --config job.inf` is valid.
6. Unknown keys are ignored (forward compatible); invalid values error.

## Consequences

Positive:

- Script-friendly jobs; no new dependency
- Clear override rule

Negative:

- No nested structures / includes / env expansion in v0.x
- Boolean “force false over config true” needs future `--no-*` if required

## Enforcement

Code review should reject:

- Config silently overriding an explicitly set CLI value
- Pulling a heavy config crate without revisiting this ADR
