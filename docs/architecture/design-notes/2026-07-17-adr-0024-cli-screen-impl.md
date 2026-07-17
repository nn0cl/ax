# Plan + design: implement ADR 0024 (CLI screen geometry)

## Goal

Make the progress dashboard honor the **Linux console 80-column baseline**
end-to-end: constants, render, terminal probe, tests, docs.

## Plan

| Step | Work | Status |
|------|------|--------|
| 1 | Normative constants `MIN_CONSOLE_COLS=80`, `ROW_COLS=80` | done (prior) |
| 2 | Display-width (CJK) truncate helpers | done (prior) |
| 3 | `effective_row_cols(term)` = `min(80, term_or_80)` | **this change** |
| 4 | Probe TTY columns (`TIOCGWINSZ` / `COLUMNS`) | **this change** |
| 5 | `render_dashboard(..., row_cols)` uses effective width | **this change** |
| 6 | Unit tests for 80-col and narrow best-effort | **this change** |
| 7 | Link LISS-0011 / design book | **this change** |

## Rules (ADR 0024)

1. Design & test at **80 columns**.
2. Wider terminals: still **cap at 80** (stable layout).
3. Narrower: best-effort `min(detected, 80)` (may look tight; not certified).
4. Paths truncated by **display columns**, metrics fields first.

## Non-goals

- Full TUI frameworks.
- Dynamic reflow mid-frame (fixed height still).
- Windows console API.
