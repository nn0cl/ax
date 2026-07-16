# LISS-0010: ADR — 圧縮後サイズを予算に使うパッキングモード

## Metadata

- Local issue ID: LISS-0010
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: architecture / spike
- Priority: P3
- Initial planning size: L
- Current planning size: L
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `docs/adr-compressed-size`
- Work plan: WP-0002 Slice S7

## Summary

論理サイズ予算では「圧縮後の実アーカイブ」がメディア制約を破る／余りすぎる
問題がある。本 issue は **実装ではなく ADR** により、圧縮後サイズモードの
是非・方式・コストを決定する。

実装 WP は ADR 承認後に別途切る。

## User story

As an Adjudicator,  
I want a written decision on whether and how to pack by post-compression size,  
so that we do not accidentally commit to an expensive multi-pass design.

## Acceptance Notes

### Must have

1. `docs/architecture/adr/00xx-compressed-size-packing.md` がレビュー可能な状態で存在。
2. ADR が少なくとも次を比較する:
   - A. 現状 logical-only（維持）
   - B. 安全係数（logical × ratio）
   - C. 一時圧縮して実測し unit を再調整（multi-pass）
   - D. ストリーミングで書きながら超過時に unit を閉じる
3. 各案の: 正確性、CPU/ディスクコスト、実装複雑性、失敗モード。
4. **Decision** 節: 採用 / 却下 / 延期と理由。
5. 採用する場合の follow-up issue 草案（ID は未採番で可）。

### Explicit non-goals

- 本 issue ブランチでの pack アルゴリズム実装。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0002（現状仕様の理解）
- Blocks: future implementation issue (not yet filed)
- Related: transfer estimate discussion (compression vs copy speed)

## Adjudicator Decision Points

- [ ] ADR の決定（A/B/C/D またはハイブリッド）
- [ ] 延期する場合の再検討トリガ（例: ユーザーからメディア溢れ報告）

## Context

- Included: media-fit-pack size_basis, estimate philosophy
- Omitted: code changes beyond docs
- Assumptions: Architecture Path; full [THOUGHT] scaffold at execution

## AI Planning Records

### AIP-0010-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: L
- Intended execution route: Architecture Path
- Intended scope: ADR document only
- Estimated token range: 40k–90k
- Estimated token midpoint: 60k
- Token metric: rough session tokens
- Estimation basis: research + ADR writing
- Assumptions: no code in this issue
- Confidence: high
- Revises:
- Revision reason:
- Superseded by:

## References

- `docs/templates/adr.md`
- `docs/specs/media-fit-pack.md` Out of Scope: 圧縮後サイズ厳密保証
- prior design note `docs/architecture/media-fit-pack-design.md`

## Work Notes

### Trigger quotes from product discussion

- 論理サイズは決定的だが、メディア厳密適合には不足し得る。
- 圧縮ファイルはコピーが速くなり得るが、スループット自体は上がらない。

## Verification

- [ ] ADR merged or accepted by Adjudicator comment
- [ ] WP-0002 Slice S7 marked complete
- [ ] If approved for build: new LISS filed under new WP
