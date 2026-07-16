# Collaboration Template Adoption Guide

Use this guide after copying the collaboration template into a new or existing
repository. It is intentionally separate from the target repository README so
midway adoption does not overwrite product documentation.

For the benefits and tradeoffs of using the template, see
`docs/collaboration/template-benefits.md`.

## New Repository Adoption

1. Run `scripts/copy-ai-collaboration-files.sh --target <repo>`.
2. Fill target-specific placeholders in `AGENTS.md`, `CLAUDE.md`,
   `.github/copilot-instructions.md`, `.grok/rules/*.md`,
   `.cursor/rules/*.mdc`, and `docs/architecture/README.md`. The copy
   script can fill project name, domain summary, and stack placeholders;
   runtime boundaries, datastore, migration tool, external resources, and
   stack-specific architecture documents still require Adjudicator-approved
   target facts.
3. Add the first target feature specification under `docs/specs/`.
4. Add only the stack-specific architecture documents that the project already
   needs.
5. Read `docs/collaboration/project-start-guide.md` for the first development
   loop.
6. Run `scripts/init-llm-context.sh <repo>` and paste the generated prompt into
   the first agent session.
7. Read `docs/collaboration/session-start-and-resume.md` for ongoing session
   start and resume patterns after adoption.

## Midway Adoption

1. Run `scripts/copy-ai-collaboration-files.sh --target <repo> --dry-run`.
2. Run the copy without `--force` so existing target files remain unchanged.
3. Review skipped files and decide manually whether any target-owned document
   should adopt collaboration wording.
4. Keep accepted target architecture and feature specifications authoritative.
5. Use Fast Path for mechanical adoption cleanup, Feature Path for accepted
   feature work, and Architecture Path for process or boundary decisions.

## Receiving Later Template Updates

Adoption via `scripts/copy-ai-collaboration-files.sh` records a
`.collaboration-template-version` marker at the target repository root. Use
that marker to pull in later template improvements without losing target
customizations:

The template repository keeps its own maintenance local issues, traces, and
sample rollout specification for audit history. Copy and update scripts
exclude those files from adopting projects so target repositories start
with their own empty issue, trace, and spec ledgers.

1. Update your local checkout of this template repository (`git pull` or
   equivalent) so it has the commit you want to sync to.
2. Optionally list paths the target has intentionally diverged from in
   `.collaboration-template-ignore` (simple glob patterns, one per line).
3. Run `scripts/update-ai-collaboration-files.sh --target <repo>` (add
   `--non-interactive` for unattended/CI runs; see below for what that
   changes).
4. Review the reported summary. Template files are split into two tiers (see
   ADR 0008): **Tier 1** (most files -- process docs, templates, shipped
   ADRs, CI/scripts) is fully template-authoritative, so a differing file is
   reported as **Overwritten** with no merge attempt. **Tier 2** (the five
   agent persona/contract files: `AGENTS.md`, `CLAUDE.md`,
   `.github/copilot-instructions.md`, `.grok/rules/*.md`,
   `.cursor/rules/*.mdc`) is never mechanically merged or overwritten; a
   conflicting Tier 2 file is reported as **NEEDS AI-ASSISTED MERGE** and left
   untouched. Other categories: **Added** (new upstream files), **Updated**
   (either tier, target had not diverged), **Restored** / **Kept deleted**
   (the target had deleted a file the template changed again -- interactive
   prompt with a default of restoring; see below), and **NUMBER COLLISIONS**
   (a newly added ADR or local issue shares a number with an existing target
   file under a different name).
5. For every file under NEEDS AI-ASSISTED MERGE, run
   `docs/templates/contract-file-sync-prompt.md` with an agent (the script's
   output names the old ref, new ref, and file path to use) before merging.
   Resolve any NUMBER COLLISIONS by renumbering. Never merge a sync PR with
   unresolved NEEDS AI-ASSISTED MERGE or NUMBER COLLISIONS items.
6. If the sync introduces new cross-cutting process vocabulary (for example,
   a new operating-path or phase concept), check whether the target's own
   `CLAUDE.md`/`AGENTS.md` needs a matching, reviewed update so the newly
   imported docs describe a concept the target's agent contract actually
   uses. A project that customized `CLAUDE.md` before that vocabulary
   existed will not get it added automatically, and imported docs that
   reference an unused concept are worse than no docs.

### Recovering When a Repository Predates the Marker

`scripts/update-ai-collaboration-files.sh` requires
`.collaboration-template-version` at the target root, written by
`scripts/copy-ai-collaboration-files.sh`. Some repositories adopted this
template's content before that script existed, or via manual copy/paste, and
have no such marker and no adoption commit to derive one from.

Do not fabricate a `ref:` value without checking it. Instead:

1. Pick a handful of representative shared files that are unlikely to have
   been hand-edited by the target (for example, one ADR under
   `docs/architecture/adr/`, `docs/templates/design-intake.md`).
2. Walk the template's own git history (oldest first) and, for each
   candidate commit, `git show <commit>:<path>` those same files.
3. Find the oldest template commit whose content for those files matches (or
   nearly matches, accounting for placeholder fills) the target's own oldest
   version of the same files. Confirm with `diff`, not by inspection alone.
4. Once confirmed, hand-write `.collaboration-template-version` with that
   commit as `ref:` and commit it on its own (no unrelated changes in the
   same commit), before running the update script for the first time.

