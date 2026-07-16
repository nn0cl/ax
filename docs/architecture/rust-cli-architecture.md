# Rust CLI Architecture

## Scope

Rules for the Rust binary and library that implement the archiver CLI.

## Module Map

| Module | Responsibility |
|--------|----------------|
| `core::domain` | formats, path safety, media budget, pack planner, mapping types |
| `core::application` | pack / restore / create / list / extract use cases |
| `core::ports` | `ArchiveEngine`, `FileScanner`, `MappingStore` |
| `adapters::archive` | ZIP, TAR, TAR.GZ, TAR.ZST engines |
| `adapters::fs` | scan + pack entry helpers |
| `adapters::mapping` | JSON mapping I/O |
| `delivery` | clap CLI |

## Dependency Rule

```text
delivery → application → domain
                ↓
              ports
                ↑
            adapters
```

- Domain and application must not import `clap`, `zip`, `tar`, `flate2`, or `zstd`.
- Application must not call `std::fs` directly; use `FileSystem` / `ContentHasher` ports.
- Pack planning (`plan_pack`) is a pure function over file metadata.
- `main.rs` composes concrete adapters and calls delivery.

## Formats

| Format | Extension | Notes |
|--------|-----------|--------|
| ZIP | `.zip` | deflate; portable |
| TAR | `.tar` | uncompressed container |
| TAR.GZ | `.tar.gz` | single-thread gzip |
| TAR.ZST | `.tar.zst` | **zstd multi-threaded** (preferred for speed) |

## CLI Surface

```text
archiver pack --source <dir> --output <dir> (--target-size <SIZE> | --media cd|dvd|bd25) \
  [--connection local|usb2|usb3|thunderbolt|network] [--format zip|tar.gz|tar.zst] [--mapping <path>]

archiver restore --mapping <path> --dest <dir> [--archives-root <dir>]

archiver create --output <path> [--format ...] <inputs...>
archiver list <archive>
archiver extract --dest <dir> <archive>
```

## Pack placement

Archives are scattered under the output hierarchy:

```text
output/
  archiver-map.json
  docs/part-0001.zip
  photos/part-0001.zip
  part-0001.zip
```

## Testing

- Domain unit tests: pack planner, media sizes, path safety.
- Integration: `tests/archive_cli.rs`, `tests/pack_restore.rs` with tempfile.
