# LISS-0016: Exclusive pack lock (concurrent writer guard)

## Metadata

- Local issue ID: LISS-0016
- Status: done
- Type: reliability
- Priority: P1
- ADR: **0026**
- Related: ADR 0019 (was policy-only)

## Summary

Enforce single-writer for pack on a given `output` / `work-dir` via
`.ax-pack.lock` (create-new). Second process fails with `pack busy`.

## Acceptance

1. Two concurrent packs to same output: one succeeds, one gets clear error.
2. Lock released on normal exit (and Drop).
3. work-dir base also locked when set.
4. Unit tests for second acquire / release.

## Non-goals

- Automatic stale lock cleanup across machines
- restore/verify locks
