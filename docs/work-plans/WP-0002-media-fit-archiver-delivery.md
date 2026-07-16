# Work Plan: Media-Fit Archiver Delivery & Hardening

## Goal

- ローカルファーストな **メディア適合アーカイバ** を、仕様・テスト・運用品質の面で
  レビュー可能な単位（スライス）に分割して完成させる。
- 既に greefield 実装済みの骨格（pack / restore / estimate / 低レベル create）を
  **受け入れ可能な成果物** に引き上げ、残課題を依存関係付きで消化する。

## Scope

### In

- 仕様 `docs/specs/media-fit-pack.md` / `archive-cli.md` の受け入れ整合
- CI・Clean Architecture 負債の解消
- マッピング完全性、接続 probe 精度、運用 CLI（dry-run / 進捗）
- 独立ユニットの並列圧縮（性能）
- 圧縮後サイズモードは **ADR スパイクまで**（本 WP で実装コミットしない）

### Out

- 暗号化アーカイブ
- 複数物理メディアへの自動ボリュームまたぎ
- ファイル内容のバイト分割
- GUI / TUI
- クラウド同期・リモートバックアッププロトコル

## As-Built Snapshot (2026-07-17)

| Capability | Status | Spec | Notes |
| --- | --- | --- | --- |
| create / list / extract | shipped (code) | archive-cli.md | LISS-0001 |
| pack (folder → archive, ext split) | shipped (code) | media-fit-pack.md | 要受け入れ freeze |
| restore + empty dirs | shipped (code) | media-fit-pack.md | |
| estimate / pack --auto-size | shipped (code) | media-fit-pack.md | 接続は主に手動 |
| zip / tar.gz / tar.zst | shipped (code) | media-fit-pack.md | |
| SHA-256 in mapping | not done | — | LISS-0005 |
| USB auto-detect | partial | — | path heuristic only |
| CI rust job | not done | — | LISS-0003 |
| dry-run / progress | not done | — | LISS-0008 |

## Issue Graph

| Issue | Status | Initial size | Current size | Planning record | Depends on | Blocks | Branch |
| --- | --- | --- | --- | --- | --- | --- | --- |
| LISS-0001 | done | M | M | AIP-0001-001 | — | 0002 | feature/archive-cli-mvp |
| LISS-0002 | done | M | M | AIP-0002-001 | 0001 | 0005,0008 | feature/spec-freeze-media-fit |
| LISS-0003 | done | S | S | (optional) | 0001 | — | chore/ci-rust |
| LISS-0004 | done | M | M | AIP-0004-001 | 0002 | 0009 | refactor/fs-ports-pack |
| LISS-0005 | done | M | M | AIP-0005-001 | 0002 | — | feature/mapping-integrity |
| LISS-0006 | done | M | M | AIP-0006-001 | 0002 | 0007 | feature/usb-probe |
| LISS-0007 | done | M | M | AIP-0007-001 | 0006 | — | feature/write-benchmark |
| LISS-0008 | done | M | M | AIP-0008-001 | 0002 | — | feature/pack-dry-run-progress |
| LISS-0009 | done | M | M | AIP-0009-001 | 0004 | — | feature/parallel-pack |
| LISS-0010 | done | L | L | AIP-0010-001 | 0002 | — | docs/adr-compressed-size |

```text
0001 (done)
  └─► 0002 Spec freeze ──┬──► 0005 Mapping integrity
                         ├──► 0006 USB probe ──► 0007 Write benchmark
                         ├──► 0008 Dry-run / progress
                         └──► 0010 ADR compressed-size
  └─► 0003 CI (parallel, low coupling)
0002 ──► 0004 FS ports ──► 0009 Parallel pack
```

## Delivery Slices

各スライスは **1 本の feature/chore ブランチ + 1 以上の LISS** を基本単位とする。
Phase は Feature Path（Red → Green → Refactor）に従う。スライス完了ごとに
main へ PR 可能な状態にする（長期ブランチ禁止）。

