# ax product design book (index)

Consolidated map of product design artifacts for the **ax** media-fit archiver.
Use this as the entry point; normative rules live in ADRs and specs.

## Product identity

| Item | Decision |
|------|----------|
| Package name | `archiver` (Cargo) |
| Default binary | **`ax`** (Archive eXchange) — ADR 0020 |
| Version line | **v0.1.0** usable baseline — ADR 0014; see `CHANGELOG.md` |
| Warranty | AS IS, no SLA — ADR 0014 |
| Platforms | macOS/Linux supported — ADR 0019 |
| Concurrent pack | Rejected at runtime (`.ax-pack.lock`) — ADR 0026 |

## Architecture map

```text
docs/specs/media-fit-pack.md          ← pack/restore behavioral spec
docs/architecture/media-fit-pack-design.md
docs/architecture/rust-cli-architecture.md
docs/architecture/adr/README.md       ← ADR index
docs/architecture/design-notes/       ← compact notes per change unit
```

## Decision index (product)

| ADR | Topic |
|-----|--------|
| 0013 | Logical pack budget (no strict compressed packing) |
| 0014 | Quality / versioning / support stance |
| 0015 | Strict plan v2 + mapping integrity |
| 0016 | Optional `verify_stage` |
| 0017 | Config `.inf` / CLI precedence |
| 0018 | Content × algorithm size **estimate** (not budget) |
| 0019 | Support tiers |
| **0020** | **`ax` short CLI** |
| **0021** | **Size presets** |
| **0022** | **Progress dashboard + UI thread** (`total` / `cmp*` / `trn`) |
| **0023** | **Log verbosity `-v`/`-vv`** |
| **0024** | **CLI screen geometry** (80 baseline; wide 120–200; name·tag; fixed nums) |
| **0025** | **Auto compress jobs (`--at`)** (CPU × units × I/O; no mid-run resize) |
| **0026** | **Pack exclusive lock** (`.ax-pack.lock` on output/work-dir) |

## Issue index (recent product)

| LISS | Topic |
|------|--------|
| 0008–0009 | Early dry-run / progress / parallel pack |
| 0010 | Compressed packing ADR |
| **0011** | Progress dashboard (UI thread, geometry, labels) |
| **0012** | ax CLI (retro) |
| **0013** | Size presets |
| **0014** | Log verbosity |
| **0015** | Auto compress jobs (`--at`, I/O-aware) |
| **0016** | Pack exclusive lock |

## Design notes (compact)

| Note | Topic |
|------|--------|
| `2026-07-17-liss-0011-progress-dashboard.md` | Dashboard modules + labels + wide mode |
| `2026-07-17-adr-0024-cli-screen-impl.md` | ADR 0024 implementation checklist |
| `2026-07-17-auto-compress-jobs.md` | `--at` formula + non-goals |
| `2026-07-17-ax-cli-and-presets.md` | Binary, shorts, presets |
| `2026-07-17-pack-logging-and-verbosity.md` | Quiet default / `-vv` |

## Runtime pack path (operator view)

```text
ax pa -i SRC -o OUT -s PRESET|SIZE -f zip|tar.zst -j N [--work-dir DIR]
        │
        ├─ scan / plan (logical budget)
        ├─ resource gate (logical × safety)
        ├─ auto_jobs? (ADR 0025: after plan, fixed N)
        ├─ dashboard (TTY): total + cmp* + trn  (ADR 0022/0024)
        └─ mapping + plan files under OUT
```

## Clean Architecture placement

| Concern | Layer |
|---------|--------|
| Size presets, parse_size | domain |
| Pack orchestration | application |
| Progress **policy** (what to show) | delivery (render pure) |
| Progress **paint** | delivery/tty |
| Screen geometry (80 cols) | ADR 0024 + `progress/width.rs` |
| Zip/tar engines | adapters |

## Gaps intentionally deferred

- Compressed-size packing (ADR 0013 C/D)
- Windows certification (ADR 0019)
- Full AT-TDD phase gates on every hot-fix (process restored via PR #4+)
