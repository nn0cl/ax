# ADR 0015: Strict plan v2 and mapping integrity (no legacy fallbacks)

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: plan file `archiver-plan.json`, mapping `archiver-map.json`, resume

## Context

Earlier designs allowed soft paths (size-only resume skip, empty member
hashes, resume without plan starting fresh). Soft paths hide corruption and
complicate testing (“did resume really verify?”).

Product stance for v0.0.1+ is **self-responsibility with strict integrity**:
fail closed rather than guess.

## Dependency Adoption Evidence

Not applicable (SHA-256 via existing `sha2` adapter).

## Decision

1. **Plan schema**: version **2** only. Done units require non-empty
   `archive_sha256` (and size). No size-only “done” recognition.
2. **`--resume`**: existing plan file is **required**; missing plan is an
   error (no soft “start fresh”).
3. **Mapping members**: non-empty `sha256` required on write/read paths used
   for integrity; empty hash is rejected.
4. **Version mismatch** on plan/mapping: hard error; re-pack or migrate
   explicitly (no silent accept).
5. **Pack-time**: member SHA-256 at hash phase; archive blob SHA-256 after
   transfer (for resume fingerprinting).

## Consequences

Positive:

- Resume and verify have a single integrity story
- Fewer ambiguous partial states

Negative:

- Older experimental plans/maps are unusable without re-pack
- Slightly more I/O (archive hash every unit)

## Enforcement

Code review should reject:

- Size-only resume skip
- Serde defaults that re-introduce empty `sha256` as success
- Soft-continue when plan version is wrong
