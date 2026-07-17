# ADR 0020: Primary binary `ax` and short CLI surface

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: LISS-0012, ADR 0017 (config globals)

## Context

Operators run pack jobs repeatedly from a shell. Long names (`archiver pack
--source … --target-size …`) are slow to type. A short primary binary and
two-letter subcommands reduce friction without removing long aliases.

## Dependency Adoption Evidence

Not applicable (clap only; already in tree).

## Decision

1. **Primary binary name**: `ax` (Archive eXchange).  
   - `Cargo.toml`: `default-run = "ax"`.  
   - Default `cargo build` / `cargo install` produce **`ax` only**.
2. **Optional compat binary**: `archiver` via feature `compat-bin`, or
   install-time symlink `archiver` → `ax`.
3. **Short subcommands** (visible long aliases kept):

   | Short | Alias |
   |-------|--------|
   | `cr` | create |
   | `ex` | extract |
   | `ls` | list |
   | `es` | estimate |
   | `pa` | pack |
   | `re` | restore |
   | `ve` | verify |

4. **Globals** (clap global):
   - `-c` / `--config` — settings file  
   - `-s` / `--size` — pack budget size or preset  
   - `-o` / `--output` — output dir, archive path, or mapping path (command-dependent)
5. **Pack source** uses `-i` / `--source` so `-s` remains budget-only.
6. Help text documents short forms and `ax` as primary name.

## Consequences

Positive:

- Faster operator workflows; scripts can use short or long forms.

Negative:

- Docs and muscle memory must migrate from `archiver pack -s source`.
- Two binary names if compat feature enabled (build warning about shared main).

## Enforcement

Code review should reject:

- Making `archiver` the default binary again without revisiting this ADR.
- Reassigning global `-s` to mean source path.
