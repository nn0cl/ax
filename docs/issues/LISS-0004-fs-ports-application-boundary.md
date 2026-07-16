# LISS-0004: Pack/Restore の FS 境界を ports に閉じ込める

## Metadata

- Local issue ID: LISS-0004
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: refactor (architecture debt)
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `refactor/fs-ports-pack`
- Work plan: WP-0002 Slice S2

## Summary

`core::application` の pack / restore が直接 `std::fs` を呼んでいる箇所を整理し、
**ファイルシステム操作を ports + adapters に閉じ込める**。振る舞い（受け入れ）は
LISS-0002 freeze 後の仕様から変えない。

## User story

As a future contributor adding parallel pack or alternate storage backends,  
I want application use cases free of direct filesystem calls,  
so that behavior can be unit-tested with fakes and adapters stay swappable.

## Acceptance Notes

### Must have

1. `src/core/application/{pack,restore,estimate}.rs` が `std::fs` / `std::fs::*` を
   **直接 import / 呼び出さない**（grep で確認可能）。
2. 必要な操作は ports 経由:
   - ディレクトリ作成、存在確認、（必要なら）パス結合ポリシーは adapter か既存 scanner
3. 既存 integration tests（pack_restore, archive_cli）がすべて緑のまま。
4. ドメインの `plan_pack` / `estimate_transfer_unit` が引き続き I/O なし。
5. `docs/architecture/rust-cli-architecture.md` に「application は FS を直接触らない」を明記。

### Explicit non-goals

- 新機能、並列化（LISS-0009）、新 probe 実装（LISS-0006）。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0002（振る舞い契約が固まっていること）
- Blocks: LISS-0009（並列は境界がきれいな方が安全）
- Related: LISS-0001

## Adjudicator Decision Points

- [ ] `Path::exists` 相当を port に含めるか、scanner/probe 結果だけを信頼するか
- [ ] Phase 1: 「禁止 grep テスト」やアーキテクチャテストの置き場

## Context

- Included: application pack/restore, ports, adapters/fs
- Omitted: delivery/cli (clap は delivery のまま)
- Assumptions: リファクタのみ; 公開 CLI フラグ不変

## AI Planning Records

### AIP-0004-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path with characterization tests first, then move I/O
- Intended scope: boundary cleanup only
- Estimated token range: 40k–100k
- Estimated token midpoint: 70k
- Token metric: rough session tokens
- Estimation basis: multi-module refactor + regression suite
- Assumptions: no behavior change required by Adjudicator
- Confidence: medium
- Revises:
- Revision reason:
- Superseded by:

## References

- `docs/architecture/project-structure.md`
- `docs/architecture/rust-cli-architecture.md`
- Clean Architecture dependency rule in `AGENTS.md`

## Work Notes

### Characterization first

1. Ensure LISS-0002 tests cover pack/restore paths used today.
2. Introduce or extend `FileSystem` / use existing ports narrowly.
3. Move `create_dir_all` etc. to adapter helpers implementing the port.
4. Grep gate in tests or CI script snippet.

## Verification

- [ ] `rg 'std::fs' src/core/application` → no matches
- [ ] `cargo test` / clippy green
- [ ] Manual smoke: pack → restore one tree
