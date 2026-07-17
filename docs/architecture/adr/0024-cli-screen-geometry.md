# ADR 0024: CLI screen geometry (Linux console width baseline)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: ADR 0020 (CLI surface), ADR 0022 (dashboard + UI thread), ADR 0023
  (text logs), LISS-0011

## Context

Live pack progress uses a **fixed multi-line block** (total + compress workers +
transfer). If any row wraps, ANSI cursor-up math breaks and lines concatenate
(especially with CJK paths, which are ~2 columns per character).

Operators often use plain Linux consoles and SSH sessions whose traditional
default geometry is **80×24** (VT / Linux virtual console). Designing for a
wider IDE terminal hides wrap bugs that appear on 80-column hosts.

This ADR consolidates **CLI screen design rules** and fixes the **minimum
design width** to the Linux console standard.

## Dependency Adoption Evidence

Not applicable (presentation rules only; no new crate).

## Decision

### 1. Baseline geometry (normative)

| Symbol | Value | Meaning |
|--------|------:|---------|
| **`MIN_CONSOLE_COLS`** | **80** | Minimum horizontal width the CLI UI is **designed and tested** for. Matches classic **Linux console / VT** text width (80 columns). |
| **`MIN_CONSOLE_ROWS`** | **24** | Soft reference for “small screen”; dashboard must still fit when `jobs` is moderate (see height). |
| **`ROW_COLS`** | **80** | Default hard cap for each painted row when TTY &lt; 120. |
| **`WIDE_CONSOLE_COLS`** | **120** | At this width and above, unlock wider rows (up to `MAX_ROW_COLS`). |
| **`MAX_ROW_COLS`** | **200** | Cap for wide mode. |

Implementation: `src/delivery/progress/width.rs` (`effective_row_cols`).

### 2. What must fit in 80 columns

1. Every progress dashboard line (total / compress* / transfer) after truncation.
2. Prefer **fixed fields first** (counts, %, speeds, elapsed/ETA, phase); **paths last**.
3. Paths use **display-width** truncation (ASCII ≈ 1 col, CJK/fullwidth ≈ 2 cols),
   with a dedicated path budget (`PATH_COLS`, implementation may tune ≤ 20–24 cols).
4. Do **not** middle-truncate whole rows in a way that corrupts speeds/ETA
   (historical bug: `…fer`, `…psed`, `…TA` from `ETA`). Only the **path**
   segment may use middle-ellipsis; if the metrics prefix itself overflows a
   narrow terminal, hard-clip from the **left** (keep leading fields).
5. **Numeric fields are right-aligned with fixed maximum display widths** so values
   do not shift left/right as magnitudes change during a run (counts, percents,
   compact rates, durations). Clock is a fixed-width
   `elapsed <dur>  ETA <dur>` block (durations right-aligned to 6 cols).
6. Metrics prefix must be designed to **fit inside 80 columns**. The **`total`**
   row has **no path** so full `elapsed` / `ETA` labels fit; paths appear only on
   `cmp*` / `trn` (truncated within the path budget).
7. **Short row tags** with a fixed label column and double-space gutters between
   major fields: `total` / `cmp0`…`cmpN` / `trn` (not `compress*` / `transfer`).
8. **Wide mode (≥ 120 cols)**: `cmp` rows may show **archive | source** while
   packing (file **names** + stable path tags only). Baseline 80-col mode keeps
   a single name. Directory paths are not required.

### 3. Height (dashboard)

Unchanged from ADR 0022 in spirit:

- Height = **`jobs + 2`** (1× total + N× compress + 1× transfer).
- Rows are rewritten in place; line count stays constant for a run.
- At 24-row consoles, large `-j` may fill the screen; default jobs remain small
  (≤ 8). No automatic job clamp required in this ADR.

### 4. Sticky text vs dashboard

- Sticky messages (`pack source=…`, `scanning …`) print **above** the block
  (normal newlines).
- Summary / `-vv` text logs follow ADR 0023 and must not assume width > 80
  for critical tokens (prefer short prefixes).

### 5. Relationship to other ADRs

| Topic | Authority |
|-------|-----------|
| Binary name, short commands | ADR 0020 |
| Dashboard threads / paint isolation | ADR 0022 |
| `-v` / `-vv` text volume | ADR 0023 |
| **Column budget & 80-col baseline** | **This ADR (0024)** |

ADR 0022 § display width “~72 cols” is **superseded** by this ADR’s **80-col**
baseline for row caps.

### 6. Narrower than 80

- **Not supported** as a design target: terminals &lt; 80 columns may wrap;
  behavior is best-effort only (ADR 0019 style).
- Wider terminals: still cap rows at 80 so layout stays stable and tested.

## Consequences

Positive:

- One clear number for QA (“test at 80×24”).
- Aligns with Linux console defaults and many SSH clients.
- Reduces wrap-induced UI corruption.

Negative:

- Paths shown shorter than on wide IDE terminals.
- Very high `-j` on 24-row screens leaves little scrollback for sticky logs.

## Enforcement

Code review should reject:

- Raising default `ROW_COLS` above 80 without revisiting this ADR.
- Progress rows built without display-width awareness for non-ASCII.
- Assuming IDE-width (e.g. 120+) for “readable” dashboard layout.

## Implementation map

| Item | Location |
|------|----------|
| Constants | `progress/width.rs` (`MIN_CONSOLE_COLS`, `ROW_COLS`) |
| `effective_row_cols` | `progress/width.rs` |
| TTY probe | `probe_terminal_columns` (`COLUMNS` + `TIOCGWINSZ`) |
| Render layout | `progress/render.rs` `Layout` |
| Fixed-width numerics | `progress/render.rs` (`W_COUNT`/`W_BYTES`/`W_PCT`/`W_DURATION`, `fmt_*`) |
| UI wiring | `progress/mod.rs` (probe once at start) |

## Verification notes

- Unit tests: row `display_width ≤ ROW_COLS` for sample dashboards (incl. CJK);
  `effective_row_cols(Some(120)) == 80`.
- Unit tests: fixed-width right-aligned numerics (`fmt_count` / `fmt_bytes` /
  `fmt_pct` / duration / time block) and stable column positions when magnitudes
  change (`counts_do_not_shift_total_line`, `bytes_do_not_shift_transfer_line`).
- Manual: `stty cols 80` (or 80×24 console) while running `ax pa …`.
