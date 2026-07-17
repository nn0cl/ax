# Design note: Pack text logging and verbosity

## Path

Fast Path / UX (retro + ADR 0023).

## Problem

Per-unit `plan …` lines on trees with 10⁵+ dirs made terminals unusable.

## Solution

| Channel | Behavior |
|---------|----------|
| **Dashboard** | Fixed block; always on when TTY progress enabled (ADR 0022) |
| **Text logs** | Summary by default; detail at `-vv` (ADR 0023) |

### Summary line filter (CLI)

Allowlist prefixes such as: `budget=`, `size_estimate:`, `resource:`,
`pipeline:`, `summary:`, `bytes:`, stage cleanup lines, …

### Caps at `-vv`

- Plan detail: first N units  
- Transfer detail: first M units  

## Implementation map

| Concern | Location |
|---------|----------|
| Verbosity count | `cli/args` (`ArgAction::Count`) |
| Merge config | `pack_config` (`verbose` u8) |
| Unit plan push | `application/pack` (only if verbose ≥ 2) |
| Transfer log | `pipeline` (`log_transfers`) |
| Print filter | `cli/mod` `is_pack_summary_line` |

## Tests

- dry-run quiet vs `-vv` class breakdown  
