# Design Note: Media-Fit Pack

## Design Note

- **Target behavior**: 1フォルダ1アーカイブ優先、過大時は拡張子分割、メディア/接続で予算調整、JSON マッピングで restore。
- **Requested phase**: 実装完了（仕様 accepted）
- **Proposed next phase**: 運用フィードバック / 圧縮後サイズ厳密 packing は ADR 0013
- **Product ADRs**: `docs/architecture/adr/README.md`（0013–0019 要約）
- **Adjudicator decision needed**: なし（2026-07-17 確定）

## Context Ledger

- **Included**: ユーザー要件4点、現行 Clean Architecture、既存 `ArchiveEngine`
- **Omitted**: 暗号化、複数物理ボリューム自動排出、GUI、クラウド
- **Assumptions**: local-first CLI; 初回は logical size 予算; JSON マッピング
- **Open decisions**: `docs/specs/media-fit-pack.md` の Open Decisions 参照

## AI Payload Plan

- **Include**: 本設計、仕様草案、既存 ports / domain
- **Exclude**: zip/tar クレート内部、無関係な collaboration ADR 全文
- **Routing**: 設計は reasoning; 実装時の Planner は単体テスト可能で deterministic
- **Deterministic checks**: Planner の表形式テスト、restore round-trip

## Contract Notes

- **Input**: source dir, output dir, media budget, format, thresholds
- **Output**: N archives + mapping manifest; restore → original tree
- **Reasoning evidence**: 分割結果は `(files, budget) → units` の純関数で説明可能

## Stop Conditions

- Open Decisions 未決のまま Phase 1 に入らない
- ファイルバイト分割が必要になったら ADR
- 「メディア空き容量自動検出」を入れるなら port + ADR
