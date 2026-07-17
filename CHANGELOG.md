# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Product meaning of 0.x versions: [ADR 0014](docs/architecture/adr/0014-product-quality-versioning-and-support.md).

## [Unreleased]

### Added

### Changed

### Fixed

## [0.1.0] - 2026-07-17

**Usable baseline** (ADR 0014): documented primary path is intentional —
pack / restore / verify / estimate, plan v2, optional config and integrity
options. Still **AS IS**, no SLA. Breaking changes from here require CHANGELOG
notes and SemVer bumps per [RELEASE.md](docs/RELEASE.md).

### Added

#### CLI surface (ADR 0020, 0021)
- Primary binary **`ax`** (Archive eXchange); optional `archiver` via `compat-bin`
- Short subcommands: `cr` `ex` `ls` `es` `pa` `re` `ve` (long names as aliases)
- Globals: `-c` config, `-s`/`--size` budget or preset, `-o`/`--output`
- Pack source: `-i`/`--source`
- Size presets: `c2` (10MiB; alias `low-net`), `fat32` (3.8GiB), `flash` (1GiB),
  `max` (practically no split)

#### Pack integrity & config (ADR 0015–0017)
- Optional `--verify-stage` / `verify_stage=true` (pre-transfer member SHA-256,
  post-transfer archive SHA-256)
- Settings file: `--config` / `.inf` / `.ini` `[pack]`; CLI overrides file
- Example: `examples/archiver.inf`

#### Size estimate (ADR 0018)
- Format prior ratios for write % / ETA until first transfer, then empirical
- Content-class × algorithm estimate; class breakdown at `-vv`

#### Progress UI (ADR 0022–0024, LISS-0011)
- Fixed-height dashboard: `total` + `cmp0..N` + `trn` (UI thread only)
- 80-column design baseline; wide terminals 120–200 cols
- Fixed-width right-aligned numerics; full `elapsed` / `ETA` on total (no path)
- File labels `name·tag` (stable path hash); dual `archive | source` when wide
- Quiet text logs by default; detail at `-vv` (ADR 0023)

#### Parallelism & resources (ADR 0025–0026, LISS-0015/0016)
- `--at` / `--auto-jobs`: plan-time jobs from CPU × units × files × I/O class
  (`--connection` / throughput / work-dir); ceiling 32; no mid-run resize
- `--max-use SIZE`: unit budget cap; serial compress → transfer → delete stage
- Exclusive pack locks: `{output}/.ax-pack.lock` (+ work-dir); concurrent pack fails busy

#### Docs & quality gates
- Product ADRs **0014–0026**, design book, LISS issues, manual [QA.md](docs/QA.md)
- CI + optional pre-push: `cargo clippy --all-targets -- -D warnings`
  (`scripts/install-git-hooks.sh`)

### Changed

- Default pack format remains `tar.zst`; default jobs `min(CPU, 8)` without `--at`
- Progress worker rows named `cmp*` / `trn` (not long `compress*` / `transfer`)
- Free-space hard gate remains **logical × safety** (compressed estimate is info only)

### Fixed

- Progress whole-row middle-truncate garble (`…TA` / metrics corruption)
- Invalid CI job-level `hashFiles()` (workflow would not start)
- Concurrent pack no longer policy-only: runtime lock enforced

### Removed / breaking (vs early 0.0.x experiments)

- No legacy resume: plan **v2** only; Done units require **archive_sha256**
- `--resume` without plan file is an **error**
- Mapping members require non-empty **sha256**
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

[Unreleased]: https://github.com/nn0cl/ax/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nn0cl/ax/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/nn0cl/ax/releases/tag/v0.0.1
