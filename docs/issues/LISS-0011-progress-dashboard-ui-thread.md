# LISS-0011: Progress dashboard (fixed-height + UI thread)

## Metadata

- Local issue ID: LISS-0011
- GitHub issue: https://github.com/nn0cl/ax/pull/4
- Status: in_review
- Phase: Phase 3 (refactor) / Fast Path with compact design note
- Type: feature / refactor
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: grok-build
- Related branch: `feature/liss-0011-progress-dashboard`
- Supersedes/extends: LISS-0008 (progress), LISS-0009 (parallel workers)

## Summary

Pack 長時間ジョブ向けの進捗 UI を、**固定行数ダッシュボード**（total + compress0..N + transfer）として安定表示する。  
描画は **専用 UI スレッド**に分離し、圧縮/転送ワーカーが TTY I/O に引きずられないようにする。  
行幅は **表示桁（CJK=2）** で切り詰め、折り返しによる画面崩壊を防ぐ。

## User story

As an operator packing multi-GB trees,  
I want a stable on-screen dashboard of total / compress workers / transfer,  
so that I can monitor progress without the terminal filling with scroll junk or freezing.

## Acceptance Notes

### Must have

1. Progress dashboard shows **fixed height**: 1 total + N compress rows + 1 transfer (`N = --jobs`).
2. Dashboard is **redrawn in place** (no unbounded scroll of status lines under normal TTY).
3. Compress and transfer workers **do not write to the TTY** for progress; they only update shared state.
4. A dedicated UI thread paints on a timer (~200ms).
5. Rows respect **display width** (CJK ≈ 2 columns); default max ~72 columns so lines do not wrap.
6. Path segments truncated; speeds/ETA fields not mid-garbled by whole-line middle-truncate.
7. Tests cover: dashboard height, CJK width, progress module unit tests; existing pack_restore still green.
8. `-vv` remains for **text log detail** (per-unit plan/transfer lines); dashboard layout is not `-vv`-only.

### Explicit non-goals

- Pixel-perfect TUI frameworks (ratatui, etc.).
- Guaranteed ETA accuracy on all media.
- Windows console certification.

## Dependencies

- Parent: WP-0002 / product pack path
- Depends on: LISS-0008, LISS-0009 (done)
- Related: LISS-0012 (ax CLI)

## Adjudicator Decision Points

- Fast Path allowed for UI refactor with compact design note (readable modules + tests).
- Blanket product direction: monitorable pack jobs without log flood.

## Context

- Included: `src/delivery/progress/*`, CLI wiring for progress.
- Omitted: full redesign of pack pipeline concurrency model.
- Assumptions: operator uses a real TTY (`/dev/tty` or stderr TTY); pipes disable UI.

## AI Planning Records

### AIP-0011-001

- Status: accepted (execute)
- Created by:
  - Agent/environment: Grok Build
  - Model as displayed: Grok
  - Reasoning setting as displayed: n/a
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path light / Fast Path refactor with tests
- Intended scope: progress UI refactor + docs/issue/trace + PR
- Estimated token range: n/a (interactive session)
- Token metric: n/a
- Estimation basis: ongoing session
- Assumptions: gh authenticated; origin is nn0cl/ax
- Confidence: high
- Revises: ad-hoc main-branch progress patches without issue
- Revision reason: restore llm-project-template process (issue + branch + PR)

## References

- `docs/architecture/testing-strategy.md`
- `docs/collaboration/source-code-quality.md`
- `docs/collaboration/branch-commit-pr-discipline.md`

## Work Notes

- Modules: `rate`, `width`, `render`, `tty`, `mod` (UI thread + state).
- Removed tangled `draw.rs` / `format.rs` after split.

## Verification

- [x] `cargo test`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo build --release`
- [ ] PR CI green
