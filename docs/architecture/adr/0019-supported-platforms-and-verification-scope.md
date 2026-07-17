# ADR 0019: Supported platforms and verification scope

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: ADR 0014, README “verification gaps”, `docs/QA.md`

## Context

Boundary conditions (huge trees, flaky USB, Windows, concurrent writers)
cannot all be certified. Silence implies unlimited support; a matrix sets
honest expectations.

## Dependency Adoption Evidence

Not applicable.

## Decision

### Support tiers

| Tier | Scope |
|------|--------|
| **Supported** | macOS and Linux; local paths; regular files/dirs; UTF-8-ish paths; pack/restore/verify/estimate primary path; plan v2 |
| **Best-effort** | Slow/USB volumes; huge single files (no byte-split); `--verify-stage`; content-class estimates; optical-ish budgets via `--media` / `--volumes` |
| **Unsupported** | Windows (not certified); concurrent pack to same `output`/`work-dir`; guaranteed behavior under ACL/TCC denials; multi-million inode trees; live network filesystems with arbitrary disconnects |

### Verification obligations

| Layer | Obligation |
|-------|------------|
| CI (`cargo test`, clippy) | Regression on supported path; Linux (and local macOS dev) |
| Manual QA (`docs/QA.md`) | Optional pre-release checklist (USB, Ctrl+C, resume) |
| User | `verify` / restore drill for important data |

### Concurrent use

Same `output` or stage `work-dir` from two processes is **unsupported**.
No file locking is promised in v0.x.

## Consequences

Positive:

- Docs and issues can close “works on Windows?” as out of tier
- Test investment prioritizes Supported + cheap Best-effort

Negative:

- Some real deployments remain untested by design

## Enforcement

Code review should reject:

- README claims of “full OS support” without matrix update
- Treating concurrent multi-writer as a supported feature without design
