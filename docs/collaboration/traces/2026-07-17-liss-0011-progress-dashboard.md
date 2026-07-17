# AI work trace: LISS-0011 Progress dashboard

## Metadata

- Date: 2026-07-17
- Issue: LISS-0011
- Branch: `feature/liss-0011-progress-dashboard`
- Related: LISS-0012 (ax CLI, retro), LISS-0014 (logs), process restore after main-direct work
- Agent: Grok Build
- PR: https://github.com/nn0cl/ax/pull/4

## Request summary

User asked to restore llm-project-template process items 1–4, then iterated on
live pack dashboard quality (jitter, garble, labels, width), then:

- Align **design docs / README / ISSUE** with the implemented UI
- Commit and push

## Path selected

Fast Path / readability refactor with compact design notes  
(`docs/architecture/design-notes/2026-07-17-liss-0011-progress-dashboard.md`,  
`docs/architecture/design-notes/2026-07-17-adr-0024-cli-screen-impl.md`).

## Changes (code — landed on branch)

- `src/delivery/progress/`: `rate`, `width`, `render`, `tty`, `mod`
- UI thread paints; workers only update state
- Fixed-width numerics; `elapsed` / `ETA`; short tags `cmp` / `trn`
- File labels `name·tag` (stable path hash); dual `archive | source` when wide
- Width: 80 baseline; ≥120 → up to 200; probe best-effort
- `create_members_with_progress` + `report_worker(primary, source, Δ)`
- Key commit: `ef06462` feat(progress): stable fixed-width dashboard with wide dual labels

## Changes (docs — this unit)

- ADR 0022 / 0024 + `adr/README.md` (geometry, tags, wide mode, verification)
- LISS-0011 acceptance criteria refreshed
- Design notes + this trace updated
- `README.ja.md` progress section; short note in `README.md`
- `ax-product-design-book.md` / `docs/QA.md` touch-ups

## Verification

- `cargo test --lib delivery::progress`
- `cargo clippy --lib -- -D warnings`
- `cargo build --release`

## Process notes

- Prior `ax` CLI landed on `main` without issue (LISS-0012 documents retroactively).
- This PR re-establishes branch + PR discipline for the progress work.
- Further feature work should open LISS + feature branch before coding.

## Follow-up on same branch (auto-jobs)

- Code: `src/core/domain/jobs.rs`, `--at` / `--auto-jobs`, I/O-aware caps
- Docs: **ADR 0025**, **LISS-0015**, design note `2026-07-17-auto-compress-jobs.md`
- Decision: plan-time fixed N only; no mid-run thread up/down (idle yields CPU)

## Handoff

- PR: https://github.com/nn0cl/ax/pull/4 — **merged** to `main` (squash).
- Operator: `ax pa …` on a real TTY; try `stty cols 80` and `120`.
- Auto jobs: `ax pa … --at --connection usb3 --work-dir /tmp/…`
