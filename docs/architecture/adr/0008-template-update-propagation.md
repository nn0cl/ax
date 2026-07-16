# ADR 0008: Pull-Based Template Update Propagation

## Status

Accepted

## Context

`scripts/copy-ai-collaboration-files.sh` only supports a one-time copy into a
new or existing repository. Once a project has adopted the template, it has
no repeatable way to bring in later template improvements (for example, ADR
0007's branching practices) without a human manually diffing every file.

A push model, where this template repository knows about every adopting
repository and opens PRs against them, would require a registry of adopting
repositories and write credentials to each one -- both are template-scope
assumptions this project avoids (see `docs/collaboration/adoption-guide.md`'s
adoption safety rules). A pull model, where each adopting repository chooses
when to sync and runs the sync locally or in its own CI, does not require the
template to know who adopted it.

The Adjudicator also raised a specific risk: a naive "copy the newest version
over the old one" update would silently overwrite or resurrect files an
adopting project intentionally customized or deleted after adoption. Any
update mechanism must preserve those local decisions by default and never
apply an unreviewed change to `main`/trunk (per ADR 0007).

## Dependency Adoption Evidence

Not applicable. The mechanism uses `git merge-file`, `git show`, and `gh`,
which the repository (or its adopters) already depends on; no new library or
service is introduced.

## Decision

1. Adoption records a version marker, `.collaboration-template-version`, at
   the target repository root, containing the source template location and
   the commit SHA the project last synced against. Both
   `scripts/copy-ai-collaboration-files.sh` (initial adoption) and
   `scripts/update-ai-collaboration-files.sh` (later syncs) write this file.
2. An adopting project may list paths in `.collaboration-template-ignore`
   (simple glob patterns, one per line) that the update script must never
   touch, regardless of upstream changes.
3. `scripts/update-ai-collaboration-files.sh` updates a target repository
   using a per-file 3-way merge: base = the file at the recorded marker
   commit, ours = the target's current file, theirs = the template's current
   file (via `git merge-file`).
   - If only the template changed the file since the marker commit, the
     target is updated automatically.
   - If only the target changed the file, the target's version is kept.
   - If both changed it, the two changes are merged; a clean merge is applied
     automatically, and an unresolved merge is written with conflict markers
     and reported, never silently resolved in either direction.
   - If the target deleted a file since the marker commit and the template
     has not changed it since, the deletion is respected silently. If the
     template *has* changed a file the target deleted, the situation is
     reported as needing a manual decision; the file is not auto-restored.
     This also fires when the target renamed the file (for example, to its
     own sequential ADR/local-issue number) rather than deleting it, since
     the script diffs by path; the reported guidance tells reviewers to
     check for a same-content match under a different filename before
     assuming a real deletion.
   - Numbered ADR (`docs/architecture/adr/NNNN-*`) and local issue
     (`docs/issues/LISS-NNNN-*`) files are a distinct class: when the
     template adds a new one, the script checks whether the target already
     has a different file under the same number. A match is reported as a
     `NUMBER COLLISION` requiring manual renumbering (with the next free
     number in the target's own sequence suggested), separate from the
     Added/Updated/Merged/Conflicts/Needs-decision categories, since a plain
     path diff cannot otherwise tell two unrelated documents "at the same
     number" apart.
4. The update script never commits to the target's trunk branch. It creates
   a dedicated branch (`process/update-collab-template-<date>-<short-sha>`),
   commits the merge result and the updated marker there, and opens a pull
   request with `gh` when available and the target has a remote, per
   `docs/collaboration/branch-commit-pr-discipline.md` and ADR 0007. A PR
   with unresolved conflicts or flagged deletions must not be merged until a
   human resolves them.
5. The template repository does not maintain a registry of adopting
   repositories and does not push updates to them.

## Tiered Sync Policy (LISS-0016, 2026-07-16)

The original per-file 3-way merge treated every template file the same way.
In practice, most template files are pure process/methodology documentation
or tooling with no adopter-fillable placeholder: `docs/collaboration/*.md`
(other than the persona files below), `docs/templates/*.md`,
`docs/at-tdd/process.md`, the shipped ADRs and CI/workflow config, and the
sync scripts themselves. Adopters are never meant to hand-edit these; running
a text-level 3-way merge on them only produces the avoidable false conflicts
this ADR already names as a downside, for files where there is nothing
adopter-specific to protect.

The Adjudicator proposed splitting template files into two tiers instead:

- **Tier 1** (everything not listed under Tier 2): the template is fully
  authoritative. If the target's copy differs from the template's current
  copy, the template's version wins outright -- no merge is attempted.
- **Tier 2**: the five agent persona/contract files that carry adopter-filled
  placeholders -- `AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`,
  `.grok/rules/*.md`, `.cursor/rules/*.mdc`. When both the target and the
  template changed one of these since the last sync, the file is left
  untouched and flagged for AI-assisted reconciliation via
  `docs/templates/contract-file-sync-prompt.md`, rather than mechanically
  merged (`git merge-file` cannot tell "adopter's project fact" apart from
  "adopter's incidental edit") or blindly overwritten (which would destroy
  the adopter's placeholder fills).

Two follow-on questions were resolved via `AskUserQuestion` (2026-07-16):

- Tier 2 mechanism: a durable prompt template plus `git show` references to
  the old and new template commits, not a throwaway generated prompt file per
  run -- consistent with this template's existing `docs/templates/*.md`
  pattern (`design-intake.md`, `agent-handoff.md`, `ai-work-trace.md`, etc.).
- Deletion handling (either tier: the target deleted a file the template
  later changed again): ask interactively when a real terminal is available
  (default: restore on empty input), default to restoring without prompting
  otherwise (or under `--non-interactive`), and report the actual per-file
  outcome at the end of the run rather than only a "needs decision" count.
  This replaces the previous "always leave it for manual resolution"
  behavior for this one case.

Numbered ADR and local-issue files the template ships (like this one) are
Tier 1: they carry no adopter placeholders, and an adopter who wants their
own decision creates a new ADR or local issue at their own next number rather
than editing a shipped one.

## Consequences

Positive:

- Adopting projects can pull in template improvements repeatedly instead of
  only at initial adoption.
- Tier 2 customizations (adopter-filled placeholders) are never silently
  overwritten or mechanically merged; Tier 1 files never produce a false
  merge conflict, since adopters are not meant to customize them at all.
- The template repository carries no list of, or credentials to, adopting
  repositories.
- The sync always goes through the same branch/PR/CI gate as any other
  change, so a bad merge cannot land without review.
- The final report names an explicit outcome (added / updated / overwritten /
  needs AI-assisted merge / restored / kept deleted / ignored / number
  collision) for every affected file, rather than only aggregate counts.

Negative:

- Adopting projects must keep a local checkout of the template repository
  with enough history to reach the marker commit; a shallow clone missing
  that commit will fail the sync with an explicit error rather than guessing.
- A Tier 1 file an adopter hand-edited despite not being meant to (for
  example, wording a process doc to fit house style) loses that edit on the
  next sync with no merge attempt and no prompt; `.collaboration-template-ignore`
  is the only escape hatch, and using it is a manual, deliberate step.
- Tier 2 conflicts are not resolved by the script at all; a sync with several
  Tier 2 conflicts requires an agent session per file before the PR can
  merge, which is slower than an automatic (if occasionally wrong) merge.
- Nothing runs this automatically; an adopting project that never re-runs the
  script simply never updates. Scheduling it (e.g., via a periodic CI job) is
  left to the adopting project.
- The default-restore behavior for locally-deleted-but-upstream-changed files
  means a project that deleted a Tier 2 file on purpose (for example, one
  that does not use Grok and removed `.grok/rules/*.md`) must either answer
  "no" interactively each time or add the path to
  `.collaboration-template-ignore` to avoid it being restored.

## Enforcement

Code review should reject:

- any change to `scripts/update-ai-collaboration-files.sh` that commits
  directly to the target's trunk branch instead of a dedicated branch.
- a sync PR merged while it still contains unresolved "NEEDS AI-ASSISTED
  MERGE" or "NUMBER COLLISIONS" items in its description.
- an AI-assisted Tier 2 merge (via `docs/templates/contract-file-sync-prompt.md`)
  committed without Adjudicator review, per
  `docs/collaboration/prompt-instruction-change-control.md`.
- removing the `.collaboration-template-ignore` honoring logic, or making the
  update script overwrite ignored paths.
- adding a registry of adopting repositories or push-based delivery to this
  template repository without superseding this ADR.
- moving a file into or out of the Tier 2 persona/contract list
  (`is_contract_persona_file` in
  `scripts/lib/collaboration-template-paths.sh`) without also updating
  `docs/collaboration/prompt-instruction-change-control.md`'s contract file
  list, since the two are meant to describe the same file set.