| Slice | Theme | Issues | Outcome | Exit criteria |
| --- | --- | --- | --- | --- |
| **S0** | Baseline close-out | 0001 | 低レベル CLI を done として確定 | 既存 `cargo test` 緑 |
| **S1** | Quality floor | 0002, 0003 | 仕様・受け入れ表の freeze + CI | CI 必須ジョブ緑、受け入れ表レビュー済 |
| **S2** | Architecture debt | 0004 | application から直接 FS I/O を除去 | clippy + 層テスト方針遵守 |
| **S3** | Trust | 0005 | マッピングにハッシュ、restore 検証 | 改ざん/破損を検出するテスト |
| **S4** | Transfer intelligence | 0006 → 0007 | 接続自動判定 → 任意の書き込み実測 | estimate が USB クラスを説明可能 |
| **S5** | Operator UX | 0008 | dry-run と進捗 | dry-run でディスク不変 |
| **S6** | Throughput | 0009 | ユニット並列 pack | 回帰なし、並列時の順序/map 一貫 |
| **S7** | Future gate | 0010 | 圧縮後サイズモード ADR のみ | Adjudicator が go/no-go |

### Slice dependency (execution order)

```text
S0 → S1 → S2 → S3
          ↘ S4 (can start after S1)
          ↘ S5 (can start after S1)
     S2 → S6
     S1 → S7 (docs only; anytime after S1)
```

S3 / S4 / S5 は S1 完了後に **並列可能**（衝突ファイルに注意）。

## Recommended Order

1. **S1** LISS-0002 → LISS-0003（品質床。以降の PR の前提）
2. **S2** LISS-0004（並列・拡張の前に境界を直す）
3. **S3** LISS-0005（復元の信頼）
4. **S4** LISS-0006 → LISS-0007（転送見積の精度）
5. **S5** LISS-0008（運用）
6. **S6** LISS-0009（性能）
7. **S7** LISS-0010（方針決定のみ）

## Current Next Issue

- Issue: none (WP-0002 slices S0–S7 implemented)
- Reason it is unblocked: LISS-0001 done; media-fit コードは存在するが受け入れ表が未 freeze
- Adjudicator approval needed: S1 開始承認（Phase 1 Red: 受け入れ表と不足テストの追加のみ）

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| As-built と仕様のズレ | 受け入れ不能 | S1 で差分を表にし、仕様か実装を明示的に合わせる |
| macOS USB 検出の脆さ | 誤った estimate | 手動 `--connection` を常に優先、自動は best-effort |
| 並列 pack の map 順序 | 非決定的 ID | unit を安定ソートしてから ID 採番 |
| 圧縮後サイズモードの早期実装 | 複雑爆発 | S7 で ADR のみ、実装は別 WP |
| 巨大ツリーのメモリ | OOM | 一覧はメタデータのみ保持（既存方針をテストで固定） |

## Verification Plan

- 全スライス: `cargo test` / `cargo clippy --all-targets -- -D warnings`
- S1 以降: GitHub Actions の rust job（LISS-0003）
- S3: 意図的に壊した member / ハッシュ不一致の restore 失敗
- S4: fixture またはモック probe で接続クラス分岐を単体検証
- S5: dry-run 前後の output ツリー diff が空
- S6: 同一入力で map の logical 内容が逐次と一致（順序ポリシー文書化）

## Adjudicator Checkpoints

| Gate | When | Decision |
| --- | --- | --- |
| G1 | S1 start | 受け入れ freeze の範囲承認 |
| G2 | S3 Phase 1 | ハッシュ必須 vs オプション |
| G3 | S4 Phase 1 | 自動検出失敗時のフォールバック |
| G4 | S6 Phase 1 | 並列度の既定値 |
| G5 | S7 | 圧縮後サイズモードを次 WP に載せるか |

## AI Planning Records (plan-level)

### AIP-WP-0002-001

- Status: proposed
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: XL（複数 issue・複数スライス）
- Intended execution route: Feature Path per issue; Architecture Path for LISS-0010
- Intended scope: delivery & hardening of media-fit archiver after as-built MVP
- Estimated token range: N/A（計画文書のみ）
- Estimated token midpoint: N/A
- Token metric: N/A
- Estimation basis: issue graph + as-built inventory
- Assumptions: Adjudicator はスライス単位で Phase 承認する
- Confidence: medium-high
- Revises:
- Revision reason:
- Superseded by:
