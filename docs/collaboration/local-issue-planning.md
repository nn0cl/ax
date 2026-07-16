# Local Issue Planning

Issues can be managed in GitHub and as local Markdown files.

Local issue files are useful when:

- planning offline.
- preparing work before a GitHub repository is connected.
- letting AI agents reason about issue dependencies without network access.
- keeping feature-unit branch planning close to the repository.

GitHub Issues remain useful for remote collaboration, notifications, and public
review. Local issues are the repository-native planning ledger.

## Location

Store local issues under:

```text
docs/issues/
```

Store multi-issue work plans under:

```text
docs/work-plans/
```

Keep `.gitkeep` files in both folders so they exist before the first issue or
plan is created.

## Issue File Naming

Use stable local IDs:

```text
LISS-0001-short-title.md
LISS-0002-short-title.md
```

`LISS` means local issue. Do not reuse IDs.

When a GitHub Issue exists, add its number or URL in the local issue metadata.

## Required Issue Fields

Each local issue should record:

- local issue ID.
- title.
- status.
- phase.
- type.
- priority.
- initial and current planning size.
- owner or agent.
- related GitHub issue when available.
- parent issue when any.
- depends on.
- blocks.
- related branch.
- acceptance notes.
- Adjudicator decision points.
- an AI planning record when the current planning size is `M` or larger.

## Bug Planning

Record a discovered bug in a local issue or an existing work plan before
fixing it. Use exactly one durable planning artifact as the canonical record;
other artifacts should link to its issue ID or AI planning record ID rather
than copying mutable details.

A separate issue or work plan is optional only when all of these are true:

- the bug is within the current Adjudicator-approved scope.
- its planning size is `S`.
- the expected behavior is explicit in an accepted specification, an accepted
  test, or established behavior approved by the Adjudicator.
- the correction remains within one file or one feature area.
- it does not change an architecture boundary, data model, migration,
  dependency, security policy, privacy policy, or external contract.
- a deterministic verification method exists.
- the correction succeeds in one execution attempt.

This exception waives only the separate planning artifact. It never waives
design intake, test review, phase gates, branch discipline, or verification.
Record an exempt correction in the active issue or plan, commit, trace, or
durable final report with:

```text
Minor bug; fixed within approved scope; separate plan not required
```

Use the existing approved plan when the bug is already within its scope. If an
accepted test already reproduces the bug, record the Red result and obtain
Adjudicator confirmation before Phase 2. If no accepted test reproduces it, add a
regression test in Phase 1 and wait for review before Phase 2. Create a new
issue or work-plan entry when scope, expected behavior, dependencies, or
boundaries are uncertain. Record but do not mix a bug that is outside the
current scope.

## Planning Size

Planning size describes scope, uncertainty, dependencies, and verification
effort. It is not an elapsed-time estimate or delivery commitment.

| Size | Planning criteria |
| --- | --- |
| `S` | One file or one area, explicit expected behavior, local correction, and deterministic verification |
| `M` | Related changes across multiple files, a small behavior change, or more than one execution attempt |
| `L` | Multiple modules or phases, broad verification, or meaningful uncertainty |
| `XL` | Architecture boundaries, migrations, multiple dependent issues, or high uncertainty |
| `TBD` | Investigation is still required before a reliable size can be assigned |

When criteria overlap, select the largest applicable size. Preserve both the
initial and current size. Do not rewrite the initial size after work begins.
Record a reclassification reason whenever the current size changes.

At the second execution attempt, re-triage the issue. Normally reclassify an
`S` issue to at least `M`; it may remain `S` only when the repeated attempt was
caused by an unrelated external or transient failure, with the reason recorded.

## AI Planning Records

Planning-size `M`, `L`, and `XL` work requires a vendor-neutral AI planning
record in its canonical local issue or work plan. `S` work may use one
optionally, but it becomes required when a second attempt starts.

Each record has a stable ID and records:

- status.
- the authoring agent/environment.
- model and reasoning setting exactly as displayed, or `N/A` with a reason.
- creation date.
- planning size.
- intended execution route and scope.
- estimated token range, midpoint, and metric, or `N/A` with a reason.
- estimation basis, assumptions, and confidence.
- revision links and reason when another record changes the plan.

Do not silently edit another agent's accepted estimate. Append a new record,
mark the prior record `superseded`, and connect them using `Revises` and
`Superseded by`. Planning and execution may be performed by different agents;
the execution trace references the accepted planning record ID.

See `docs/collaboration/ai-work-trace-log.md` for attempt boundaries and the
conditions that make a trace mandatory.

## Dependency Rules

Use issue dependencies to define work order before implementation.

Allowed dependency meanings:

- `depends_on`: this issue should not start before the listed issue is done or
  explicitly waived.
- `blocks`: listed issues are blocked by this issue.
- `parent`: this issue is part of a larger work item.
- `related`: useful context, but not an ordering constraint.

Agents must not start work on an issue with unresolved `depends_on` entries
unless the Adjudicator explicitly waives the dependency.

Agents must not implement issue work directly on `main` or the trunk branch.
Every local issue or GitHub Issue requires a dedicated branch before any
commit for that issue is made, per
`docs/collaboration/branch-commit-pr-discipline.md`.

## Planning Flow

Before starting planned feature or bug work:

1. create or update local issues.
2. identify issue dependencies.
3. create a work plan under `docs/work-plans/`.
4. select the next unblocked issue.
5. create a feature-unit branch for that issue or feature slice.
6. run design intake.

## Status Values

Use:

- `proposed`
- `ready`
- `in_progress`
- `blocked`
- `review`
- `done`
- `wont_do`

## Phase Values

Use:

- `phase-0-design`
- `phase-1-red`
- `phase-2-green`
- `phase-3-refactor`
- `docs-only`
- `process-only`

## Synchronization with GitHub Issues

When both local and GitHub issues exist:

- keep the local issue as the detailed planning artifact.
- keep GitHub Issue title, status, and links aligned when practical.
- include the GitHub Issue URL in the local issue.
- include the local issue ID in the GitHub Issue or PR text.

Do not require GitHub network access for local planning.

## Review Rule

Adjudicator review is required when:

- issue dependencies are unclear.
- an issue is split or merged.
- work starts despite unresolved dependencies.
- the planned branch scope does not match the issue scope.
