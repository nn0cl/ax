# Install (macOS & Linux)

Install **ax** (Archive eXchange / package name `archiver`) on macOS or Linux.

| | |
|--|--|
| **Product version** | **v0.0.1** (`Cargo.toml` / `ax --version`) |
| **Default binary** | **`ax`** (`cargo build` / `cargo install`; `default-run = "ax"`) |
| **Compat name** | optional `archiver` binary (`--features compat-bin`) or symlink → `ax` |
| **Disclaimer** | AS IS, no warranty, no SLA — use at your own risk |

## Dependency versions

### Toolchain (build host)

| Dependency | Requirement | Notes |
|------------|-------------|--------|
| **Rust** | **1.74+** (`rust-version` in `Cargo.toml`) | Tested with **rustc 1.96** (Homebrew / rustup stable) |
| **Cargo** | same as Rust toolchain | via rustup |
| **C compiler** | required | macOS: Xcode CLT; Linux: `gcc`/`clang` + `make` (for `cc` / `zstd-sys`) |
| **pkg-config** | optional | some crates may use it |

Exact crate resolutions are **pinned in `Cargo.lock`**. Prefer:

```bash
cargo build --release --locked
# or
./scripts/install.sh   # uses the tree’s lockfile when building
```

### Direct crates (`Cargo.toml` ranges → locked in this tree)

Ranges below are what `Cargo.toml` requests. **Locked** = version resolved in `Cargo.lock` at packaging time (update with `cargo update` carefully).

| Crate | Cargo.toml | Locked (example / current tree) |
|-------|------------|----------------------------------|
| clap | 4.5 | 4.6.2 |
| thiserror | 2.0 | 2.0.18 |
| zip | 2.2 | 2.4.2 |
| tar | 0.4 | 0.4.46 |
| flate2 | 1.0 | 1.1.9 |
| walkdir | 2.5 | 2.5.0 |
| serde | 1.0 | 1.0.228 |
| serde_json | 1.0 | 1.0.150 |
| zstd | 0.13 (feature `zstdmt`) | 0.13.3 |
| libc | 0.2 | 0.2.186 |
| sha2 | 0.10 | 0.10.9 |
| hex | 0.4 | 0.4.3 |
| rayon | 1.10 | 1.12.0 |
| ctrlc | 3.4 | 3.5.2 |
| tempfile (dev) | 3.14 | 3.27.0 |
| assert_cmd (dev) | 2.0 | 2.2.2 |
| predicates (dev) | 3.1 | 3.1.4 |

To print current lock versions:

```bash
cargo tree -d
# or
cargo metadata --format-version 1 | ...
```

### Runtime OS

| OS | Status |
|----|--------|
| macOS | Supported (primary real-world use) |
| Linux (glibc x86_64) | Supported (CI + install script) |
| Windows | Not officially supported |

## Quick install (recommended)

From a clone of this repository:

```bash
# Requires: Rust (rustup), a C toolchain (for zstd-sys / crates that need cc)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # if needed
source "$HOME/.cargo/env"

git clone <THIS_REPO_URL> archiver
cd archiver
./scripts/install.sh
```

This builds a **release** binary and installs to `~/.local/bin/archiver` by default
(no root required). Ensure `~/.local/bin` is on your `PATH`:

```bash
# bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc

# zsh (macOS default)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
```

Verify:

```bash
archiver --version
archiver --help
```

---

## Options for `scripts/install.sh`

```bash
./scripts/install.sh                 # release build from this tree → ~/.local/bin
./scripts/install.sh --prefix /usr/local   # may need sudo
./scripts/install.sh --debug         # debug build (faster compile, slower runtime)
./scripts/install.sh --cargo-install # `cargo install --path . --force`
./scripts/install.sh --from-release  # download GitHub Release binary (if published)
./scripts/install.sh --uninstall
```

Environment:

| Variable | Meaning |
|----------|---------|
| `PREFIX` | Install prefix (default: `$HOME/.local`) |
| `BIN_DIR` | Override binary directory (default: `$PREFIX/bin`) |
| `REPO` | GitHub `owner/repo` for `--from-release` (default: from `git remote`) |
| `VERSION` | Release tag without `v`, e.g. `0.0.1` (default: latest) |

Examples:

```bash
PREFIX=/usr/local sudo -E ./scripts/install.sh
REPO=myorg/archiver VERSION=0.0.1 ./scripts/install.sh --from-release
```

---

## Method A — From source (cargo)

### Prerequisites

| Tool | macOS | Linux |
|------|-------|-------|
| Rust | [rustup](https://rustup.rs/) | same |
| C compiler | Xcode CLT: `xcode-select --install` | `build-essential` / `gcc` + `make` |
| (optional) pkg-config | Homebrew `pkg-config` | distro package |

### Build & install

```bash
cd /path/to/archiver
cargo build --release
install -m 755 target/release/archiver ~/.local/bin/archiver

# or
cargo install --path . --force --root ~/.local
# binary at ~/.local/bin/archiver
```

System-wide (requires write access):

```bash
cargo build --release
sudo install -m 755 target/release/archiver /usr/local/bin/archiver
```

### From git without cloning manually

```bash
cargo install --git <THIS_REPO_URL> --locked --force
# installs to ~/.cargo/bin (ensure that is on PATH)
```

---

## Method B — Prebuilt GitHub Release

After a `vX.Y.Z` tag is pushed, CI may publish assets:

- `archiver-linux-x86_64`
- `archiver-macos-universal` (or host-specific name)

```bash
# Example (adjust REPO / asset names to match the Release page)
REPO=owner/archiver
VERSION=0.0.1   # or omit for latest via install.sh

# macOS
curl -fsSL -o archiver \
  "https://github.com/${REPO}/releases/download/v${VERSION}/archiver-macos-universal"
chmod +x archiver
install -m 755 archiver ~/.local/bin/archiver

# Linux x86_64
curl -fsSL -o archiver \
  "https://github.com/${REPO}/releases/download/v${VERSION}/archiver-linux-x86_64"
chmod +x archiver
install -m 755 archiver ~/.local/bin/archiver
```

Or:

```bash
./scripts/install.sh --from-release
```

---

## Method C — Homebrew (optional, when you have a tap)

Not published to homebrew-core by default. If you maintain a tap:

```bash
brew install yourname/tap/archiver
```

Formula sketch (for a private tap): see comments in `scripts/homebrew/archiver.rb.example`.

---

## Uninstall

```bash
./scripts/install.sh --uninstall
# or
rm -f ~/.local/bin/archiver
rm -f /usr/local/bin/archiver
cargo uninstall archiver   # if installed via cargo install
```

---

## Platform notes

### macOS

- Apple Silicon and Intel: build on the target machine, or use the release asset if universal.
- USB paths are often under `/Volumes/...`.
- Full Disk Access / TCC may block reading Desktop, Downloads, etc. without permission or `sudo`.
- Prefer a **release** build for real backups: `cargo build --release`.

### Linux

- Works on typical glibc x86_64 (Ubuntu/Debian CI).
- musl static builds are not the default CI artifact; build from source on the target distro if needed.
- Ensure `~/.local/bin` or `/usr/local/bin` is on `PATH`.

---

## Verify install

```bash
which archiver
archiver --version
archiver pack --help
```

Copyright (c) nn0cl@dstechnology.co.jp. All rights reserved.
