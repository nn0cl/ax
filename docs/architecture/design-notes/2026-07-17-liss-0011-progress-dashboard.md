# Compact design note: LISS-0011 Progress dashboard

## Path

**Fast Path / readability refactor** of pack progress (extends LISS-0008/0009).  
Behavior evolved through operator feedback on jitter, garble, and labels.

## Scope

- In: `src/delivery/progress/*`, pack `report_worker` / compress member progress,
  ADR 0022–0024 alignment, tests for height/width/labels.
- Out: pack pipeline concurrency redesign, Windows console certification.

## Design

```text
Workers: report / report_worker(primary, source, Δ) → ProgressState (Mutex only)
UI thread "ax-progress": every ~150–200ms → render_dashboard (pure) → TtyBlock.paint
```

| Module | Responsibility |
|--------|----------------|
| `rate` | EMA throughput; `WorkerView` (`current` + `source`) |
| `width` | display columns (CJK=2), `effective_row_cols`, `display_file_label` / `path_tag` |
| `render` | pure dashboard strings; fixed-width nums; `cmp`/`trn`; dual ` \| ` |
| `tty` | cursor-up paint contract |
| `mod` | shared state + UI thread + PackProgress |

### Layout (normative sketch)

```text
total      1/   31   33.5%     6.1G/s  elapsed  4m17s  ETA 20h50m  [hash ]
cmp0     474.0M/s  [compr]  part-0004.zip·a1b2
cmp1     410.5M/s  [hash ]  report.docx·c3d4
trn      153.1M    0.1%   816.3K/s  elapsed  4m17s  ETA 20h50m  part-0002.zip·9c1b

# TTY ≥ 120 (cap 200), compress:
cmp0     474.0M/s  [compr]  part-0004.zip·a1b2 | report.docx·c3d4
```

- **`total`**: no file path (room for `elapsed` / `ETA` words).
- **Rates**: compact fixed field (` 144.3M/s`), decimal column stable.
- **Tags**: FNV-1a 4-hex from full rel_path — not random; same basename differs by folder.
- **Sort**: plan/unit order remains full `rel_path`; tags are display-only.

### Width policy (ADR 0024)

| Detected cols | Paint width |
|---------------|-------------|
| unknown / fail | 80 |
| &lt; 120 | min(detected, 80) |
| ≥ 120 | min(detected, 200) |

## Why full AT-TDD scaffold is unnecessary

- Acceptance is operational UI stability (not new domain pack rules).
- Regression covered by unit tests (height, width, tags, dual) + pack tests.
- Spec surface: progress remains best-effort TTY UI (ADR 0019 / 0024 probe).

## Deterministic checks

```bash
cargo test --lib delivery::progress
cargo clippy --lib -- -D warnings
cargo build --release
```

## Risks

- Broken TTYs / pipes: UI disabled; sticky messages on stderr.
- Width probe unavailable: always 80 (no dual labels).
- Tag collisions theoretically possible (16-bit); display-only risk.

## Normative ADRs

- ADR 0022 — dashboard + UI thread  
- ADR 0023 — text log verbosity (dashboard independent of `-v`)  
- **ADR 0024 — CLI screen geometry** (80 baseline, wide 120–200, labels)

## Next

- Merge PR #4 when CI green; keep operator feedback on tag readability.
