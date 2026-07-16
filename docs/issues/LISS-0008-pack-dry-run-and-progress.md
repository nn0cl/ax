# LISS-0008: pack の dry-run と進捗表示

## Metadata

- Local issue ID: LISS-0008
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature
- Priority: P2
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/pack-dry-run-progress`
- Work plan: WP-0002 Slice S5

## Summary

`pack` に **ディスクへアーカイブを書かない dry-run** と、長時間ジョブ向けの
**進捗ログ** を追加する。運用者が「何本のアーカイブになるか」「どのフォルダが分割されるか」
を事前確認できるようにする。

## User story

As an operator packing a multi-GB tree to USB,  
I want a dry-run that prints the planned archives and an optional progress log,  
so that I can validate unit sizing before spending time and disk wear.

## Acceptance Notes

### Must have

1. `pack --dry-run`（または `--plan-only`）:
   - source を scan + plan する
   - **output に archive ファイルを作らない**
   - mapping を書く場合は Adjudicator 方針に従う（推奨: dry-run では mapping も書かない、stdout に要約）
2. dry-run 出力に少なくとも:
   - budget bytes
   - unit 数
   - 各 unit の placement path と logical bytes、member 数
3. `pack --verbose`（名称可）で unit 完了ごとに 1 行ログ（stderr 推奨）。
4. テスト: dry-run 前後で output ディレクトリに新規 archive が無いこと。
5. 仕様 CLI 節を更新。

### Explicit non-goals

- リッチ TUI プログレスバー、ETA の高精度推定。
- restore の dry-run（必要なら別 issue）。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0002
- Blocks: none
- Related: LISS-0009（並列時も進捗が意味を持つ）

## Adjudicator Decision Points

- [ ] dry-run で mapping JSON を書くか（推奨: 書かない）
- [ ] 進捗の出力先: stderr vs stdout
- [ ] Phase 1 Red 承認

## Context

- Included: CLI pack, application pack orchestration
- Omitted: machine-readable JSON plan format (future optional)
- Assumptions: plan_pack already pure; dry-run stops before engine.create_members

## AI Planning Records

### AIP-0008-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path
- Intended scope: dry-run + verbose progress only
- Estimated token range: 40k–90k
- Estimated token midpoint: 60k
- Token metric: rough session tokens
- Estimation basis: CLI + application branch + integration test
- Assumptions: no schema change required
- Confidence: high
- Revises:
- Revision reason:
- Superseded by:

## References

- `docs/specs/media-fit-pack.md` CLI section
- clap derive patterns for flags

## Work Notes

### Implementation sketch

```text
plan = scan + plan_pack
if dry_run { print plan; return }
for unit in units {
  create archive
  if verbose { log }
}
write mapping
```

## Verification

- [ ] Integration test: dry-run creates no zip/tar under output
- [ ] Verbose smoke on small tree
- [ ] `--help` にフラグ説明
