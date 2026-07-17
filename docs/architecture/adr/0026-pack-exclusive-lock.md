# ADR 0026: Exclusive pack lock (output / work-dir)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: ADR 0019 (concurrent pack was **unsupported**), LISS-0016

## Context

Concurrent `ax pa` processes writing the same `output` or sharing a `work-dir`
base can corrupt plan/mapping files and stage directories. ADR 0019 declared this
unsupported but the runtime did not enforce it.

## Decision

1. At pack start, create an exclusive lock file:
   - `{output}/.ax-pack.lock`
   - `{work_dir}/.ax-pack.lock` when `--work-dir` is set (if distinct from output)
2. Acquisition uses **create-new** (atomic fail if exists).
3. Lock contents include `pid=` and timestamp for operator diagnostics.
4. Lock is released on process exit via RAII (`Drop` removes the files).
5. Second pack receives [`ArchiveError::PackBusy`] with path + hint.
6. Stale locks (crash) may require **manual removal** of `.ax-pack.lock`
   (no cross-host process liveness API in v0.x).

## Non-goals

- Distributed locks / NFS lease correctness guarantees
- Locking restore/verify (read-only or separate paths)
- Automatic stale-lock reclamation by probing remote PIDs

## Implementation

- `src/core/application/pack/lock.rs`
- `FileSystem::create_new_file` / `read_to_string`
- Acquired in `PackSourceTree::execute_with_progress_inner` before plan I/O

## Consequences

Positive: concurrent writers fail fast with a clear error.  
Negative: unclean kill leaves a lock file until deleted.
