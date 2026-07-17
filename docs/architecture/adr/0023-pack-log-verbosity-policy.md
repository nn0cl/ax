# ADR 0023: Pack log verbosity policy (`-v` / `-vv`)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: LISS-0014, ADR 0022 (dashboard is independent)

## Context

Early pack logged every planned unit and every transfer, which produced tens of
thousands of lines on large trees. Operators need a quiet default, optional
detail, and a clear split from the live dashboard.

## Dependency Adoption Evidence

Not applicable (clap `ArgAction::Count`).

## Decision

1. **Dashboard** (ADR 0022) is shown whenever progress UI is enabled — **not**
   gated on `-v`.
2. **Verbosity levels** for **text logs** (stdout/stderr messages, not the
   fixed dashboard block):

   | Level | Flag | Text logs |
   |-------|------|-----------|
   | 0 | (default) | Summary lines only (`budget=`, `size_estimate:` short, `resource:` compact, `summary:`, …) |
   | 1 | `-v` | Same as 0 for unit dumps; reserved for light notes if needed |
   | 2+ | `-vv` | Capped per-unit `plan …` lines, capped `transferred …` lines, full content-class `size_estimate` breakdown |

3. **Never** dump unbounded per-unit plan lines at level 0/1.
4. Caps when level ≥ 2 (e.g. first 20 plan units, first 30 transfers) with an
   “and N more” marker.
5. Config `[pack] verbose=true` maps to level **1** (not 2).
6. Resource preflight: one compact line + Warn/Fail only (no routine Ok spam).

## Consequences

Positive:

- Large packs remain operable in a terminal.
- Detail available when debugging (`-vv`).

Negative:

- Operators must learn that the dashboard does not require `-v`.
- Level 1 is intentionally thin (most “detail” is level 2).

## Enforcement

Code review should reject:

- Reintroducing unbounded `messages.push` per unit at default verbosity.
- Coupling “show compress worker rows” exclusively to `-v` (dashboard always shows them when UI is on).
