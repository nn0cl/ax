# archiver

Local-first CLI that **packs** directory trees into media-sized archives (scattered
by folder hierarchy, with a JSON mapping for restore), and also supports
single-archive create / list / extract.

Supported formats: **ZIP**, **TAR**, **TAR.GZ**, **TAR.ZST** (zstd, multi-threaded).

## Pack rules (accepted)

1. Prefer **one archive per folder** (immediate files only).
2. If a folder is too large for the budget, group by **extension**, then size-bin.
3. A single file larger than the budget is still **one archive** (not byte-split).
4. Empty directories are recorded in the mapping and recreated on restore.
5. Budget = media capacity × connection factor (or `--target-size`).

## Build

```bash
cargo build --release
```

## Usage

### Pack / restore (primary)

```bash
# Estimate a fast-transfer unit size for a USB stick (probe free space + link class)
archiver estimate --path /Volumes/USB --connection usb3
# → recommended_unit based on ~throughput × 45s, capped by free space

# Pack using that estimate automatically
archiver pack \
  --source ./photos \
  --output /Volumes/USB/backup \
  --auto-size \
  --connection usb3 \
  --format tar.zst

# DVD capacity budget (legacy capacity mode)
archiver pack \
  --source ./photos \
  --output /Volumes/USB/backup \
  --media dvd \
  --connection usb3 \
  --format tar.zst

# Explicit size
archiver pack -s ./data -o ./out --target-size 700M --format zip

# Restore
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

**Transfer estimate formula:** `unit ≈ practical_write_throughput(connection) × target_secs`, clamped to 4MiB–4GiB and free space.

**CI:** `cargo test` + `cargo clippy -D warnings` on Ubuntu (see `.github/workflows/ci.yml`).

Path traversal members are **rejected** on extract.

## Development

- Spec (pack): [`docs/specs/media-fit-pack.md`](docs/specs/media-fit-pack.md)
- Spec (single): [`docs/specs/archive-cli.md`](docs/specs/archive-cli.md)
- Architecture: [`docs/architecture/rust-cli-architecture.md`](docs/architecture/rust-cli-architecture.md)

```bash
cargo test
```

## Layout

```text
src/
  core/domain/       # pack planner, media budget, mapping types, path safety
  core/application/  # pack / restore / create / list / extract
  core/ports/        # ArchiveEngine, FileScanner, MappingStore
  adapters/          # zip/tar/zstd, FS scan, JSON mapping
  delivery/          # clap CLI
```

## License

MIT (project-specific license may be added later).
