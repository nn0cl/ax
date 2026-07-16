# LISS-0006: 保存先の接続クラス自動検出（USB 等）

## Metadata

- Local issue ID: LISS-0006
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/usb-probe`
- Work plan: WP-0002 Slice S4

## Summary

`MediaProbe` を拡張し、出力先ボリュームの **接続クラス（usb2 / usb3 / local / network 等）** を
best-effort で推定する。`estimate` と `pack --auto-size` が、手動 `--connection` なしでも
妥当な転送見積を出せるようにする。

**手動 `--connection` は常に最優先**（自動検出を上書き）。

## User story

As a user packing to a USB stick,  
I want the tool to notice roughly whether the volume is USB2-class or faster,  
so that recommended archive unit sizes match transfer reality without memorizing flags.

## Acceptance Notes

### Must have

1. `MediaProbe::probe` が、hint 未指定時に `ConnectionKind` を推定して返す。
2. 推定根拠を `estimate` 出力に含める（例: `detection: path_heuristic|/Volumes` または `detection: platform_usb`）。
3. 優先順位（仕様に明記）:
   1. CLI `--connection`
   2. プラットフォーム固有検出（実装可能な範囲）
   3. パスヒューリスティック（現行の `/Volumes` 等）
   4. `local` フォールバック
4. 単体テスト: ヒューリスティックと、platform 層は **fake probe** で分岐を検証。
5. macOS では少なくとも次のいずれか:
   - 現状ヒューリスティックの文書化 + 改善、または
   - `diskutil` / IOKit 由来の Protocol 文字列の利用（取得失敗時はフォールバック）
6. 失敗しても pack は続行可能（estimate 不能で fatal にしない。free space 失敗は別ポリシー）。

### Should have

- Linux: `/sys` や mount options からのヒント（取れる範囲）。
- `bcdUSB` → `connection_from_bcd_usb` の接続（取得できる場合）。

### Explicit non-goals

- 書き込みベンチ（LISS-0007）。
- Windows 完全対応（別 issue 可; 本 issue は unix 中心で可）。
- 100% 正確な USB 世代判定の保証。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0002
- Blocks: LISS-0007（実測は検出結果の上に載せる）
- Related: domain `transfer_estimate.rs`

## Adjudicator Decision Points

- [ ] macOS 実装の許容手段: 純粋 Rust API vs `diskutil` サブプロセス
- [ ] network mount を誤って usb3 と eth する問題の扱い（`/Volumes` 曖昧性）
- [ ] Phase 1 Red 承認

## Context

- Included: adapters/fs/probe, estimate use case, CLI estimate output
- Omitted: throughput measurement
- Assumptions: best-effort; user override always wins

## AI Planning Records

### AIP-0006-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path; platform code behind port with fakes
- Intended scope: connection class detection only
- Estimated token range: 60k–140k
- Estimated token midpoint: 90k
- Token metric: rough session tokens
- Estimation basis: platform branching + tests with fakes
- Assumptions: diskutil subprocess acceptable if Adjudicator approves
- Confidence: medium
- Revises:
- Revision reason:
- Superseded by:

## References

- macOS `diskutil info` documentation (fetch-verify at implementation)
- domain `connection_from_bcd_usb`
- `docs/specs/media-fit-pack.md` transfer estimate section

## Work Notes

### Detection sketch

```text
if hint { return hint }
if let Some(c) = platform_detect(path) { return c }
if path_heuristic(path) { return Usb3 or Network }
return Local
```

### Risk: /Volumes ambiguity

Document that macOS network shares also appear under `/Volumes`; prefer platform protocol when available; else recommend `--connection network`.

## Verification

- [ ] Unit tests for priority order
- [ ] `estimate --path .` shows detection line
- [ ] With `--connection usb2`, detection does not override
