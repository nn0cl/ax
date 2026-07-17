# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Optional pack integrity: `--verify-stage` / config `verify_stage=true`
  - Pre-transfer: extract staged archive and re-check member SHA-256
  - Post-transfer: dest archive SHA-256 must equal stage archive SHA-256
- Settings file: global `--config PATH` (`.inf` / `.ini`); `[pack]` section; CLI overrides file
- Example: `examples/archiver.inf`
- Format-aware compressed size **estimate** (algorithm prior):
  - `tar` ≈ 1.02, `zip` ≈ 0.70, `tar.gz` ≈ 0.65, `tar.zst` ≈ 0.55
  - Used for progress write % / ETA until first transfer; then empirical ratio
  - Logged as `size_estimate:` / resource informational line (free-space gate still logical worst-case)
- Content-class + algorithm estimate: extension → class (text/source/image/video/…)
  × format efficiency (`ratio ≈ 1 - (1 - base_zstd) * eff`); class breakdown in `size_estimate:`
- Product ADRs **0014–0019** (quality/versioning, strict integrity, verify_stage,
  config precedence, content estimates, support tiers) + `docs/architecture/adr/README.md`
- Manual QA checklist: `docs/QA.md`

### Removed / breaking

- No legacy resume: plan **v2** only; Done units require **archive_sha256** (size-only skip removed)
- `--resume` without plan file is an **error** (no soft “start fresh”)
- Mapping members require non-empty **sha256** (no serde default empty)
- `--measure` failure no longer falls back to table throughput
- Mapping/plan version mismatch is a hard error

## [0.0.1] - 2026-07-17

Initial product-oriented release of the media-fit archiver.

### Added

- Commands: `pack`, `restore`, `estimate`, `verify`, `create`, `list`, `extract`
- Formats: zip, tar, tar.gz, tar.zst (zstd multi-thread)
- Resume (`archiver-plan.json`), work-dir staging, resource preflight
- Pack-time member SHA-256 + archive SHA-256 for resume integrity
- Over-budget same-directory split by whole files (no byte-split of single file)
- Progress UI with wall ETA (`max(in, xfer)`), Ctrl+C cooperative + 2nd force exit
- macOS/Linux install (`docs/INSTALL.md`, `scripts/install.sh`)
- Release process (`docs/RELEASE.md`, `scripts/release.sh`, GitHub Actions release)
- Dependency version table in `docs/INSTALL.md` (Cargo.toml ranges + Cargo.lock pins)
- Author: nn0cl \<nn0cl@dstechnology.co.jp\>; All rights reserved
- Disclaimer: no warranty, no SLA, use at own risk

### Defaults

- pack `--format tar.zst`
- pack `--jobs` = CPU count (max 8)

[Unreleased]: https://github.com/nn0cl/archiver/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/nn0cl/archiver/releases/tag/v0.0.1
