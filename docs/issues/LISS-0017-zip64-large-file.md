# LISS-0017: ZIP64 large-file support in zip engine

## Metadata

- Local issue ID: LISS-0017
- GitHub issue: none
- Status: review (fix verified on branch; awaiting PR review/merge)
- Phase: 1+2 complete (Adjudicator approved combined Red+Green for size-S bug)
- Type: bug
- Priority: high
- Initial planning size: S
- Current planning size: S
- Reclassification reason: n/a
- Owner/agent: Claude Code (Fable 5)
- Related branch: fix/zip64-large-file

## Summary

- Real pack run failed after 1h12m while writing `part-0001.zip` (38.7G) with
  `error: I/O error: Large file option has not been set`.
- The `zip` crate requires `large_file(true)` on `SimpleFileOptions` before a
  single member may exceed 4 GiB (ZIP64). `zip_engine.rs` never sets it.
- Secondary defect in the same function: the source file is read fully into
  memory (`read_to_end`) before writing, so multi-GiB members risk memory
  exhaustion even after the ZIP64 fix. Replace with streaming `io::copy`.

## Acceptance Notes

- Given a source file larger than 4 GiB, when `ax create --output out.zip`
  runs, then it succeeds and `ax list out.zip` reports the member with its
  full size (> 4 GiB).
- Memory usage does not grow proportionally to member size (streaming write).
- Existing create/list/extract behavior for small files is unchanged.
- The > 4 GiB test is `#[ignore]` (too heavy for CI per testing-strategy
  "Huge trees ... not automated"); it is executed manually for this fix and
  recorded under Verification.

## Dependencies

- Parent: none
- Depends on: none
- Blocks: none
- Related: LISS-0001 (archive CLI MVP), docs/specs/archive-cli.md

## Adjudicator Decision Points

- Approved: size-S bug triage without separate work plan; Phase 1+2 combined
  in one attempt (this session, 2026-07-17).

## Context

- Included: `src/adapters/archive/zip_engine.rs`, `tests/archive_cli.rs`,
  `Cargo.toml` (zip 2.2), reproduced error output.
- Omitted: pack pipeline, domain modules, tar engines, progress UI.
- Assumptions: tar-based formats handle > 8 GiB members via the `tar` crate's
  automatic PAX extensions; out of scope unless observed failing.

## AI Planning Records

- Not required (size S, first attempt).

## References

- zip crate `FileOptions::large_file` (ZIP64 opt-in for members > 4 GiB).

## Work Notes

- 2026-07-17: diagnosed from user-reported pack failure; branch created.

## Verification

- 2026-07-17 `cargo test --release`: 27 passed, 0 failed.
- 2026-07-17 `cargo test --release --test archive_cli
  create_zip_member_over_4gib -- --ignored`: Red before fix (reproduced the
  reported error verbatim), Green after fix in 7.2s (create + list + extract
  roundtrip, extracted size 4 GiB + 16 B).
- 2026-07-17 `cargo clippy --all-targets --release -- -D warnings`: clean.
- Trace: `docs/collaboration/traces/2026-07-17-liss-0017-zip64-large-file.md`.
