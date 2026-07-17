# LISS-0012: `ax` CLI short commands, globals, size presets

## Metadata

- Local issue ID: LISS-0012
- GitHub issue: (with PR if available)
- Status: done (implementation on main; process retro-documented)
- Phase: complete (acceptance documented retroactively)
- Type: feature
- Priority: P1
- Initial planning size: M
- Current planning size: M
- Reclassification reason: originally landed on main without issue; now tracked
- Owner/agent: grok-build
- Related branch: landed as `9cb31da` on `main`; follow-ups via LISS-0011 branch/PR
- Related: ADR 0017 (config), size presets in domain

## Summary

Primary binary **`ax`** (Archive eXchange), short subcommands (`pa`, `es`, …),  
global `-c` / `-s` / `-o`, and size presets (`c2`, `fat32`, `flash`, `max`).  
Compat: optional `archiver` via `compat-bin` or symlink.

## User story

As an operator running pack jobs from a shell,  
I want a short binary name and compact flags,  
so that common pack/estimate/restore commands are fast to type without a config file.

## Acceptance Notes

### Must have

1. Default binary name after `cargo build`: **`ax`** (`default-run = "ax"`).
2. Short commands with long aliases: cr/create, ex/extract, ls/list, es/estimate, pa/pack, re/restore, ve/verify.
3. Globals: `-c` config, `-s` size|preset, `-o` output/mapping path.
4. Pack source: `-i` / `--source` (not `-s`).
5. Size presets: `c2` (10MiB), `fat32` (3.8GiB), `flash` (1GiB), `max` (practically no split); numeric sizes still work.
6. Help documents presets and short forms.
7. Tests use `cargo_bin("ax")` and cover short pack / presets as needed.
8. Install script installs `ax` and links `archiver` → `ax` when appropriate.

### Explicit non-goals

- Windows-native installer.
- Removing long aliases.

## Dependencies

- Related: LISS-0011 (progress UI)
- ADR: 0014, 0017, 0018, 0019

## Adjudicator Decision Points

- Retroactive issue acceptance for process hygiene after main landing.
- Further CLI changes should go through branch + PR.

## Context

- Included: `src/delivery/cli/*`, `Cargo.toml` bins, README/INSTALL, `media::SizePreset`.
- Omitted: full CLI rewrite of restore/verify UX beyond mapping via `-o`.

## AI Planning Records

### AIP-0012-001

- Status: accepted (retro)
- Created by: Grok Build
- Created at: 2026-07-17
- Planning size: M
- Intended execution route: already implemented; document + gate future work
- Intended scope: process closure for ax CLI
- Assumptions: product owner continues ax as primary binary name

## Verification

- [x] Implementation present on `main` (`9cb31da` and related)
- [x] `cargo test` green in session
- [ ] Future CLI changes only via PR
