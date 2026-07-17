#!/usr/bin/env bash
# Cut a SemVer release: changelog note, Cargo.toml version, commit, annotated tag.
# Usage: ./scripts/release.sh X.Y.Z [--dry-run]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${1:-}"
DRY_RUN=0
if [[ "${2:-}" == "--dry-run" ]] || [[ "${1:-}" == "--dry-run" ]]; then
  if [[ "${1:-}" == "--dry-run" ]]; then
    echo "usage: $0 X.Y.Z [--dry-run]" >&2
    exit 2
  fi
  DRY_RUN=1
fi

if [[ -z "$VERSION" || ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: $0 X.Y.Z [--dry-run]" >&2
  exit 2
fi

TAG="v${VERSION}"
DATE="$(date +%Y-%m-%d)"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree not clean" >&2
  exit 1
fi

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "error: tag $TAG already exists" >&2
  exit 1
fi

echo "==> cargo test"
if [[ "$DRY_RUN" -eq 0 ]]; then
  cargo test -q
else
  echo "(dry-run) skip cargo test"
fi

echo "==> set Cargo.toml version = $VERSION"
if [[ "$DRY_RUN" -eq 0 ]]; then
  # portable-ish: only first version = line under [package]
  if [[ "$(uname)" == "Darwin" ]]; then
    sed -i '' -E "s/^version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"/version = \"${VERSION}\"/" Cargo.toml
  else
    sed -i -E "s/^version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"/version = \"${VERSION}\"/" Cargo.toml
  fi
else
  echo "(dry-run) would set version in Cargo.toml"
fi

echo "==> remind: move CHANGELOG [Unreleased] → [$VERSION] - $DATE"
if ! grep -q "\[$VERSION\]" CHANGELOG.md 2>/dev/null; then
  echo "warning: CHANGELOG.md has no section [$VERSION] yet — edit before push" >&2
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "(dry-run) would: commit + tag $TAG"
  exit 0
fi

git add Cargo.toml Cargo.lock CHANGELOG.md 2>/dev/null || git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): v${VERSION}"
git tag -a "$TAG" -m "archiver v${VERSION}"

echo ""
echo "Release prepared locally:"
echo "  commit + tag $TAG"
echo "Push with:"
echo "  git push origin HEAD"
echo "  git push origin $TAG"
echo "Optional binary:"
echo "  cargo build --release"
echo "  ./target/release/ax --version"
