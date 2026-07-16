# Branch, Commit, and PR Discipline

This document defines Git workflow for AI-TDD collaboration.

## Branches

Create branches by feature or process task.

Recommended branch names:

```text
feature/<short-feature-name>
test/<short-behavior-name>
refactor/<short-area-name>
docs/<short-topic>
process/<short-process-topic>
chore/<short-maintenance-topic>
```

Rules:

- one branch should represent one feature, process change, or reviewable unit.
- direct pushes to `main` or the trunk branch are prohibited; all changes must
  arrive through a reviewed pull request.
- feature branches should be tied to a local issue, GitHub issue, or explicit
  Adjudicator waiver.
- work on any local issue (`docs/issues/LISS-*`) or GitHub Issue must happen on
  a dedicated branch; do not implement issue work directly on `main` or the
  trunk branch, even for a single commit.
- do not mix unrelated documentation, tests, implementation, and refactor work.
- do not start Phase 2 implementation on a branch whose Phase 1 tests have not
  been reviewed.
- branch names should describe user-visible feature or process purpose, not the
  AI tool used.
- keep branches short-lived: merge or close a branch as soon as its reviewable
  unit (one Phase, one issue, one process change) is accepted, instead of
  accumulating multiple issues or phases on one long-running branch.
- automated maintenance branches (for example, the
  `process/update-collab-template-*` branches created by
  `scripts/update-ai-collaboration-files.sh`, see
  `docs/architecture/adr/0008-template-update-propagation.md`) are exempt from
  the local/GitHub issue requirement above, but must still go through a PR and
  the CI gate before merging; they must never commit to `main` directly.

## Continuous Integration Gate

- a branch must pass CI before it merges into `main` or the trunk branch; do
  not merge on a red or skipped pipeline.
- repository hosting settings should protect `main` or the trunk branch from
  direct pushes and require the applicable pull-request checks and reviews;
  repository documents alone cannot enforce this server-side restriction.
- when PR volume or contributor count makes race conditions between merges
  likely, adopt a merge queue (or equivalent serialized-merge mechanism) so
  each merge is tested against the current state of `main` before landing.
  Which merge-queue tool to use is a stack-specific choice, not a template
  assumption.

## Parallel Agent Work (Worktrees)

When more than one agent or session works on this repository at the same
time:

- give each in-flight issue its own branch and its own `git worktree` (or
  equivalent isolated checkout) rather than sharing one working directory
  across agents.
- do not let two agents write to the same worktree/branch concurrently.
- keep the number of concurrent agent worktrees within what the Adjudicator can
  actually review; more parallel branches than the Adjudicator can review before
  they go stale defeats the point of short-lived branches.

## Stacked Branches for Phase Splitting

A single issue's Red, Green, and Refactor phases may be submitted as stacked
branches/PRs (each based on the previous phase's branch) instead of one large
PR, as long as:

- each stacked branch still targets `main` as its eventual destination and is
  still checked by the same CI/branch-protection rules as a normal PR.
- the stack order matches phase order: Red before Green before Refactor.
- the Adjudicator can tell, from the PR description, where each branch sits in the
  stack and which phase it represents.

## Commits

Prefer commits by phase:

```text
docs: add design intake for <topic>
test: add red tests for <behavior>
feat: implement <behavior>
refactor: clarify <area>
chore: update process tooling
```

Rules:

- keep commits reviewable.
- do not hide test changes inside implementation commits.
- when issue status changes, include the matching issue/documentation synchronization and any applicable work-plan update in the same reviewable unit.
- mention AI assistance in PR notes when it materially shaped the change.
- never commit secrets or full exports of private data.

## Pull Requests

PRs should identify:

- current phase.
- Adjudicator approval points.
- changed files.
- deterministic verification.
- whether tests were reviewed before implementation.
- whether AI payload included private context.
- CI status (must be passing before merge; see Continuous Integration Gate
  above).

## Feature-Unit Branch Creation

When starting a new feature:

1. create or update local issue and work plan files.
2. verify issue dependencies are resolved or waived.
3. create or update the design intake.
4. create a feature branch.
5. add Phase 1 tests only.
6. wait for Adjudicator review.
7. continue with Phase 2 on the same feature branch or a clearly linked branch.

Recommended command shape:

```text
git switch -c feature/<short-feature-name>
```

Use `docs/architecture/agent-quickstart.md` before making changes on the branch.

See `docs/collaboration/local-issue-planning.md` for local issue and dependency
rules.
