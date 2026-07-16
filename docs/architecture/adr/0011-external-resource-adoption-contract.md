# ADR 0011: Optional External Resource Adoption Contract

## Status

Accepted

## Context

ADR 0002 defines general input, output, and reasoning contracts for
AI-assisted tasks. `docs/architecture/dependency-policy.md` defines an
adoption checklist for software dependencies (packages, libraries, provider
SDKs, CLI tools, test helpers). Neither document states a rule for adopting
a *resource* — AI-generated or human-sourced — that is not a software
dependency: a generated artifact, a downloaded dataset, a third-party
content asset, or similar.

External feedback from real usage of this template (Adjudicator-supplied,
2026-07-13) showed this gap in practice: generated output was treated as
usable before any human verdict was recorded on it.

An earlier draft of this ADR defined the contract using image-specific
vocabulary (orientation, dimensions, crop policy, alpha/channel policy) and
a deterministic-vs-human boundary table naming specific artifact properties.
The Adjudicator rejected that draft: this template must avoid describing
specific technologies, and the actual principle needed is more general —
**no resource that originates outside the project's own reviewed work is
adopted without passing an explicit, recorded check, whether that resource
is AI-generated or brought in by a human, and whether the check is
automated or human.** Skipping the check is never acceptable regardless of
who is doing the adopting. This ADR states that general principle instead of
a media-specific one.

## Decision

1. Add a standalone, optional document,
   `docs/architecture/external-resource-adoption-contract.md`, placed flat
   under `docs/architecture/` (not a new subdirectory), sibling to
   `dependency-policy.md`. It covers AI-generated artifacts and
   human-sourced external content/data resources. It explicitly does not
   cover software packages, libraries, SDKs, CLI tools, or test helpers —
   those remain covered by `dependency-policy.md`.
2. This document extends ADR 0002 for the resources it covers. It does not
   modify or override ADR 0002's general input/output/reasoning contract for
   any AI-assisted task type, and it does not modify or override
   `dependency-policy.md`'s software-dependency adoption checklist.
3. Define a generic adoption lifecycle:
   `intake -> checked -> accepted | rejected | needs_recheck`, and
   `accepted -> adopted`. `intake` must never transition directly to
   `adopted`; an explicit, recorded check step is mandatory in between,
   performed by a deterministic tool, a human, or both — never skipped. This
   applies equally when a human is the one adopting the resource, not only
   when an AI produced it.
4. The check record states: resource source, what was checked, check method
   (deterministic tool name/version, or human reviewer plus criteria),
   verdict, and timestamp — reusing ADR 0002's source-reference and
   review-status shape rather than inventing a new schema.
5. The template does not prescribe concrete per-resource-type check
   criteria. It does not name fields like image dimensions or audio sample
   rate as required template vocabulary. Each project defines and records
   its own criteria for its own resource types. The template mandates only
   that criteria exist and are applied before a resource reaches `adopted`.
6. Whether a given criterion is checked by a deterministic tool or by a
   human is a project-specific, per-criterion decision, not a
   template-prescribed split. State only the general steer already
   established in `docs/collaboration/llm-cost-reduction.md`: prefer a
   deterministic tool when a criterion is mechanically verifiable; require a
   human when the criterion needs subjective or intent judgment; re-verify
   which bucket a given criterion falls into per project rather than
   assuming a fixed, permanent split. Whether a specific deterministic check
   actually works as expected is itself a capability-matrix concern (see ADR
   0010's verified/inferred/unknown compatibility state).
7. ADR 0010's rule that a candidate record already carrying a human verdict
   is never deleted or overwritten on resume (LISS-0008) is one
   domain-specific instance of this ADR's general adoption-check rule,
   applied to AI generation job outputs. It does not duplicate this ADR's
   schema.
8. Define an optional, generic `scope` field (for example: environment,
   tier, or dataset scope) so that a resource adopted for one scope does not
   leak into a production/trusted scope for another purpose. The template
   does not name any specific domain category as a first-class concept; an
   example row may illustrate the field without becoming template
   vocabulary.
9. `docs/architecture/io-reasoning-contracts.md`,
   `docs/architecture/dependency-policy.md`, and
   `docs/collaboration/ai-work-trace-log.md` gain, at most, a one-line
   pointer each to this document. No resource-type-specific vocabulary is
   added inline to any of them.
10. This entire document is optional: a project that never adopts an
    AI-generated or externally-sourced content/data resource beyond its
    software dependencies is not required to read or adopt it.

## Consequences

Positive:

- Closes a real gap (unreviewed output treated as usable) with a rule that
  applies uniformly regardless of who produced the resource or who is
  adopting it.
- Stays technology-neutral: no image, audio, or dataset vocabulary becomes
  required template terminology.
- Reuses ADR 0002's contract shape and `dependency-policy.md`'s adoption-
  checklist spirit instead of introducing a third, incompatible pattern.
- Clarifies the boundary with `dependency-policy.md`, avoiding overlap
  between software-dependency adoption and content/data-resource adoption.

Negative:

- Adds a new architecture document that projects with no such workload can
  ignore but must be aware exists.
- Projects adopting this contract must define their own concrete check
  criteria; the ADR defines the contract shape, not project-specific values.
- Two documents (ADR 0002's and this one) must be kept consistent over time
  as both evolve.

## Enforcement

Code review should reject:

- any adoption of an `intake` resource directly to `adopted` without a
  recorded check, by tool or by human, regardless of who is adopting it.
- resource-type-specific vocabulary (e.g. image, audio, or dataset field
  names) added to this template's core documents instead of a project's own
  architecture docs.
- a permanent, hardcoded automation-vs-human split for a named criterion,
  instead of a per-project, per-criterion decision.
- new required fields that name a specific domain/content category as
  first-class template vocabulary, instead of the generic `scope` field.
- software-dependency adoption content added here instead of
  `dependency-policy.md`, or vice versa.
