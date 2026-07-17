# ADR 0016: Optional pack-time stage integrity (`verify_stage`)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: ADR 0015, pack pipeline, CLI `--verify-stage`, config `verify_stage`

## Context

Full daily `verify` (extract + hash all archives) can be too expensive for
large packs. Users still want stronger confidence that:

1. compressed members match pack-time member hashes **before** slow transfer
2. the archive blob on the destination matches the staged blob **after** copy

Default-on would double CPU/disk for every pack.

## Dependency Adoption Evidence

Not applicable (uses existing archive engine + hasher ports).

## Decision

1. **Optional** flag `--verify-stage` / config `[pack] verify_stage=true`
   (default **false**).
2. When enabled, per unit after compress:
   - extract staged archive to a temp dir
   - re-hash members; must match mapping member SHA-256 and size
   - compute stage archive SHA-256
3. After transfer (rename or copy) to final path:
   - dest archive SHA-256 must equal stage SHA-256
4. On mismatch: **fail the pack** (no soft skip).
5. This does **not** replace post-pack `verify` or user restore drills.

## Consequences

Positive:

- Catches compress/extract bugs and many transfer bit-rot cases early
- Cost is opt-in

Negative:

- Large extra I/O when enabled
- Does not prove long-term media health after unmount

## Enforcement

Code review should reject:

- Enabling verify_stage by default without a new ADR
- Soft-continuing on stage/dest hash mismatch
