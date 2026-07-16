# ADR 0007: Trunk-Oriented Branching for AI-TDD Collaboration

## Status

Accepted

## Context

ADR 0005 established that feature work is tied to local or GitHub issues and
gave branches a scope, but it did not say how long a branch should live, how
CI participates in the merge decision, or how multiple AI agents should share
one repository without colliding.

Industry research on high-performing teams (DORA's State of DevOps research)
and 2026 practice reports on AI-assisted and multi-agent coding converge on a
small set of practices: short-lived branches merged frequently into a single
trunk, mandatory CI before merge (with a merge queue once volume makes races
likely), isolated working directories (`git worktree`) per concurrent agent,
and stacked branches instead of one oversized PR. These practices are
compatible with this template's Red/Green/Refactor phase discipline and the
mandatory issue-per-branch rule already in ADR 0005.

Feature flags are the mechanism most sources use to make trunk-based
development safe for unfinished work, but they conflict with this template's
current assumption that nothing merges to `main` until its phase is reviewed
and complete. Adopting feature flags is therefore left as an open question,
not decided here.

## Dependency Adoption Evidence

Not applicable. This ADR does not select a library, framework, or SDK. Any
concrete merge-queue or feature-flag tool a project later adopts is a
separate, stack-specific ADR.

## Decision

1. Branches stay short-lived: a branch is merged or closed as soon as its one
   reviewable unit (one Phase, one issue, one process change) is accepted,
   rather than accumulating multiple issues or phases.
2. CI must pass before a branch merges into `main`. Projects with enough PR
   volume or contributors to see merge races should add a merge queue or
   equivalent serialized-merge mechanism; the specific tool is a stack-specific
   choice.
3. When more than one agent works on this repository concurrently, each
   in-flight issue gets its own branch and its own isolated working directory
   (`git worktree` or equivalent), never a shared working directory across
   agents.
4. A single issue's Red, Green, and Refactor phases may be split into stacked
   branches/PRs (each based on the previous phase's branch) instead of one
   large PR, provided the stack still targets `main`, still passes the same
   CI/branch-protection checks, and the phase order is preserved.
5. Feature flags for shipping incomplete work to `main` are explicitly not
   adopted by this decision. Whether to adopt them is deferred to a future ADR
   if a project's release cadence requires it.

## Consequences

Positive:

- Branch lifetime and CI expectations are explicit, closing the gap ADR 0005
  left open.
- Multiple AI agents can work on the same repository without silent
  overwrites or lock contention, using worktree isolation.
- Large issues can still be reviewed incrementally via stacked branches
  without violating the "one branch, one reviewable unit" rule.
- Feature-flag adoption stays a deliberate, reviewed decision instead of an
  assumed default.

Negative:

- Short-lived branches and mandatory CI require the project to already have
  reasonably fast, reliable automated tests; a project without them will feel
  friction until that investment is made.
- Worktree-per-agent coordination adds operational overhead (disk usage,
  cleanup of stale worktrees) that a single-agent project does not need.
- Stacked branches add review complexity (tracking stack order) compared to
  one PR per issue.

## Enforcement

Code review should reject:

- branches that stay open across multiple unrelated issues or phases.
- merges to `main` on a red or skipped CI run.
- two agents committing to the same branch/worktree concurrently.
- stacked branches submitted out of phase order (e.g. Green stacked before
  Red is reviewed).
- introduction of feature flags for shipping incomplete work without a
  dedicated ADR approving it.
