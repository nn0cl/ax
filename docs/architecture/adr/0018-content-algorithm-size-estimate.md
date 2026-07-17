# ADR 0018: Content-class × algorithm archive size estimate

- Status: **Accepted**
- Date: 2026-07-17
- Deciders: Product owner (nn0cl); recorded as approved
- Related: ADR 0013 (packing budget remains logical), `content_class.rs`,
  progress write ETA

## Context

Write progress % and transfer ETA need an early estimate of archive bytes
before any unit finishes. A single format prior (e.g. tar.zst = 0.55) is
wrong for photo/video-heavy trees (near 1.0) and for text-heavy trees
(much lower).

Users asked for estimates using **compression algorithm + file type**.
This must not be confused with **compressed-size packing** (ADR 0013).

## Dependency Adoption Evidence

Not applicable (pure domain tables by extension).

## Decision

1. **Classify** each source path by extension into a `ContentClass`
   (text, source, image_lossy, video, already_archived, …).
2. Each class has a **zstd baseline** `base_ratio` (expected archive/logical).
3. Each `ArchiveFormat` has **compression_efficiency** vs zstd
   (`tar` = 0 store, `tar.zst` = 1.0, `tar.gz` ≈ 0.88, `zip` ≈ 0.82).
4. Combined model:
   `ratio ≈ 1 - (1 - base_zstd) * efficiency` (+ small container overhead);
   store formats use ≈ 1.02 regardless of class.
5. **Weighted sum** over files → `ContentAwareEstimate` (bytes + ratio +
   per-class breakdown). Logged as `size_estimate:`.
6. **Progress / ETA** uses this ratio as `ratio_prior` until the first unit
   transfer completes; then **empirical** `written / logical_transferred`.
7. **Free-space hard gate** remains **logical × safety** (worst case). Content
   estimate is informational only.
8. **Pack unit budget** remains **logical** (ADR 0013 option A). This ADR
   does not implement compressed packing.

## Consequences

Positive:

- Better early ETA for media vs source trees
- Transparent class breakdown in dry-run

Negative:

- Extension mislabels and unknown types wrong
- Tiny files ceil to whole bytes (ratio displayed from float sum)

## Enforcement

Code review should reject:

- Using content prior as free-space hard fail without a new ADR
- Using content prior as pack budget without reopening ADR 0013
- Claiming estimate accuracy guarantees
