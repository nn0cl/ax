# ADR 0025: Auto compress jobs (`--at` / I/O-aware)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: LISS-0009 (parallel units), LISS-0015 (this feature), ADR 0022
  (dashboard height tracks jobs)

## Context

Pack compresses independent units in parallel (`--jobs` / `-j`), while
**transfer remains single-threaded**. Default jobs were `min(CPU, 8)`.

Operators packing many units on multi-core hosts want more than 8 workers when
the disk can feed them. On slow USB / network, too many concurrent compress
threads **contend on source reads** (and without `--work-dir`, on the output
volume), can fill the stage queue, and do not finish wall-clock faster when
transfer is the bottleneck.

Idle threads already yield CPU to other processes; mid-run thread-pool resize
is not required for fairness. What is required is a **good fixed N** chosen
from CPU, plan size, and media class.

## Dependency Adoption Evidence

Not applicable (stdlib `available_parallelism` only).

## Decision

### 1. CLI / config

| Surface | Meaning |
|---------|---------|
| `-j` / `--jobs N` | **Fixed** compress workers. Always wins. |
| `--at` / `--auto-jobs` | After plan, **auto-tune** N. Ignored if `-j` is set. |
| Config `jobs=` | Same as `-j` (explicit). |
| Config `auto_jobs=true` | Same as `--at` when `jobs=` is unset. |

Default (neither): `min(CPU, 8)` — unchanged safe baseline.

### 2. When auto runs

1. Scan + plan complete; count **pending units** and **source files**.
2. Compute N via `resolve_compress_jobs` (`src/core/domain/jobs.rs`).
3. Apply `progress.set_jobs(N)` so dashboard height matches.
4. Resource check and rayon pool use that N.
5. Log a one-line summary, e.g.  
   `auto_jobs: cpu=… units=… files=… io_cap=… (usb3 thr=… work_dir=…) → jobs=…`

N is **fixed for the rest of the pack**. No mid-run up/down of the thread pool
in this ADR (deferred; see Consequences).

### 3. Auto formula (normative intent)

```text
jobs = min(cpu_cap, units_pending, io_cap, file_heuristic)
```

| Factor | Rule |
|--------|------|
| **cpu_cap** | `min(available_parallelism, 32)` |
| **units_pending** | Never more workers than units still to build |
| **io_cap** | From `--connection`, optional throughput, work-dir, compress-only |
| **files** | Very few files → keep N small; many files may raise toward the caps |

#### I/O cap (`io_jobs_cap`)

- Prefer **measured / CLI `--throughput`** when present; else practical default
  for the connection class (ADR-related transfer defaults).
- Connection class ceilings (approx.):

  | Connection | Class ceiling |
  |------------|---------------|
  | local / thunderbolt | up to 32 (CPU-limited) |
  | usb3 | 6 |
  | usb2 / network | 3 |
  | unset + media pack | treat as usb3 |
  | unset + `compress-only` | treat as local |

- Throughput bands further tighten the cap (slow stick → 2).
- **No work-dir** and not compress-only: multiply cap by ~¾ (output contention).
- **With work-dir**: allow one extra slot up to the class ceiling.

### 4. Related: `--max-use` (serial low free-space mode)

Not auto-jobs, but related operator control:

| Flag | Effect |
|------|--------|
| `--max-use 4G` | Cap **unit budget** to ≤ 4G; force **jobs=1** and **stage queue=0** (rendezvous). Pipeline: compress one → transfer → delete stage → next. |

Use when free space is tight: only one primary archive occupies stage at a time.
Config: `max_use=4G`. A single source file larger than the cap may still exceed it
(existing over-budget whole-file rule).

### 5. Explicit non-goals (this ADR)

- Mid-pack dynamic rayon resize / “elastic” thread count.
- Per-file parallelism inside one unit.
- Guaranteeing optimal wall-clock on every medium (AS IS; ADR 0014).

### 6. Idle threads

Idle compress slots **do not monopolize CPU**; the OS can schedule other
processes. Reducing N mid-run is therefore **not** required for multi-tenant
CPU fairness. I/O contention and stage pressure remain the reasons for a
conservative `io_cap`.

## Consequences

Positive:

- Operators can drop the default max-8 when plan + media allow.
- Slow media gets fewer concurrent compressors without manual `-j` guesswork.
- Dashboard and resource estimates stay aligned with the chosen N.

Negative / deferred:

- N is static after plan; media throttling mid-run is not auto-tracked.
- Future option: queue backpressure or concurrent-slot limiting without
  resizing the pool (would be a new ADR if implemented).

## Enforcement

Code review should reject:

- Changing default (non-`--at`) max from 8 without revisiting this ADR.
- Mid-run `num_threads` mutation without a new ADR.
- Ignoring connection/throughput when implementing further auto-jobs changes.

## Implementation map

| Item | Location |
|------|----------|
| Formula + I/O cap | `src/core/domain/jobs.rs` |
| Request fields | `PackRequest::{auto_jobs,jobs_explicit,connection,throughput_bps}` |
| Retune after plan | `src/core/application/pack/mod.rs` |
| CLI `--at` / config | `args.rs`, `pack_config.rs` |
| Dashboard resize | `PackProgress::set_jobs` / `progress/mod.rs` |

## Verification

- Unit tests in `jobs.rs` (explicit, default max 8, usb2/usb3/local, throughput).
- Manual: `--at --connection usb3 --work-dir …` logs `io_cap` and stable N.
- `-j 4 --at` → jobs remain 4.
