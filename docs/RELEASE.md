# Release process

How to cut a versioned release of **archiver**.

## Versioning

- **SemVer** in `Cargo.toml` (`major.minor.patch`)
- Git tag: `vX.Y.Z` (e.g. `v0.0.1`)
- Document changes in `CHANGELOG.md` under `[Unreleased]`, then move to `[X.Y.Z]` on release

| Bump | When |
|------|------|
| **patch** | Bugfix, docs, internal refactors (no intentional CLI break) |
| **minor** | New flags/commands, compatible behavior extensions |
| **major** | Breaking CLI/mapping/plan schema changes |

## Preconditions

```bash
git status          # clean working tree
cargo test
cargo clippy --all-targets -- -D warnings   # if clippy is available
cargo build --release
./target/release/archiver --version
./target/release/archiver --help | head
```

## Steps (manual)

1. Update `CHANGELOG.md`: move `[Unreleased]` items under `## [X.Y.Z] - YYYY-MM-DD`
2. Set version in `Cargo.toml` to `X.Y.Z`
3. Run `cargo build` so `Cargo.lock` stays consistent if needed
4. Commit: `chore(release): vX.Y.Z`
5. Tag: `git tag -a vX.Y.Z -m "archiver vX.Y.Z"`
6. Push: `git push origin main` and `git push origin vX.Y.Z`
7. Create GitHub Release (optional): attach `target/release/archiver` or CI artifacts

Helper script:

```bash
./scripts/release.sh 0.1.1
# dry-run:
./scripts/release.sh 0.1.1 --dry-run
```

## CI

- Pull requests / pushes run tests via `.github/workflows/ci.yml`
- Optional: add a `release.yml` that builds on tag `v*` and uploads binaries

## Install after release

Users on macOS/Linux can:

```bash
# From source (clone)
./scripts/install.sh

# From GitHub Release asset (after tag push + release workflow)
REPO=owner/repo ./scripts/install.sh --from-release
```

See [INSTALL.md](INSTALL.md).

## What a release does **not** include

- No SLA, no fitness-for-purpose warranty
- Automated tests reduce regressions; they do not guarantee every environment
  (see **Known verification gaps** in README.ja.md)

Use at your own risk.
