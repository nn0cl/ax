# LISS-0002: Media-fit 仕様・受け入れ freeze

## Metadata

- Local issue ID: LISS-0002
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature / quality (spec + tests)
- Priority: P0
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/spec-freeze-media-fit`
- Work plan: WP-0002 Slice S1

## Summary

greenfield 実装済みの media-fit pack / restore / estimate について、
**受け入れ仕様とテスト行列を freeze** し、as-built と `docs/specs/media-fit-pack.md`
の差分を解消する。以降の issue はこの freeze を契約とする。

本 issue は新機能の追加を目的としない。仕様の明確化・不足テスト・文書整合のみ。

## User story

As an Adjudicator reviewing the archiver product,  
I want a single accepted behavior contract and a mapped automated test matrix,  
so that later PRs can be judged against explicit Given/When/Then rather than tribal knowledge.

## Acceptance Notes

### Must have

1. `docs/specs/media-fit-pack.md` が as-built と一致する（差分があれば仕様更新 or 欠陥 issue 起票）。
2. 受け入れ表（本 issue 内または `docs/specs/media-fit-acceptance.md`）に、各 Gherkin シナリオと
   対応テスト（unit / integration ファイル・テスト名）が 1:1 で載る。
3. 次の振る舞いが **自動化テストで固定** されていること:
   - 1 フォルダ・予算内 → 1 アーカイブ
   - フォルダ過大 → 拡張子分割
   - 巨大単一ファイル → 単独アーカイブ（分割なし）
   - 空ディレクトリ → mapping + restore
   - estimate / `--auto-size` が budget > 0 を返す
   - restore round-trip でバイト一致
4. `docs/architecture/rust-cli-architecture.md` の CLI 面が estimate / auto-size を含む。
5. Out of scope が仕様に明示され、誤って「未実装バグ」扱いにならない。

### Nice to have

- 失敗系（budget 未指定、mapping 欠落、archive 欠落）の表追加。

### Explicit non-goals

- 新フォーマット、並列 pack、ハッシュ、USB 自動検出の実装（後続 LISS）。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0001 (done)
- Blocks: LISS-0005, LISS-0006, LISS-0008, LISS-0010（契約前提）
- Related: LISS-0003 (CI で本 issue のテストを常時実行)

## Adjudicator Decision Points

- [ ] as-built と仕様が食い違う場合、**仕様を正**とするか **実装を直す issue** を切るか
- [ ] 受け入れ表の置き場: 本 issue 内 vs 独立 `docs/specs/media-fit-acceptance.md`
- [ ] Phase 1 Red 承認（不足テストの追加のみ）

## Context

- Included: media-fit-pack.md, archive-cli.md, tests/pack_restore.rs, tests/archive_cli.rs, domain pack_plan tests
- Omitted: performance tuning, platform USB IOKit
- Assumptions: 現行 `cargo test` は緑; freeze はリグレッション防止が主目的

## AI Planning Records

### AIP-0002-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path Phase 1 (tests/docs) → Phase 2 only if spec/impl gap requires code fix
- Intended scope: acceptance matrix + spec alignment + missing tests only
- Estimated token range: 30k–80k
- Estimated token midpoint: 50k
- Token metric: rough session tokens (planning estimate, not billing)
- Estimation basis: multi-file docs + test inventory across pack/restore/estimate
- Assumptions: large gaps are rare; most work is documentation + 1–3 tests
- Confidence: medium
- Revises:
- Revision reason:
- Superseded by:

## References

- `docs/specs/media-fit-pack.md`
- `docs/specs/archive-cli.md`
- `docs/architecture/testing-strategy.md`
- `docs/collaboration/local-issue-planning.md`
- `docs/at-tdd/process.md`

## Work Notes

### Suggested Phase 1 checklist

- [ ] Inventory: list every public CLI flag and domain rule
- [ ] Diff table: Spec row ↔ Test name ↔ Pass/Fail/Missing
- [ ] Add missing automated tests only (no refactors)
- [ ] Update spec "Ambiguities / Out of Scope" if needed

### Suggested Phase 2 (only if gap)

- Minimal code fix to match accepted spec; do not expand scope

## Verification

- [ ] Acceptance matrix reviewed by Adjudicator
- [ ] `cargo test` green
- [ ] No unexplained Spec↔Code gap remains (or tracked as new LISS)
