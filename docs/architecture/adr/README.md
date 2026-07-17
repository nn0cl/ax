# Architecture Decision Records (product)

Collaboration-template ADRs (`0001`–`0012`) govern AI/process.  
Product/runtime decisions for **archiver** start at **0013**.

## Product ADRs (summary)

| ADR | Title | Status | One-line summary |
|-----|--------|--------|------------------|
| [0013](0013-compressed-size-packing-mode.md) | Compressed-size packing mode | Accepted (defer C/D) | Pack **budget = logical** size; strict compressed packing deferred |
| [0014](0014-product-quality-versioning-and-support.md) | Product quality, versioning, support | **Accepted** | AS IS, no SLA; 0.x semantics; no 1.0 without revisit |
| [0015](0015-strict-plan-and-mapping-integrity.md) | Strict plan/mapping integrity | **Accepted** | Plan v2 + archive_sha256; no size-only resume; fail closed |
| [0016](0016-optional-pack-stage-verify.md) | Optional stage verify | **Accepted** | `--verify-stage` opt-in pre/post transfer integrity |
| [0017](0017-config-file-and-cli-precedence.md) | Config file precedence | **Accepted** | `.inf`/`.ini` `[pack]`; **CLI > config > default** |
| [0018](0018-content-algorithm-size-estimate.md) | Content × algorithm size estimate | **Accepted** | Extension classes × format efficiency for ETA only; not budget |
| [0019](0019-supported-platforms-and-verification-scope.md) | Support & verification scope | **Accepted** | macOS/Linux supported; Windows/concurrent unsupported |
| [0020](0020-primary-binary-ax-and-short-cli.md) | Primary binary `ax` + short CLI | **Accepted** | default-run ax; shorts pa/es/…; globals -c/-s/-o |
| [0021](0021-environment-size-presets.md) | Size presets | **Accepted** | c2 / fat32 / flash / max (+ numeric) |
| [0022](0022-progress-dashboard-ui-thread.md) | Progress dashboard + UI thread | **Accepted** | fixed height; workers paint-free; CJK width |
| [0023](0023-pack-log-verbosity-policy.md) | Pack log verbosity | **Accepted** | quiet default; detail at -vv; dashboard ≠ -v |

## How these interact

```text
Pack budget (unit split)     ──► ADR 0013  (logical only)
Size estimate / write ETA    ──► ADR 0018  (content × algo prior → then empirical)
Free-space hard gate         ──► logical × safety (not 0018 prior)
Resume / mapping integrity   ──► ADR 0015
Extra pack integrity         ──► ADR 0016 (optional)
Config defaults              ──► ADR 0017
CLI surface / binary name    ──► ADR 0020
Named budgets (-s presets)   ──► ADR 0021
Live progress UI             ──► ADR 0022
Text log volume              ──► ADR 0023
Product promises             ──► ADR 0014 + 0019
```

## Design book

Consolidated entry: [`../ax-product-design-book.md`](../ax-product-design-book.md)

## Template ADRs (process)

See `0001`–`0012` in this directory for design-first routing, collaboration
governance, branching, etc.
