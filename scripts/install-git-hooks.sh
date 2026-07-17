#!/usr/bin/env bash
# Install repo git hooks (pre-push → clippy gate matching CI).
# Usage (from repo root):
#   ./scripts/install-git-hooks.sh

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
hooks_dir="$root/scripts/git-hooks"

if [[ ! -d "$hooks_dir" ]]; then
  echo "missing $hooks_dir" >&2
  exit 1
fi

chmod +x "$hooks_dir"/* 2>/dev/null || true

# Prefer core.hooksPath so hooks are versioned in-repo (not copied to .git/hooks).
git -C "$root" config core.hooksPath scripts/git-hooks

echo "Installed git hooks via: git config core.hooksPath scripts/git-hooks"
echo "  pre-push runs: cargo clippy --all-targets -- -D warnings"
echo "  skip: AX_SKIP_CLIPPY=1 git push   or   git push --no-verify"
