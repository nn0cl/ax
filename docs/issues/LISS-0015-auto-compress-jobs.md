# LISS-0015: I/O-aware auto compress jobs (`--at`)

## Metadata

- Local issue ID: LISS-0015
- GitHub issue: n/a (ship on `feature/liss-0011-progress-dashboard` / follow-up PR)
- Status: done
- Phase: complete (code + ADR 0025)
- Type: feature / performance
- Priority: P2
- Initial planning size: S
- Current planning size: S
- Owner/agent: grok-build
- Related branch: `feature/liss-0011-progress-dashboard`
- ADR: **0025**
- Related: LISS-0009 (parallel units), LISS-0011 (dashboard `set_jobs`)

## Summary

`--at` / `--auto-jobs` で圧縮ワーカー数を **plan 後に 1 回**自動決定する。  
CPU・残 unit・ファイル数に加え、**`--connection` / スループット / work-dir** で I/O 上限を掛け、遅い媒体では並列を抑える。  
実行中のスレッド数上下は **やらない**（idle 時は CPU を他プロセスに譲れる。ADR 0025）。

## User story

As an operator packing to USB or local disk,  
I want compress parallelism chosen from CPU and media class without guessing `-j`,  
so that packs stay fast on SSD but do not thrash slow sticks.

## Acceptance Notes

### Must have

1. `--at` / `--auto-jobs` (config `auto_jobs=true`) enables auto mode.
2. `-j N` always wins; auto is ignored when jobs are explicit.
3. Default without either: `min(CPU, 8)`.
4. After plan: `jobs = min(cpu_cap, units, io_cap, file rules)` (ADR 0025).
5. `io_cap` uses connection + optional throughput; work-dir relaxes slightly; no work-dir tightens.
6. Progress dashboard worker rows resize via `set_jobs`.
7. Log line `auto_jobs: cpu=… units=… files=… io_cap=… → jobs=…`.
8. Unit tests for formula and CLI merge (`-j` vs `--at`).

### Explicit non-goals

- Mid-pack dynamic thread-pool resize.
- Intra-unit file parallelism.
- Guaranteed optimal wall-clock (AS IS).

## Dependencies

- Depends on: LISS-0009 (parallel compress exists)
- Related: LISS-0011 (dashboard height)

## Work Notes

- Implementation: `src/core/domain/jobs.rs`, pack retune, CLI wiring.
- Operator guidance: prefer `--at --connection usb3 --work-dir <fast disk>`.

## Verification

- [x] `cargo test --lib` (incl. `jobs::`)
- [x] `cargo build --release`
- [ ] Optional: compare wall-clock `-j 2` vs `--at` on same USB tree