If no reasonable match exists (the target's content has diverged too far to
identify a common ancestor), treat the target as a from-scratch adoption
instead: review `docs/collaboration/template-benefits.md` and the current
template content manually rather than running the automated sync.

The script never commits to the target's trunk branch; it creates a
dedicated branch and opens a PR, per
`docs/collaboration/branch-commit-pr-discipline.md` and
`docs/architecture/adr/0008-template-update-propagation.md`. It does not
clone or register repositories on its own, and this template repository does
not track which projects have adopted it.

## LLM Session Setup

Use `scripts/init-llm-context.sh <repo>` once to print a compact first prompt
after adoption. For daily new sessions and resuming work, see
`docs/collaboration/session-start-and-resume.md` instead of rerunning the
script.

For more detailed, project-neutral prompt examples, see
`docs/templates/examples/adoption-prompts.md`.

The first-session prompt instructs the agent to:

- read `AGENTS.md` and `docs/architecture/agent-quickstart.md`.
- select Fast Path, Feature Path, or Architecture Path.
- avoid introducing target stack, datastore, provider, or domain decisions
  without an accepted specification or ADR.
- stop when the target specification or requested phase is missing.

Grok Build discovers `.grok/rules/*.md` as a distinct, stronger-binding rules
surface (visible via `grok inspect`) separate from generic context loading;
keep it in sync with `AGENTS.md`/`CLAUDE.md` like any other contract file.
As of 2026, Grok Build also reads `AGENTS.md` (at global, repo-root, and
cwd levels) and `CLAUDE.md` natively as a fallback, but `.grok/rules/*.md`
remains the stronger-binding surface, so keep both in sync.

Cursor discovers `.cursor/rules/*.mdc` (files must use the `.mdc` extension
with frontmatter — a plain `.md` file in `.cursor/rules/` is ignored by
Cursor's rules system) as its primary, most powerful rules mechanism; this
template sets `alwaysApply: true` on each file so the rules apply to every
request regardless of which files are open. As of LISS-0015 (2026-07-16,
Adjudicator-approved after live verification the same day), `.cursor/rules/*.mdc`
holds Cursor-complementary rules only and does not `@`-reference or
full-mirror shared sections from `AGENTS.md`. Grounds: Cursor lists
`AGENTS.md` as its own Rules type and "picks it up automatically"
([Rules](https://cursor.com/docs/rules.md);
[Help: Rules](https://cursor.com/help/customization/rules.md)); live session
confirmed separate injection of root `AGENTS.md` alongside always-apply
`.mdc` files — see ADR 0006 and
`docs/collaboration/traces/2026-07-16-cursor-mdc-drop-agents-ref.md`. Keep
the `.mdc` set for phase-gate detail, Decision Gates, and other Cursor-side
complements rather than relying on `AGENTS.md` alone.

Codex reads `AGENTS.md` directly (its own `~/.codex/rules/` is a user-home
setting, not a project-distributable one), so it needs no dedicated
template file.

Claude Code supports `@path/to/file` imports (expanded inline into context at
launch) and its own `.claude/rules/*.md` directory with `paths:`
frontmatter, equivalent to Cursor's `globs`. `CLAUDE.md` uses `@AGENTS.md` to
avoid duplicating `AGENTS.md`'s content, per Anthropic's own documented
recommendation for this exact purpose — see ADR 0006.

## Adding Stack-Specific Scoped Rules

Once a stack ADR is accepted, a project may need rules that apply only to a
specific area (for example, frontend conventions that should not load when an
agent is only touching backend code). Add these as new files, scoped so they
load only for matching paths, rather than growing the always-applying
contract files:

- **Cursor**: a new `.cursor/rules/<topic>.mdc` file with a non-empty `globs`
  field (for example, `globs: src/frontend/**`) and `alwaysApply: false`.
- **Claude Code**: a new `.claude/rules/<topic>.md` file with a `paths:`
  frontmatter list of glob patterns.
- **GitHub Copilot**: a new `.github/instructions/<topic>.instructions.md`
  file with an `applyTo` frontmatter glob; it combines with
  `.github/copilot-instructions.md` rather than replacing it.
- **Grok / Codex**: no confirmed path-scoped rule mechanism as of 2026-07-16;
  keep stack-specific rules for these tools inside the existing full-mirror
  files, scoped by a heading that states which area they apply to.

Add each new scoped-rule file to
`docs/collaboration/prompt-instruction-change-control.md`'s contract file
list when it is created, and follow the same trace/review requirements as any
other contract-file change.

## Cost and Reasoning Control

Use `docs/collaboration/llm-cost-reduction.md` to keep LLM cost control
observable without adding heavy process. For substantial tasks, traces should
record the selected operating path, omitted context, deterministic checks, and
any escalation reason.

## Adoption Safety Rules

- Do not use this template to replace target project architecture.
- Do not ship target-specific domain models as part of the reusable template.
- Do design the target domain model after adoption, through target specs,
  reviewed tests, Adjudicator decisions, and ADRs.
- Do not use `--force` unless the target owner has reviewed each overwrite.
- Do not treat placeholder examples as selected technology.
- Keep target secrets, private data, and full exports out of AI prompts.
- Prefer deterministic checks over AI reasoning for copy verification,
  formatting, parsing, and CI sanity.
