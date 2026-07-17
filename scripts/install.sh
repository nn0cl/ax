#!/usr/bin/env bash
# Install archiver on macOS or Linux.
# Usage:
#   ./scripts/install.sh
#   ./scripts/install.sh --prefix /usr/local
#   ./scripts/install.sh --cargo-install
#   ./scripts/install.sh --from-release
#   ./scripts/install.sh --uninstall
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
MODE="source" # source | cargo-install | release
PROFILE="release"
UNINSTALL=0

usage() {
  cat <<'EOF'
Install archiver (macOS / Linux)

  ./scripts/install.sh                 Build --release from this tree → $PREFIX/bin
  ./scripts/install.sh --prefix DIR    Install prefix (default: ~/.local)
  ./scripts/install.sh --debug         Debug build instead of release
  ./scripts/install.sh --cargo-install cargo install --path . --root $PREFIX
  ./scripts/install.sh --from-release  Download GitHub Release binary
  ./scripts/install.sh --uninstall     Remove installed binary

Env:
  PREFIX, BIN_DIR, REPO (owner/repo), VERSION (e.g. 0.1.0 or empty=latest)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --prefix) PREFIX="$2"; BIN_DIR="$PREFIX/bin"; shift 2 ;;
    --debug) PROFILE="debug"; shift ;;
    --cargo-install) MODE="cargo-install"; shift ;;
    --from-release) MODE="release"; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
  Darwin|Linux) ;;
  *)
    echo "error: unsupported OS '$OS' (supported: macOS, Linux)" >&2
    exit 1
    ;;
esac

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

detect_repo() {
  if [[ -n "${REPO:-}" ]]; then
    echo "$REPO"
    return
  fi
  local url
  url="$(git remote get-url origin 2>/dev/null || true)"
  # git@github.com:owner/repo.git  or  https://github.com/owner/repo.git
  if [[ "$url" =~ github\.com[:/]([^/]+)/([^/.]+)(\.git)?$ ]]; then
    echo "${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    return
  fi
  echo ""
}

install_file() {
  local src="$1"
  local dest="$BIN_DIR/ax"
  local compat="$BIN_DIR/archiver"
  mkdir -p "$BIN_DIR"
  if [[ -w "$BIN_DIR" ]]; then
    install -m 755 "$src" "$dest"
    ln -sfn ax "$compat"
  else
    echo "note: $BIN_DIR not writable; trying sudo"
    sudo install -m 755 "$src" "$dest"
    sudo ln -sfn ax "$compat"
  fi
  echo "installed: $dest (+ $compat -> ax)"
  "$dest" --version || true
}

do_uninstall() {
  for dest in "$BIN_DIR/ax" "$BIN_DIR/archiver"; do
    if [[ -e "$dest" || -L "$dest" ]]; then
      if [[ -w "$BIN_DIR" ]]; then
        rm -f "$dest"
      else
        sudo rm -f "$dest"
      fi
      echo "removed: $dest"
    fi
  done
  if command -v cargo >/dev/null 2>&1; then
    if cargo uninstall archiver 2>/dev/null; then
      echo "cargo uninstall archiver: ok"
    fi
  fi
}

if [[ "$UNINSTALL" -eq 1 ]]; then
  do_uninstall
  exit 0
fi

echo "OS=$OS ARCH=$ARCH MODE=$MODE PREFIX=$PREFIX BIN_DIR=$BIN_DIR"

case "$MODE" in
  source)
    need_cmd cargo
    need_cmd install
    echo "==> cargo build --${PROFILE}"
    if [[ "$PROFILE" == "release" ]]; then
      cargo build --release
      SRC="$ROOT/target/release/ax"
    else
      cargo build
      SRC="$ROOT/target/debug/ax"
    fi
    test -f "$SRC"
    install_file "$SRC"
    ;;

  cargo-install)
    need_cmd cargo
    echo "==> cargo install --path . --force --root $PREFIX (default bin: ax)"
    cargo install --path . --force --root "$PREFIX" --locked 2>/dev/null \
      || cargo install --path . --force --root "$PREFIX"
    # Ensure compat name points at ax when cargo only installed ax
    if [[ -x "$BIN_DIR/ax" && ! -e "$BIN_DIR/archiver" ]]; then
      ln -sfn ax "$BIN_DIR/archiver" 2>/dev/null || true
    fi
    if [[ -x "$PREFIX/bin/ax" && ! -e "$PREFIX/bin/archiver" ]]; then
      ln -sfn ax "$PREFIX/bin/archiver" 2>/dev/null || true
    fi
    echo "installed via cargo (default: ax)"
    if [[ -x "$BIN_DIR/ax" ]]; then
      "$BIN_DIR/ax" --version || true
    elif [[ -x "$PREFIX/bin/ax" ]]; then
      "$PREFIX/bin/ax" --version || true
    fi
    ;;

  release)
    need_cmd curl
    need_cmd install
    REPO_ID="$(detect_repo)"
    if [[ -z "$REPO_ID" ]]; then
      echo "error: set REPO=owner/repo (could not detect git remote)" >&2
      exit 1
    fi
    # Asset names must match .github/workflows/release.yml
    case "$OS" in
      Darwin) ASSET="archiver-macos-universal" ;;
      Linux)
        case "$ARCH" in
          x86_64|amd64) ASSET="archiver-linux-x86_64" ;;
          *)
            echo "error: no prebuilt asset for Linux $ARCH; use source install" >&2
            exit 1
            ;;
        esac
        ;;
    esac

    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT
    if [[ -n "${VERSION:-}" ]]; then
      URL="https://github.com/${REPO_ID}/releases/download/v${VERSION}/${ASSET}"
      echo "==> download $URL"
      curl -fsSL -o "$TMP/archiver" "$URL"
    else
      # latest: resolve tag via API
      need_cmd python3
      echo "==> resolve latest release for $REPO_ID"
      TAG="$(curl -fsSL "https://api.github.com/repos/${REPO_ID}/releases/latest" \
        | python3 -c 'import sys,json; print(json.load(sys.stdin).get("tag_name",""))')"
      if [[ -z "$TAG" ]]; then
        echo "error: no latest release found for $REPO_ID" >&2
        exit 1
      fi
      URL="https://github.com/${REPO_ID}/releases/download/${TAG}/${ASSET}"
      echo "==> download $URL"
      curl -fsSL -o "$TMP/archiver" "$URL"
    fi
    chmod +x "$TMP/archiver"
    install_file "$TMP/archiver"
    ;;
esac

# PATH hint
if ! command -v archiver >/dev/null 2>&1; then
  echo ""
  echo "note: 'archiver' not on PATH yet. Add:"
  echo "  export PATH=\"${BIN_DIR}:\$PATH\""
fi

echo "done."
