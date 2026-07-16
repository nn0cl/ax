# LISS-0001: Archive CLI MVP（低レベル create/list/extract）

## Metadata

- Local issue ID: LISS-0001
- GitHub issue: n/a
- Status: done
- Phase: Phase 1–3 complete
- Type: feature
- Priority: P0
- Initial planning size: M
- Current planning size: M
- Reclassification reason:
- Owner/agent: grok
- Related branch: `feature/archive-cli-mvp`
- Work plan: WP-0002 Slice S0 (baseline)

## Summary

ローカルファーストな Rust CLI として、単一アーカイブの **create / list / extract** を
Clean Architecture（domain / application / ports / adapters / delivery）で実装した。

後続の media-fit pack（同一ブランチに as-built で同居）の土台。受け入れの正式 freeze は
LISS-0002 で media-fit を含めて行う。

## Acceptance Notes

- Spec: `docs/specs/archive-cli.md`
- Architecture: `docs/architecture/rust-cli-architecture.md`
- `cargo test` passes（低レベル + 後続テストを含むリポジトリ全体）
- Path traversal on extract is rejected
- Formats: zip, tar, tar.gz（後に tar.zst も追加）

## Dependencies

- Parent: none (founding issue)
- Depends on: template adoption
- Blocks: LISS-0002, LISS-0003
- Related: WP-0002

## Adjudicator Decision Points

- Resolved: Language Rust; full implement request treated as Phase 1–3 for founding MVP

## Context

- Included: create/list/extract, Clean Architecture skeleton
- Omitted (at issue open): media-fit pack (landed later as as-built; tracked via WP-0002)
- Assumptions: local filesystem only

## AI Planning Records

### AIP-0001-001

- Status: accepted
- Created by:
  - Agent/environment: Grok Build CLI
  - Model as displayed: Grok
  - Reasoning setting as displayed: default
  - N/A reason:
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: Feature Path end-to-end
- Intended scope: create/list/extract + tests + docs
- Estimated token range: N/A
- Estimated token midpoint: N/A
- Token metric: N/A
- Estimation basis: greenfield MVP
- Assumptions: clap + zip + tar + flate2 + tempfile
- Confidence: medium-high
- Revises:
- Revision reason:
- Superseded by:

## References

- https://docs.rs/clap
- https://docs.rs/zip
- https://docs.rs/tar
- https://docs.rs/flate2

## Work Notes

- Template copied from llm-project-template
- Scope later expanded on same branch to media-fit pack/estimate (see WP-0002 as-built table)

## Verification

- [x] `cargo test`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] Manual smoke: create → list → extract
