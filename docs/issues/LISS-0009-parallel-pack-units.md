# LISS-0009: 独立 PackUnit の並列圧縮

## Metadata

- Local issue ID: LISS-0009
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature / performance
- Priority: P2
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/parallel-pack`
- Work plan: WP-0002 Slice S6

## Summary

互いに独立した `PackUnit` のアーカイブ生成を **並列実行** し、マルチコアと
`tar.zst` の併用で wall-clock を短縮する。マッピングの内容的一貫性と
決定的な archive ID 採番を維持する。

## User story

As a user packing large trees on a multi-core machine,  
I want independent folder archives to compress in parallel,  
so that total pack time drops without changing restore semantics.

## Acceptance Notes

### Must have

1. `pack --jobs N`（`N=1` が現行と同等、既定は Adjudicator 決定: 推奨 `num_cpus` or 1）。
2. 同一入力・同一 budget で:
   - member 集合と archive_relpath の対応が逐次実行と一致（ID は安定ソート後に採番）
3. エラー時: いずれかの unit が失敗したら全体失敗（部分成功の扱いを仕様化; 推奨: fail-fast or collect — **推奨 collect-all-then-fail** で cleanup 方針を明記）。
4. テスト: jobs=1 と jobs=4 で mapping members の multiset 一致。
5. 仕様に並列の決定性ルールを追記。

### Explicit non-goals

- 単一 unit 内のファイル並列（フォーマット制約が大きい）。
- クラスタ分散。
- 実行中の jobs 動的上下（→ LISS-0015 / ADR 0025 は **plan 後 1 回**の auto のみ）。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0004（FS/ports が整理されていること）
- Blocks: none
- Follow-up: **LISS-0015** (`--at` I/O-aware auto jobs, ADR 0025)
- Related: LISS-0008 (progress under concurrency)

## Adjudicator Decision Points

- [ ] 既定 jobs: 1（安全） vs CPU 数
- [ ] 失敗時セマンティクスと中間ファイル削除
- [ ] Phase 1 Red 承認

## Context

- Included: application pack orchestration, thread pool (`rayon` or `std::thread`)
- Omitted: UI
- Assumptions: units do not share output paths (placement ensures unique part-NNNN)

## AI Planning Records

### AIP-0009-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path; dependency adoption note for rayon if used
- Intended scope: parallel create_members per unit
- Estimated token range: 60k–130k
- Estimated token midpoint: 90k
- Token metric: rough session tokens
- Estimation basis: concurrency + determinism tests
- Assumptions: archive paths unique per unit
- Confidence: medium
- Revises:
- Revision reason:
- Superseded by:

## References

- https://docs.rs/rayon (candidate)
- `docs/architecture/dependency-policy.md`

## Work Notes

### Determinism recipe

1. `plan_pack` → units (already ordered)
2. `assign_archive_paths` → stable paths
3. Assign IDs `a0001…` in that order **before** parallel create
4. Parallel only the create_members calls
5. Build mapping from preassigned metadata + results

## Verification

- [ ] jobs=1 vs jobs>1 mapping multiset equal
- [ ] stress small tree under race-prone FS (tempdir)
- [ ] clippy / test green
