# ADR 0021: Environment pack-size presets (`c2`, `fat32`, `flash`, `max`)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: LISS-0013, ADR 0013 (logical budget), `SizePreset` in domain

## Context

Numeric budgets (`4G`) are flexible but easy to mistype. Operators often want
named profiles for common media and transfer constraints (small units for slow
links, under FAT32 4 GiB file limit, flash-friendly 1 GiB, or “no practical
split”).

## Dependency Adoption Evidence

Not applicable (pure domain constants).

## Decision

1. `-s` / `--size` / `target_size` / config accept **numeric sizes** or **presets**.
2. Presets (case-insensitive; `_`/`-` normalized):

   | Keyword | Budget (before connection derate) | Intent |
   |---------|-------------------------------------|--------|
   | **`c2`** (aliases: `low-net`, `lownet`, `slow`) | **10 MiB** | Fine units; resume-friendly on slow paths |
   | **`fat32`** (aliases: `fat`, `usb-old`) | **3.8 GiB** | Stay under FAT32 ~4 GiB single-file limit |
   | **`flash`** (aliases: `usb`, `stick`) | **1 GiB** | Typical USB flash balance |
   | **`max`** (aliases: `nosplit`, `no-split`, `single`) | **16 TiB** | Practically one archive for normal trees |

3. Preset bytes still pass through `effective_budget` (connection factor + mapping reserve) when used as pack budget.
4. Presets do **not** change size_basis in mapping (still logical packing — ADR 0013).
5. Domain owns `SizePreset` + `parse_size` / `parse_size_with_preset`; CLI only surfaces keywords.

## Consequences

Positive:

- Short, memorable budgets for common deployments.

Negative:

- Keyword names (`c2`) may be misread outside product docs; aliases document intent.
- `max` is not infinite; pathological trees could still split if logical size exceeds 16 TiB.

## Enforcement

Code review should reject:

- Using presets as free-space hard gate inputs (gate stays logical remaining × safety).
- Hard-coding preset bytes outside domain `SizePreset::bytes`.
