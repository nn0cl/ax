# LISS-0003: CI に Rust test / clippy ジョブを追加する

## Metadata

- Local issue ID: LISS-0003
- GitHub issue: n/a
- Status: done
- Phase: complete (blanket Adjudicator approval 2026-07-17)
- Type: chore / infrastructure
- Priority: P0
- Initial planning size: S
- Current planning size: S
- Reclassification reason:
- Owner/agent: unassigned
- Related branch: `chore/ci-rust`
- Work plan: WP-0002 Slice S1

## Summary

GitHub Actions に **Rust の決定的検証**（`cargo test`, `cargo clippy -D warnings`）を追加し、
PR マージ前に media-fit / archive CLI の退行を自動検出する。

既存の template `repository-sanity` ジョブは残す。本 issue は **追加ジョブ**のみ。

## User story

As a maintainer,  
I want every PR to run unit and integration tests plus clippy,  
so that broken pack/restore behavior cannot merge silently.

## Acceptance Notes

### Must have

1. `.github/workflows/ci.yml`（または専用 workflow）に `rust` ジョブがある。
2. ジョブは少なくとも次を実行する:
   - `cargo test`
   - `cargo clippy --all-targets -- -D warnings`
3. `ubuntu-latest` で安定動作する（zstd ネイティブ依存を含む）。
4. `Cargo.lock` をコミットした状態で CI が再現可能（binary アプリ方針を維持）。
5. README または architecture に「CI が何を保証するか」1 段落がある。

### Explicit non-goals

- coverage レポート、release バイナリ配布、クロスコンパイル matrix 全 OS。
- テンプレート document sanity の削除。

## Dependencies

- Parent: WP-0002
- Depends on: LISS-0001 (codebase exists)
- Blocks: none (enables confidence for all later issues)
- Related: LISS-0002

## Adjudicator Decision Points

- [ ] OS matrix: ubuntu only (推奨) vs ubuntu+macos
- [ ] clippy を `-D warnings` で必須にするか（推奨: 必須）

## Context

- Included: `.github/workflows/ci.yml`, `Cargo.toml`, native `zstd-sys`
- Omitted: packaging, caching 高度最適化（sccache 等は任意）
- Assumptions: GitHub-hosted runner で `pkg-config` / ビルドツールは標準で足りる

## AI Planning Records

S サイズのため任意。2 回目の試行時は必須化。

## References

- https://doc.rust-lang.org/cargo/guide/continuous-integration.html
- https://github.com/actions-rust-lang/setup-rust-toolchain (候補; 採用時はバージョン pin)

## Work Notes

- Prefer official `dtolnay/rust-toolchain` or `actions-rust-lang/setup-rust-toolchain` with pinned action major version.
- Cache `~/.cargo` + `target` if wall time is painful; not required for S.

## Verification

- [ ] PR 上で rust ジョブが緑
- [ ] 意図的にテストを壊すとジョブが赤になることを一度確認（ローカルまたは draft PR）
