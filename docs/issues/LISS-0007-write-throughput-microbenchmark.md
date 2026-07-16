# LISS-0007: 書き込みスループット微測定による見積精度向上

## Metadata

- Local issue ID: LISS-0007
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature
- Priority: P2
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/write-benchmark`
- Work plan: WP-0002 Slice S4

## Summary

保存先に対して **短いシーケンシャル書き込みサンプル** を行い、実効 throughput
(bytes/sec) を測定して `estimate` / `--auto-size` に反映するオプションを追加する。

接続クラスのテーブル値より、実 USB スティックの遅さに追従できる。

## User story

As a user with a slow USB flash drive that is USB3-labeled but writes slowly,  
I want an optional quick write probe,  
so that archive unit sizes match real transfer speed—not marketing bus speed.

## Acceptance Notes

### Must have

1. CLI:
   - `estimate --path P --measure`（名称は実装時に確定可）
   - `pack --auto-size --measure`
2. 測定は出力先ボリューム上の一時ファイルに **シーケンシャル write + fsync 相当** を行い、
   終了後に一時ファイルを削除する。
3. 測定パラメータ（仕様に既定値）:
   - sample size 既定例: 64 MiB（空きが少ない場合は縮小）
   - 失敗時: テーブル値にフォールバックし warning を stderr へ
4. 測定結果は estimate レポートに `measured_throughput` として表示。
5. テスト: probe を fake して「測定値があればテーブルを上書き」を純関数/use case で固定。
   実ディスク測定は `#[ignore]` の手動テストでよい。

### Explicit non-goals

- 長時間ベンチ、ランダム I/O プロファイル、GUI。
- 測定中の他プロセス干渉の完全排除。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0006（接続クラスと probe 基盤）
- Blocks: none
- Related: LISS-0005（測定がディスクを触る点で運用注意）

## Adjudicator Decision Points

- [ ] 既定で measure するか（推奨: **オプトイン**）
- [ ] sample size / タイムアウト上限
- [ ] pack --auto-size 時に measure を許可するか（副作用: 出力先に一時書込）

## Context

- Included: MediaProbe or new `ThroughputMeter` port, estimate use case, CLI flags
- Omitted: continuous monitoring
- Assumptions: users accept brief write to destination for accuracy

## AI Planning Records

### AIP-0007-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path; careful temp-file cleanup (defer/finally)
- Intended scope: optional write sample → throughput override
- Estimated token range: 50k–110k
- Estimated token midpoint: 75k
- Token metric: rough session tokens
- Estimation basis: port + adapter + CLI + fake tests
- Assumptions: opt-in only
- Confidence: medium
- Revises:
- Revision reason:
- Superseded by:

## References

- domain `estimate_transfer_unit` (`throughput_bytes_per_sec` override already exists)
- OS sequential write best practices (buffer size, fsync policy — decide in design intake)

## Work Notes

### Safety

- Never measure on read-only media without clear error.
- Cap sample to `min(configured, free_bytes/4)`.
- Use unique temp name under output path; delete on all exit paths.

## Verification

- [ ] Fake meter unit/use-case tests
- [ ] Manual: `--measure` on real USB shows different unit than table-only when drive is slow
- [ ] Temp file absent after estimate
