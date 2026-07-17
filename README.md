# archiver

[日本語](README.ja.md)

Local-first CLI that **packs** directory trees into media-sized archives (scattered
by folder hierarchy, with a JSON mapping for restore), and also supports
single-archive create / list / extract.

Supported formats: **ZIP**, **TAR**, **TAR.GZ**, **TAR.ZST** (zstd, multi-threaded).

> For a fuller guide in Japanese (recommended if you work in Japanese), see
> **[README.ja.md](README.ja.md)**.

## Disclaimer (no warranty)

This software is provided **AS IS**, without warranty of any kind. Automated tests
exist to reduce regressions; they do **not** guarantee correctness on every
environment or dataset. **No SLA.** Use at your own risk.

**Version: v0.0.1**

Author: nn0cl \<nn0cl@dstechnology.co.jp\>.  
Copyright (c) nn0cl@dstechnology.co.jp. All rights reserved.

Install & dependency versions: [docs/INSTALL.md](docs/INSTALL.md)  
Release process: [docs/RELEASE.md](docs/RELEASE.md) · Changelog: [CHANGELOG.md](CHANGELOG.md)

## Pack rules (accepted)

1. Prefer **one archive per folder** (immediate files only).
2. If a folder is too large for the budget, group by **extension**, then size-bin.
3. A single file larger than the budget is still **one archive** (not byte-split).
4. Empty directories are recorded in the mapping and recreated on restore.
5. Budget = media capacity × connection factor, explicit `--target-size`, or
   `--auto-size` transfer estimate.

## Install (macOS & Linux)

Full guide: **[docs/INSTALL.md](docs/INSTALL.md)**

```bash
# Requires Rust (rustup) + C toolchain
./scripts/install.sh
# → ~/.local/bin/archiver by default
export PATH="$HOME/.local/bin:$PATH"
archiver --version
```

| Method | Command |
|--------|---------|
| From this tree (release) | `./scripts/install.sh` |
| Custom prefix | `./scripts/install.sh --prefix /usr/local` |
| `cargo install` | `./scripts/install.sh --cargo-install` |
| GitHub Release binary | `REPO=owner/repo ./scripts/install.sh --from-release` |
| Uninstall | `./scripts/install.sh --uninstall` |

```bash
cargo build --release
install -m 755 target/release/archiver ~/.local/bin/archiver
```

## Build

```bash
cargo build --release
```

## Usage

### Pack / restore (primary)

```bash
# Estimate a fast-transfer unit size for a USB stick
archiver estimate --path /Volumes/USB --connection usb3
archiver estimate --path /Volumes/USB --measure

# Pack using that estimate
archiver pack \
  --source ./photos \
  --output /Volumes/USB/backup \
  --auto-size \
  --connection usb3 \
  --format tar.zst \
  --jobs 4 \
  -v

# Plan only
archiver pack -s ./photos -o ./out --auto-size --dry-run

# Explicit size / media preset
archiver pack -s ./data -o ./out --target-size 700M --format zip
archiver pack -s ./data -o ./out --media dvd --connection usb3

# Restore (SHA-256 verify on by default)
archiver restore \
  --mapping /Volumes/USB/backup/archiver-map.json \
  --dest ./restored
```

### Single archive (low-level)

```bash
archiver create -o backup.zip ./src README.md
archiver list backup.zip
archiver extract -d ./restored backup.zip
```

### Commands

| Command | Description |
|---------|-------------|
| `estimate -p <path> [--connection] [--target-secs] [--throughput] [--measure]` | Recommend pack unit size for fast transfer |
| `pack -s <dir> -o <dir> (--auto-size\|--target-size\|--media) [--dry-run] [-v] [--jobs N] ...` | Media-fit multi-archive pack + JSON map |
| `restore -m <map.json> -d <dir> [--archives-root] [--verify true\|false]` | Restore original tree (SHA-256 verify default on) |
| `create -o <path> [-f] <inputs...>` | One archive |
| `list [-f] <archive>` | List members |
| `extract -d <dir> [-f] <archive>` | Extract one archive |

**Transfer estimate:** `unit ≈ practical_write_throughput(connection) × target_secs`,
clamped to 4MiB–4GiB and free space.

**CI:** `cargo test` + `cargo clippy -D warnings` on Ubuntu (see `.github/workflows/ci.yml`).

Path traversal members are **rejected** on extract.

## Development

- Spec (pack): [`docs/specs/media-fit-pack.md`](docs/specs/media-fit-pack.md)
- Acceptance matrix: [`docs/specs/media-fit-acceptance.md`](docs/specs/media-fit-acceptance.md)
- Spec (single): [`docs/specs/archive-cli.md`](docs/specs/archive-cli.md)
- Architecture: [`docs/architecture/rust-cli-architecture.md`](docs/architecture/rust-cli-architecture.md)
- Agent contract: [`AGENTS.md`](AGENTS.md)

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## Layout

```text
src/
  core/domain/       # pack planner, media budget, mapping types, path safety
  core/application/  # pack / restore / estimate / create / list / extract
  core/ports/        # ArchiveEngine, FileScanner, FileSystem, ...
  adapters/          # zip/tar/zstd, FS scan, JSON mapping, measure
  delivery/          # clap CLI
```

## Out of scope (current)

- Byte-splitting a single huge file across archives
- Multi-volume spanning across physical media
- Encryption
- Strict post-compression size budgeting ([ADR 0013](docs/architecture/adr/0013-compressed-size-packing-mode.md))

## License

MIT (project-specific LICENSE file may be added later).
