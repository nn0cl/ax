# LISS-0005: マッピング完全性（SHA-256）と restore 検証

## Metadata

- Local issue ID: LISS-0005
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: feature
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `feature/mapping-integrity`
- Work plan: WP-0002 Slice S3

## Summary

pack 時に各ソースファイル（および任意でアーカイブファイル）の **SHA-256** を
JSON マッピングに記録し、restore 時に検証できるようにする。転送・保管中の
破損や取り違えを検出する。

## User story

As a user restoring a backup from USB media,  
I want the tool to detect corrupted or substituted files,  
so that I do not silently restore wrong data.

## Acceptance Notes

### Must have

1. `MappingMember` に `sha256`（hex 小文字）フィールドを追加（schema version 方針を仕様に記載）。
2. pack は各 member のソース内容から sha256 を計算して mapping に書く。
3. restore は検証モード（少なくとも既定 ON、または `--verify` 既定 true）で:
   - 展開後ファイルの sha256 が mapping と一致しない場合 **非ゼロ終了**
   - 成功時は全 member 検証済み
4. 仕様 `media-fit-pack.md` の JSON 例と EARS/Gherkin を更新。
5. テスト:
   - round-trip で verify 成功
   - 展開後に 1 バイト改ざん → restore --verify 失敗（または pack 済み map を改ざん）

### Should have

- アーカイブファイル自体の sha256 を `MappingArchive` に記録。

### Explicit non-goals

- デジタル署名、暗号化、Merkle ツリー、リモート attestation。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0002
- Blocks: none
- Related: LISS-0008（進捗表示で hash 計算の時間が見える）

## Adjudicator Decision Points

- [ ] **ハッシュ必須**か、`--no-verify` で省略可能か（推奨: 記録は必須、restore 検証は `--verify` 既定 true）
- [ ] schema `version` を 2 に上げるか、v1 後方互換で optional フィールドにするか
- [ ] Phase 1 Red 承認

## Context

- Included: mapping types, pack, restore, JSON store, crypto hash crate choice
- Omitted: streaming progressive hash UI
- Assumptions: ローカル CPU で sha256 は許容; 巨大ファイルはストリームハッシュ

## AI Planning Records

### AIP-0005-001

- Status: done
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path; dependency check for `sha2` (or `blake3` if Adjudicator prefers — default sha2 for familiarity)
- Intended scope: hash record + restore verify
- Estimated token range: 50k–120k
- Estimated token midpoint: 80k
- Token metric: rough session tokens
- Estimation basis: schema + pack/restore paths + tests
- Assumptions: Adjudicator accepts sha256 hex in JSON
- Confidence: medium-high
- Revises:
- Revision reason:
- Superseded by:

## References

- https://docs.rs/sha2
- https://datatracker.ietf.org/doc/html/rfc6234 (SHA family overview)
- `docs/architecture/dependency-policy.md`（新規依存採用チェック）

## Work Notes

### Dependency adoption note (draft)

| Item | Note |
| --- | --- |
| Candidate | `sha2` + `hex` (or manual hex) |
| Layer | adapter or small domain-pure hasher via port `ContentHasher` |
| Boundary | Prefer `ContentHasher` port; adapter implements streaming hash |
| Vuln check | run at implementation time (`cargo deny` optional) |

### Phase sketch

1. Phase 1: failing tests for hash field + verify failure
2. Phase 2: port + sha2 adapter + pack/restore wiring
3. Phase 3: naming / avoid buffering whole file

## Verification

- [ ] `cargo test` includes integrity scenarios
- [ ] Manual: pack, flip one bit in restored file path before verify path, observe failure
- [ ] Spec JSON example updated
