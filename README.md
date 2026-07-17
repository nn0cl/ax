# archiver (`ax`)

[日本語](README.ja.md)

Local-first CLI that **packs** directory trees into media-sized archives (scattered
by folder hierarchy, with a JSON mapping for restore), and also supports
single-archive create / list / extract.

**Default binary: `ax`** (Archive eXchange).  
`cargo build` / `cargo run` / `cargo install` produce **`ax`** by default (not `archiver`).  
Optional second binary: `cargo build --features compat-bin` → also builds `archiver`.  
Install script links `archiver` → `ax` for compatibility.

| Short | Alias | Role |
|-------|--------|------|
| `cr` | create | single archive |
| `ex` | extract | extract one archive |
| `ls` | list | list members |
| `es` | estimate | unit size estimate |
| `pa` | pack | media-fit pack |
| `re` | restore | restore from map |
| `ve` | verify | verify map/archives |

Globals: `-c` config · `-s` budget size/preset · `-o` output/map path. Pack source: `-i`/`--source`.

**Size presets** (`-s`): `c2`=10MiB · `fat32`=3.8GiB · `flash`=1GiB · `max`=no practical split  
(or numeric `10M` / `4G`; `low-net` is an alias of `c2`).

```bash
ax --help          # overview + presets + examples
ax pa --help       # pack options
ax -h              # short help
```

Supported formats: **ZIP**, **TAR**, **TAR.GZ**, **TAR.ZST** (zstd, multi-threaded).

> For a fuller guide in Japanese (recommended if you work in Japanese), see
> **[README.ja.md](README.ja.md)**.

## Disclaimer (no warranty)

**Support tiers** (see [ADR 0019](docs/architecture/adr/0019-supported-platforms-and-verification-scope.md)):
Supported = macOS/Linux primary path; Best-effort = USB / verify_stage / estimates;
Unsupported = Windows certification, concurrent pack to same output.
Product ADRs: [docs/architecture/adr/README.md](docs/architecture/adr/README.md).
Manual QA: [docs/QA.md](docs/QA.md).

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
# → ~/.local/bin/ax (+ archiver → ax symlink)
export PATH="$HOME/.local/bin:$PATH"
ax --version
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
install -m 755 target/release/ax ~/.local/bin/ax
ln -sf ax ~/.local/bin/archiver
```

## Build

```bash
# Default: builds ax only → target/release/ax
cargo build --release
cargo run -- --help          # runs ax (default-run)
cargo run -- pa --help

# Optional: also build a binary named archiver
cargo build --release --features compat-bin
# → target/release/ax and target/release/archiver

# Install into PATH
cargo install --path . --locked
# → ~/.cargo/bin/ax
# ln -s ax ~/.cargo/bin/archiver   # optional alias
```

## Usage

### Pack / restore (primary)

```bash
# Estimate a fast-transfer unit size for a USB stick
ax es -p /Volumes/USB --connection usb3
ax es -p /Volumes/USB --measure

# Pack (short): -i source, -o out, -s budget
# Live dashboard on a real TTY: total / cmp0..N / trn (ADR 0022/0024)
ax pa -i ./photos -o /Volumes/USB/backup --auto-size --connection usb3 -f tar.zst -j 4

# Plan only
ax pa -i ./photos -o ./out --auto-size --dry-run

# Explicit size / media preset
ax pa -i ./data -o ./out -s 700M -f zip
ax pa -i ./data -o ./out --media dvd --connection usb3

# Optional stage integrity
ax pa -i ./data -o ./out -s 700M --verify-stage --work-dir /tmp/aw

# Settings file; CLI overrides — see examples/archiver.inf
ax -c ./archiver.inf pa
ax -c ./archiver.inf pa -j 8 --verify-stage

# Restore (SHA-256 verify on by default)
ax re -m /Volumes/USB/backup/archiver-map.json -d ./restored
# or: ax -o map.json re -d ./restored
```

### Single archive (low-level)

```bash
ax -o backup.zip cr ./src README.md
ax ls backup.zip
ax ex -d ./restored backup.zip
```

### Commands

| Command | Description |
|---------|-------------|
| `es` / `estimate -p <path> …` | Recommend pack unit size for fast transfer |
| `pa` / `pack -i <dir> -o <dir> (-s SIZE\|--auto-size\|--media) …` | Media-fit multi-archive pack + JSON map |
| `re` / `restore -m <map.json> -d <dir> …` | Restore original tree |
| `ve` / `verify -m <map.json> …` | Verify map/archives |
| `cr` / `create -o <path> [-f] <inputs...>` | One archive |
| `ls` / `list [-f] <archive>` | List members |
| `ex` / `extract -d <dir> [-f] <archive>` | Extract one archive |

**Transfer estimate:** `unit ≈ practical_write_throughput(connection) × target_secs`,
clamped to 4MiB–4GiB and free space.

**Pack progress (TTY):** fixed-height dashboard (`total` + `cmp0..N` + `trn`), UI
thread only; 80-col design baseline, optional wide layout up to 200 cols; file
labels are `name·tag` (stable path hash). See ADR 0022 / 0024 and [README.ja.md](README.ja.md)
for field meanings. No TTY → no dashboard (logs only; `-vv` for detail).

**CI:** `cargo test` + `cargo clippy -D warnings` on Ubuntu (see `.github/workflows/ci.yml`).

Path traversal members are **rejected** on extract.

## Development

- Spec (pack): [`docs/specs/media-fit-pack.md`](docs/specs/media-fit-pack.md)
- Acceptance matrix: [`docs/specs/media-fit-acceptance.md`](docs/specs/media-fit-acceptance.md)
- Spec (single): [`docs/specs/archive-cli.md`](docs/specs/archive-cli.md)
- Architecture: [`docs/architecture/rust-cli-architecture.md`](docs/architecture/rust-cli-architecture.md)
- Product design book: [`docs/architecture/ax-product-design-book.md`](docs/architecture/ax-product-design-book.md)
- Progress UI: [ADR 0022](docs/architecture/adr/0022-progress-dashboard-ui-thread.md), [ADR 0024](docs/architecture/adr/0024-cli-screen-geometry.md), [LISS-0011](docs/issues/LISS-0011-progress-dashboard-ui-thread.md)
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
