# ADR 0014: Product quality stance, versioning, and support scope

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: README disclaimer, `docs/RELEASE.md`, `docs/INSTALL.md`

## Context

The archiver is a local-first media-fit tool. Users and reviewers ask whether
it is “product quality.” That phrase conflates:

1. Feature completeness for a stated workflow
2. Automated regression coverage
3. Commercial warranty / SLA / multi-platform certification

Without an explicit decision, version numbers (e.g. `1.0.0`) and docs can
imply guarantees the project does not provide.

## Dependency Adoption Evidence

Not applicable.

## Decision

1. **Warranty**: Software is provided **AS IS**, no warranty, **no SLA**, no
   data-loss compensation. Use at own risk. Automated tests reduce
   regressions; they do not certify every environment or corpus.
2. **Support**: No contractual support. Optional best-effort maintainer
   response only; no response-time commitment.
3. **Version semantics (0.x product line)**:
   - `0.0.x` — early product; CLI/mapping/plan may change with CHANGELOG notes
   - `0.1.0` — **usable baseline**: documented primary path
     (pack / restore / verify / estimate, plan v2, optional config &
     verify_stage) is intentional; breaking changes require CHANGELOG +
     minor/major bump rules in `docs/RELEASE.md`
   - `1.0.0` — **not targeted** until (if ever) the project accepts stronger
     compatibility and support commitments
4. **User duty**: Important data requires user-run `verify` (and optionally
   `--verify-stage`), restore drills, and multi-media copies.

## Consequences

Positive:

- Clear product identity: practical tool, not insured backup appliance
- Version bumps communicate stability of *documented paths*, not legal fitness

Negative:

- Some users will still expect commercial backup product behavior
- Marketing language must stay conservative

## Enforcement

Code review / docs should reject:

- Claims of “guaranteed restore,” SLA, or “certified for all media”
- Jumping to `1.0.0` without revisiting this ADR
- Removing the AS IS disclaimer from CLI help / README
