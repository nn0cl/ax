# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
