# External Resource Adoption Contract

This document is optional. It applies to projects that adopt resources
originating outside the project's own reviewed work: AI-generated artifacts
(images, audio, text, structured data, or any other generated output) and
human-sourced external content/data resources (downloaded datasets,
third-party content assets, imported files, and similar).

It does **not** cover software packages, libraries, provider SDKs, CLI
tools, or test helpers — those go through
`docs/architecture/dependency-policy.md`'s adoption checklist instead. If a
resource is a software dependency, use that document, not this one.

This document extends `docs/architecture/adr/
0002-input-output-reasoning-contracts.md` for the resources it covers. It
does not modify ADR 0002 for any other AI-assisted task type. See
`docs/architecture/adr/0011-external-resource-adoption-contract.md` for the
full decision record.

## Core Rule

No resource that originates outside the project's own reviewed work is
adopted without passing an explicit, recorded check.

This applies regardless of:

- **who produced it** — an AI generation/extraction/synthesis process, or a
  human who downloaded, copied, or otherwise brought in the resource.
- **who is adopting it** — an AI agent proposing to use the resource, or a
  human clicking "accept."
- **who performs the check** — a deterministic tool, a human reviewer, or
  both.

Skipping the check is never acceptable. A human adopting a resource without
any check is exactly as much a violation of this contract as an AI
auto-promoting its own output without one.

## Adoption Lifecycle

```text
intake -> checked -> accepted | rejected | needs_recheck
accepted -> adopted
```

- **intake**: the resource has been obtained but not yet checked.
- **checked**: an explicit check has been performed and recorded (see Check
  Record below).
- **accepted**: the check passed.
- **rejected**: the check failed; the resource is not used.
- **needs_recheck**: the check was inconclusive, or something changed since
  the last check (a new version of the resource, a changed criterion, or
  similar) and it must be checked again before use.
- **adopted**: the resource is in active, trusted use.

`intake` must never transition directly to `adopted`. There is no path that
skips `checked`.

## Check Record

Every check produces a record. Reuse ADR 0002's source-reference and
review-status shape rather than inventing a new schema:

- `source`: where the resource came from (matches ADR 0002's source
  reference shape: `source_id`, `source_type`, `uri_or_path`, `captured_at`
  when safe to expose).
- `what_was_checked`: the specific criteria evaluated.
- `check_method`: a deterministic tool (name and version) and/or a human
  reviewer plus the criteria they applied.
- `verdict`: accepted, rejected, or needs_recheck.
- `timestamp`: when the check occurred.

A `rejected` or `needs_recheck` verdict is never silently discarded — later
attempts to adopt the same or a similar resource should be able to see what
was already checked and why it did not pass.

## Check Criteria Are Project-Specific

This template does not prescribe what gets checked. No specific field names
(dimensions, sample rate, schema, license, or similar) are template
vocabulary — each project defines and records its own criteria for its own
resource types.

Illustrative examples only (not required fields):

- a project generating images might check dimensions, format, and licensing
  before adoption.
- a project importing a third-party dataset might check schema, provenance,
  and license compatibility.
- a project synthesizing text content might check length constraints,
  encoding, and source attribution.

Define the actual criteria for your project in its own architecture
documents or ADRs.

## Deterministic Tool vs. Human Check

Whether a given criterion is checked by a deterministic tool or a human is a
project-specific, per-criterion decision — this template does not fix a
permanent split. Use this general steer instead:

- prefer a deterministic tool when a criterion is mechanically verifiable.
- require a human when the criterion needs subjective or intent judgment.

Whether a specific deterministic check can actually be trusted for a given
provider/tool configuration is itself tracked using the
verified/inferred/unknown compatibility state in `docs/collaboration/
model-tool-capability-matrix.md`. Re-verify which bucket a given criterion
falls into per project rather than assuming today's split is permanent —
what is mechanically checkable can change as tooling improves.

## Optional Scope Field

A resource may carry an optional, generic `scope` field (for example:
environment, tier, or dataset scope) so that a resource adopted for one
scope does not leak into a production or trusted scope used for another
purpose. This template does not name any specific domain or content
category as a first-class concept — define the scope values that make sense
for your project.

## Relationship to Other Contracts

- `docs/architecture/adr/0010-ai-failure-recovery-and-runner-cli-contract.md`'s
  rule that a candidate record already carrying a human verdict is never
  deleted or overwritten on resume (see `docs/collaboration/
  ai-failure-recovery.md`, `docs/collaboration/runner-cli-contract.md`) is
  one domain-specific instance of this document's general adoption-check
  rule, applied to AI generation job outputs. It does not duplicate this
  document's schema.
- `docs/architecture/io-reasoning-contracts.md` defines the general
  input/output/reasoning contract for AI-assisted tasks; this document
  reuses its source-reference and review-status shape rather than replacing
  it.
- `docs/architecture/dependency-policy.md` covers software dependencies;
  this document covers everything else that enters the project from
  outside review.
