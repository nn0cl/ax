# LISS-0011: Progress dashboard (fixed-height + UI thread)

## Metadata

- Local issue ID: LISS-0011
- GitHub issue / PR: https://github.com/nn0cl/ax/pull/4
- Status: in_review
- Phase: Phase 3 (refactor) / Fast Path with compact design note
- Type: feature / refactor
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: grok-build
- Related branch: `feature/liss-0011-progress-dashboard`
- ADR: **0022** (dashboard + UI thread), **0023** (log verbosity), **0024** (screen geometry)
- Related: LISS-0015 / ADR 0025 (`set_jobs` when auto-jobs retunes after plan)
- Supersedes/extends: LISS-0008 (progress), LISS-0009 (parallel workers)

## Summary

Pack 長時間ジョブ向けの進捗 UI を、**固定行数ダッシュボード**（`total` + `cmp0..N` + `trn`）として安定表示する。  
描画は **専用 UI スレッド**に分離し、圧縮/転送ワーカーが TTY I/O に引きずられないようにする。  
行幅は **Linux コンソール 80 列を設計下限**とし（ADR 0024）、数値は固定幅右寄せ、ファイル表示は **basename + 安定タグ**。広い端末（≥120、上限 200）では compress 中に **`archive | source`** を出せる。

## User story

As an operator packing multi-GB trees,  
I want a stable on-screen dashboard of total / compress workers / transfer,  
so that I can monitor progress without the terminal filling with scroll junk,
horizontal jitter, or line garble.

## Acceptance Notes

### Must have

1. Progress dashboard shows **fixed height**: 1 total + N compress rows + 1 transfer (`N = --jobs`).
2. Dashboard is **redrawn in place** (no unbounded scroll of status lines under normal TTY).
3. Compress and transfer workers **do not write to the TTY** for progress; they only update shared state.
4. A dedicated UI thread paints on a timer (~150–200ms).
5. Rows respect **display width** (CJK ≈ 2 columns); design baseline **80** columns (ADR 0024).
6. **No whole-row middle-truncate** of metrics (historical `…TA` / `…fer` bugs).
7. **Numeric fields** fixed-width, right-aligned (counts, %, rates, durations).
8. Row tags: **`total` / `cmp0..` / `trn`** with gutters; `total` has **no path** so full `elapsed` / `ETA` fit.
9. File labels: **`name·xxxx`** (stable path tag from full rel_path); not pure random; same basenames disambiguated.
10. Wide TTY (≥120, cap 200): compress may show **`archive | source`** name·tags; per-member progress during compress.
11. Tests cover: height, CJK/width, fixed numerics, tags, wide dual labels; existing pack tests still green.
12. `-vv` remains for **text log detail**; dashboard is not `-v`-only (ADR 0023).

### Explicit non-goals

- Pixel-perfect TUI frameworks (ratatui, etc.).
- Guaranteed ETA accuracy on all media.
- Windows console certification.
- Full directory paths on the dashboard (name·tag only by design).

## Dependencies

- Parent: WP-0002 / product pack path
- Depends on: LISS-0008, LISS-0009 (done)
- Related: LISS-0012 (ax CLI), LISS-0014 (log verbosity)

## Adjudicator Decision Points

- Fast Path allowed for UI refactor with compact design note (readable modules + tests).
- Blanket product direction: monitorable pack jobs without log flood.
- File labels use **deterministic short tags**, not random IDs (sort/log stability).

## Context

- Included: `src/delivery/progress/*`, pack `report_worker` / compress progress callbacks, CLI wiring.
- Omitted: full redesign of pack pipeline concurrency model.
- Assumptions: operator uses a real TTY when dashboard is desired; pipes disable UI; width probe is best-effort.

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

### AIP-0011-002

- Status: accepted (execute)
- Created at: 2026-07-17 (continued session)
- Intended scope: fixed-width numerics; elapsed/ETA; cmp/trn tags; name·tag;
  wide dual labels; compress member progress; docs/README/issue alignment
- Confidence: high

## References

- ADR 0022, 0023, 0024
- `docs/architecture/design-notes/2026-07-17-liss-0011-progress-dashboard.md`
- `docs/architecture/design-notes/2026-07-17-adr-0024-cli-screen-impl.md`
- `docs/collaboration/traces/2026-07-17-liss-0011-progress-dashboard.md`
- `docs/collaboration/branch-commit-pr-discipline.md`

## Work Notes

- Modules: `rate`, `width`, `render`, `tty`, `mod` (UI thread + state).
- Engine: `create_members_with_progress` for per-file compress reporting.
- Commits on branch include `ef06462` (stable fixed-width dashboard + wide dual labels).

## Verification

- [x] `cargo test`
- [x] `cargo clippy --lib -- -D warnings` (and broader as available)
- [x] `cargo build --release`
- [ ] PR CI green / merge to main
