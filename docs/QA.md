# Manual QA checklist (pre-release / best-effort)

Automated tests cover the **Supported** path (ADR 0019). This list is for
humans before tagging a release (e.g. **v0.1.0**) or validating USB workflows.

**Not a certification.** Skip items that do not apply; record date/environment
if you file a release note. Product version: see `Cargo.toml` / `ax --version`.

## Environment

- [ ] OS: ________________ (macOS / Linux)
- [ ] version: `ax --version`
- [ ] `cargo test` green on this machine (or CI green for the tag)

## Core path

- [ ] `pack --dry-run` shows `size_estimate:` with `classes=` and `algo_eff=`
- [ ] `pack` small tree → `restore` → files match
- [ ] `verify` OK after pack; `verify --quick` OK
- [ ] `pack --config examples/archiver.inf` (with paths filled) or custom `.inf`
- [ ] CLI overrides config (e.g. `--jobs` / `--verify-stage`)
- [ ] Optional: `--at --connection usb3` logs `auto_jobs: … io_cap=… → jobs=N`; `-j 2 --at` keeps jobs=2
- [ ] Optional: `--max-use 4G` forces serial pipeline (jobs=1, one staged unit)
- [ ] Second concurrent `pack` to same `-o` fails with pack busy / `.ax-pack.lock`

## Integrity

- [ ] `--resume` without plan fails
- [ ] Interrupt mid-pack (Ctrl+C) → re-run with `--resume` continues or fails clearly
- [ ] Optional: `--verify-stage` on small tree succeeds
- [ ] Tamper one restored file → restore/verify with verify detects mismatch

## Resources / media (best-effort)

- [ ] Pack to USB or external volume; progress dashboard updates (total / cmp* / trn, elapsed/ETA)
- [ ] Optional: `stty cols 80` and `stty cols 120` — no row wrap garble; wide shows `archive | source`
- [ ] Insufficient free space → hard fail unless `--force-resources` (if tested)
- [ ] `--work-dir` on fast disk + output on slow volume; stage cleaned by default

## Explicit non-goals (do not block release)

- Windows
- Concurrent pack to same output
- Million-file trees
- Guaranteed behavior after USB unplug mid-write

## Sign-off (optional)

- Date:
- Tester:
- Notes:
