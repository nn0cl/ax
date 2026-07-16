# Grok Agent Instructions: Collaboration and Completion

## Adjudicator Interaction

When a decision affects architecture, capture it as an ADR. When a decision
is unknown, list it in the path-appropriate design note as an ambiguity
boundary.

Every request starts from design intake. Select only the AI payload context
needed for the task, define lightweight VO or DTO candidates when clear, and
route subtasks to an appropriate model, code assistant, or deterministic
tool. When AI or model output is involved, define input, output, and
reasoning evidence contracts before implementation.

Use the full `[THOUGHT]` scaffold only for Feature Path and Architecture
Path work. For Fast Path work, use a compact design note that states scope,
omitted context, deterministic checks, and why the full scaffold is
unnecessary.

## Decision Gates

Stop for Adjudicator decision when:

- phase is not explicitly selected.
- issue dependencies are unclear or unresolved.
- requirements imply a new architecture decision.
- a payload would need unrelated large context.
- a task requires secrets, full source documents, or full private data
  exports.
- an external provider, SDK, model, DB product, or schema convention must be
  chosen.
- a change would alter accepted tests.
- deterministic verification contradicts AI assumptions.

## Handoff and Completion

When handing off or stopping before completion, use
`docs/templates/agent-handoff.md`, stating current phase, completed
artifacts, next safe action, blockers, files changed, and verification
status. When asking the Adjudicator for approval, use the review points from
`docs/templates/adjudicator-review.md`.

Generated source code must minimize human cognitive load. Prefer clear
responsibility boundaries, small functions, straightforward names, and
reviewable tests. Do not compress implementation into dense code just to be
minimal.

Before reporting completion, check `docs/collaboration/definition-of-done.md`.
Create AI work traces under `docs/collaboration/traces/` when the trace
policy requires it. Use feature-unit branches for feature work; do not
implement issue work directly on `main` or the trunk branch, per
`docs/collaboration/branch-commit-pr-discipline.md`.

For feature work, identify local issue (`docs/issues/LISS-*`) or GitHub
issue dependencies before creating the branch, per
`docs/collaboration/local-issue-planning.md`.
