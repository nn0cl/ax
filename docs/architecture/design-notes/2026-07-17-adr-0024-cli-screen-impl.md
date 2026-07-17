# Plan + design: implement ADR 0024 (CLI screen geometry)

## Goal

Make the progress dashboard honor the **Linux console 80-column baseline**
and optional **wide mode**, end-to-end: constants, render, terminal probe,
file labels, tests, docs.

## Plan

| Step | Work | Status |
|------|------|--------|
| 1 | Normative constants `MIN_CONSOLE_COLS=80`, `ROW_COLS=80` | done |
| 2 | Display-width (CJK) helpers | done |
| 3 | `effective_row_cols`: &lt;120 → cap 80; ≥120 → min(w, **200**) | done |
| 4 | Probe TTY columns (`TIOCGWINSZ` / `COLUMNS`) | done |
| 5 | Fixed-width right-aligned numerics + `elapsed`/`ETA` clock | done |
| 6 | Short tags `total` / `cmp*` / `trn`; `total` without path | done |
| 7 | File labels `name·tag` (stable FNV path tag); dual `archive \| source` | done |
| 8 | Per-member compress progress (`create_members_with_progress`) | done |
| 9 | Unit tests (80 / wide / tags / no metric garble) | done |
| 10 | Docs: ADR 0024, LISS-0011, design notes, README.ja, trace | **this change** |

## Rules (ADR 0024 summary)

1. Design & test at **80 columns**.
2. TTY ≥ **120**: paint width up to **200**; dual labels allowed.
3. Never middle-truncate whole rows (metrics).
4. Labels are **name·tag**, not full directories; tags not pure random.
5. Width probe is best-effort; failure → 80.

## Non-goals

- Full TUI frameworks.
- Dynamic reflow mid-frame (fixed height still).
- Windows console API.
- Random IDs in on-disk archive names (display tags only).
