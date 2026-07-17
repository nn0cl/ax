# Compact design note: LISS-0011 Progress dashboard

## Path

**Fast Path / readability refactor** of existing pack progress (extends LISS-0008/0009).  
Full `[THOUGHT]` scaffold omitted: behavior already in production use; change is structure + stability.

## Scope

- In: `src/delivery/progress/*`, CLI progress wiring, tests for height/width.
- Out: pack pipeline concurrency redesign, Windows console certification.

## Design

```text
Workers: report / report_worker → ProgressState (Mutex only)
UI thread "ax-progress": every ~200ms → render_dashboard (pure) → TtyBlock.paint (ANSI)
```

| Module | Responsibility |
|--------|----------------|
| `rate` | EMA throughput, worker ticks |
| `width` | display columns (CJK=2), truncate |
| `render` | pure dashboard strings, fixed height |
| `tty` | cursor-up paint contract |
| `mod` | shared state + UI thread + PackProgress |

## Why full AT-TDD scaffold is unnecessary

- Acceptance is operational UI stability (not new domain pack rules).
- Regression covered by unit tests (height, CJK width) + existing pack_restore.
- Spec surface: progress remains best-effort TTY UI (ADR 0019).

## Deterministic checks

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

## Risks

- Broken TTYs / pipes: UI disabled (sink); sticky messages on stderr.
- Terminal width &lt; 72: path truncated more aggressively.

## Next

- PR for LISS-0011; CI gate before merge to main.
