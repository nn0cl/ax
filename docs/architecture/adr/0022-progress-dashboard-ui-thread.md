# ADR 0022: Fixed-height progress dashboard and dedicated UI thread

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: LISS-0011, LISS-0008/0009, design note 2026-07-17-liss-0011

## Context

Long pack jobs need live visibility of compress workers and transfer. Naïve
logging per unit floods the terminal. Multi-line status drawn from worker
threads caused freezes and scroll garble (especially with CJK paths wrapping).

## Dependency Adoption Evidence

Not applicable (stdlib + ANSI only; no TUI crate).

## Decision

1. **Dashboard layout** (fixed height = `jobs + 2`):

   ```text
   total     …
   compress0 …
   compressN …
   transfer  …
   ```

2. **Separation of concerns**:
   - Pack workers: `report` / `report_worker` only update shared state.
   - UI thread (`ax-progress`): ~200 ms tick → pure render → ANSI paint.
3. **Paint contract**: fixed row count; rewrite in place (cursor up + clear line).
   Cursor ends **below** the block between frames.
4. **Display width**: see **ADR 0024** (Linux console **80-column** baseline;
   display-width truncation; paths truncated; rows must not wrap).
5. **Module split** (delivery only): `rate`, `width`, `render`, `tty`, `mod`.
6. **Disabled** when no TTY (`/dev/tty` or stderr not a terminal): no dashboard.
7. **Verbose text logs** (`-v` / `-vv`) are separate from dashboard layout
   (ADR 0023).

> **Note:** Early drafts of this ADR mentioned ~72 columns. **Superseded by
> ADR 0024** (`ROW_COLS` / `MIN_CONSOLE_COLS` = **80**).

## Consequences

Positive:

- Operators see workers + transfer without log flood.
- Pack CPU/IO not blocked on terminal writes.

Negative:

- Broken / piped terminals lose the dashboard.
- ANSI-only; not a full TUI toolkit.

## Enforcement

Code review should reject:

- Calling TTY write APIs from compress/transfer threads for progress.
- Per-unit `println` progress as the default path.
- Whole-line middle-truncate of rows that contain speeds/ETA (garble).
