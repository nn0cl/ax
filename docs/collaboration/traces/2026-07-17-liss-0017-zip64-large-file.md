# Trace: LISS-0017 ZIP64 large-file fix (2026-07-17)

## User Request

- Real pack run failed after 1h12m writing `part-0001.zip` (38.7G):
  `error: I/O error: Large file option has not been set`.
- Adjudicator approved size-S bug triage with combined Phase 1+2 and
  requested the standard planning documents.

## Current Phase

- Phase 1+2 completed in one approved attempt (attempt 1).

## Canonical Issue

- `docs/issues/LISS-0017-zip64-large-file.md`, branch `fix/zip64-large-file`.

## Included Context

- `src/adapters/archive/zip_engine.rs`, `tests/archive_cli.rs`,
  `Cargo.toml` (zip 2.2), reproduced error output.

## Omitted Context

- Pack pipeline, domain modules, tar engines, progress UI, private data.

## Routing

- Single session, Claude Code (model: Fable 5), direct edits.
- Deterministic verification: `cargo test`, `cargo clippy -D warnings`.

## Execution Record (attempt 1)

1. Diagnosis: `zip` crate requires `FileOptions::large_file(true)` before a
   member may exceed 4 GiB (ZIP64); `zip_engine.rs` never set it. Secondary:
   whole-file `read_to_end` buffering risks memory exhaustion on huge members.
2. Phase 1 (Red): added `create_zip_member_over_4gib_succeeds` to
   `tests/archive_cli.rs` (`#[ignore]`, sparse 4 GiB+16 B source; heavy tests
   stay out of CI per testing-strategy). Observed Red with the exact reported
   error message.
3. Phase 2 (Green): in `create_zip_members_with_progress` set
   `.large_file(true)` and replaced buffered read with streaming
   `std::io::copy`. Reviewed test was not modified.
4. Verification: ignored test Green (create + list + extract roundtrip,
   extracted size 4 GiB+16 B); full suite 27 passed; clippy clean with
   `-D warnings`.

## Adjudicator Decisions

- Combined Phase 1+2 approved; separate work plan waived (size S).
- Tar-format large-member behavior explicitly out of scope (assumed handled
  by `tar` crate PAX extensions; open a new issue if observed failing).

## Changed Files

- `src/adapters/archive/zip_engine.rs`
- `tests/archive_cli.rs`
- `docs/issues/LISS-0017-zip64-large-file.md`
- `docs/collaboration/traces/2026-07-17-liss-0017-zip64-large-file.md`
