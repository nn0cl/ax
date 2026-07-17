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

This ADR consolidates **CLI screen design rules**, fixes the **minimum design
width** to the Linux console standard, and defines optional **wide mode**.

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

Terminal width is **best-effort**: `COLUMNS` env, then Unix `TIOCGWINSZ` on
`/dev/tty`. If probing fails (pipe, CI, no TTY), fall back to **80**. Rust
running does **not** guarantee a width is available.

### 2. What must fit (layout rules)

1. Every progress dashboard line after truncation must not wrap at the effective
   row width.
2. Prefer **fixed fields first** (counts, %, speeds, elapsed/ETA, phase); **labels last**.
3. Labels use **display-width** awareness (ASCII ≈ 1 col, CJK/fullwidth ≈ 2 cols).
4. Do **not** middle-truncate whole rows in a way that corrupts speeds/ETA
   (historical bug: `…fer`, `…psed`, `…TA` from `ETA`). Only the **path label**
   segment may ellipsize; if the metrics prefix itself overflows a narrow
   terminal, hard-clip from the **left** (keep leading fields).
5. **Numeric fields are right-aligned with fixed maximum display widths** so values
   do not shift left/right as magnitudes change during a run (counts, percents,
   compact rates, durations). Clock is a fixed-width
   `elapsed <dur>  ETA <dur>` block (durations right-aligned to 6 cols).
6. Metrics prefix must fit inside **80 columns**. The **`total`** row has **no
   path** so full `elapsed` / `ETA` labels fit; file labels appear only on
   `cmp*` / `trn`.
7. **Short row tags** with a fixed label column and double-space gutters:
   `total` / `cmp0`…`cmpN` / `trn` (not long `compress*` / `transfer`).
8. **File labels** are **basename + stable 4-hex path tag**
   (`report.docx·a3f2`), derived from the full relative path (FNV-1a). Not pure
   random (must be stable for logs). Same basename, different folders → different
   tags. Underlying plan/sort order remains full `rel_path`.
9. **Wide mode (≥ 120 cols, ≤ 200)**: `cmp` may show **`archive | source`**
   (both name·tag) while packing. Baseline 80-col mode shows a single label.

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
| **Column budget & 80-col baseline + wide** | **This ADR (0024)** |

ADR 0022 § display width “~72 cols” is **superseded** by this ADR’s **80-col**
baseline for the default row cap.

### 6. Narrower / wider than 80

| TTY width | Behavior |
|-----------|----------|
| &lt; 80 | Best-effort only (may wrap); not a design target (ADR 0019 style). |
| 80–119 | Cap rows at **80**. |
| ≥ 120 | Use `min(detected, 200)` — dual labels allowed on `cmp`. |
| Probe fails | **80**. |

## Consequences

Positive:

- Clear QA baseline (“test at 80×24”).
- Aligns with Linux console defaults and many SSH clients.
- Reduces wrap-induced UI corruption and numeric jitter.
- Wide terminals can see archive and source file names without directories.

Negative:

- Directory context is not on the dashboard (by design: name·tag only).
- Very high `-j` on 24-row screens leaves little scrollback for sticky logs.
- Width probe is environment-dependent.

## Enforcement

Code review should reject:

- Raising default `ROW_COLS` / baseline above 80 without revisiting this ADR.
- Progress rows built without display-width awareness for non-ASCII.
- Assuming IDE-width alone for “readable” layout without an 80-col path.
- Whole-row middle-truncate of metrics (speeds / `elapsed` / `ETA`).
- Pure random IDs in file labels (breaks stability / comparability).

## Implementation map

| Item | Location |
|------|----------|
| Constants | `progress/width.rs` (`MIN_CONSOLE_COLS`, `ROW_COLS`, `WIDE_*`, `MAX_ROW_COLS`) |
| `effective_row_cols` / `is_wide_layout` | `progress/width.rs` |
| File label + `path_tag` | `progress/width.rs` (`display_file_label`) |
| TTY probe | `probe_terminal_columns` (`COLUMNS` + `TIOCGWINSZ`) |
| Render layout | `progress/render.rs` `Layout`, `fmt_*`, `DUAL_SEP` (` \| `) |
| Worker source path | `WorkerView.source`; `report_worker(primary, source, …)` |
| Per-member compress progress | `create_members_with_progress` + pack `unit_work` |
| UI wiring | `progress/mod.rs` (probe once at start) |

## Verification notes

- Unit tests: row `display_width ≤` effective width; CJK; fixed-width numerics;
  stable columns when magnitudes change; basename tags differ for same names;
  wide dual `archive | source`; `effective_row_cols(Some(120)) == 120`,
  `Some(250) == 200`, `Some(100) == 80`.
- Manual: `stty cols 80` and `stty cols 120` while running `ax pa …`.
