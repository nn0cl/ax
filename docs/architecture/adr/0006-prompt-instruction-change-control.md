# ADR 0006: Prompt and Instruction Change Control

## Status

Accepted

## Context

`AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`,
`.grok/rules/*.md`, and `.cursor/rules/*.mdc` are near-duplicate operating
contracts for different AI coding tools, together with
`docs/at-tdd/process.md`, `docs/collaboration/*.md`, and
`docs/templates/*.md`. Agent behavior depends directly on these files.
(Codex reads `AGENTS.md` directly and does not need its own contract file.)

As of 2026, several of these tools also read `AGENTS.md` (and in Grok
Build's case, `CLAUDE.md` too) natively, independent of their own dedicated
rule surface: Cursor documents `AGENTS.md` as a "simple alternative to
`.cursor/rules`", and current Grok Build documentation states it reads
`AGENTS.md` at three levels (`~/.grok/AGENTS.md`, `<repo-root>/AGENTS.md`,
`<cwd>/AGENTS.md`) plus `CLAUDE.md` "for compatibility" (verified via live
web search, 2026-07-14; see the accompanying trace).

LISS-0006 and LISS-0010 originally resolved this with one blanket rule: full
mirror across all five files, no thin pointers, so every tool gets the same
explicit, strongly-bound entry point. LISS-0015 (2026-07-16) revisited that
blanket rule on Adjudicator instruction, on the grounds that "we decided this
once before" is not itself evidence, and found the picture differs per
vendor:

- **GitHub Copilot** now reads `AGENTS.md` natively (coding agent since
  2025-08-28, code review GA since 2026-06-18), but GitHub's own
  documentation states this reading is "read-and-apply, not strict
  enforcement" and does not guarantee adherence as literal as Claude Code
  following `CLAUDE.md`. This is the same category of evidence that
  originally justified Grok's dedicated file (generic context loading proving
  insufficient in practice) applied to a different vendor.
- **Claude Code** supports `@path/to/file` imports that expand inline into
  context at session launch — a guaranteed content-inlining mechanism, not a
  hope-based pointer — and Anthropic's own documentation explicitly
  recommends `@AGENTS.md` specifically to avoid duplicating instructions
  between `AGENTS.md` and `CLAUDE.md`. This is a materially different
  mechanism from a plain-text cross-reference, and removes the original
  "thin pointers aren't reliable" objection for this one vendor pair.
- **Cursor** — shared contract via native `AGENTS.md`, not `@` inside `.mdc`.

  Evidence (primary sources, fetched 2026-07-16):

  1. Rules types list `AGENTS.md` separately from Project Rules
     (`.cursor/rules`): [Rules | Cursor Docs](https://cursor.com/docs/rules.md)
     ("Cursor supports four types of rules: … Project Rules … AGENTS.md").
  2. Help: "Create an `AGENTS.md` file in your project root. …
     Cursor picks it up automatically."
     ([Help: Rules](https://cursor.com/help/customization/rules.md)).
  3. Nested `AGENTS.md` is "automatically applied when working with files in
     that directory" ([Rules § AGENTS.md](https://cursor.com/docs/rules.md)).
  4. FAQ: `@filename` in a rule includes that file in rule context
     ([Rules FAQ](https://cursor.com/docs/rules.md)) — valid, but for root
     `AGENTS.md` it duplicates (1)–(2) rather than substituting for them.
  5. Live Cursor session 2026-07-16 (this repo, branch
     `process/agent-rule-file-parity`): agent context received root
     `AGENTS.md` as its own always-applied workspace rule *and* the three
     `alwaysApply` `.mdc` files; `@AGENTS.md` prose inside `.mdc` bodies was
     not expanded inline. Shared Expected Workflow / dependency-rule content
     was present via the `AGENTS.md` injection. Trace:
     `docs/collaboration/traces/2026-07-16-cursor-mdc-drop-agents-ref.md`.

  Conclusion: omitting shared sections from `.mdc` (and omitting `@AGENTS.md`
  there) does not drop them from Cursor Agent context while root `AGENTS.md`
  auto-apply remains in force. Keep `.mdc` for Cursor-only complements.
- **Grok**'s `.grok/rules/` stronger-binding finding (LISS-0006's live `grok
  inspect` test, 2026-07-08) was not re-examined this round.

Decision, per vendor (Adjudicator-confirmed 2026-07-16; Cursor policy refined
and **Adjudicator-approved** same day after live verification + cited grounds):

- `CLAUDE.md` now imports `AGENTS.md` (`@AGENTS.md`) instead of duplicating
  its body, keeping only genuinely Claude Code-specific sections.
- `.cursor/rules/*.mdc` keeps only Cursor-complementary content (phase gate
  detail, anti-hallucination, Decision Gates, handoff/completion). Shared
  sections formerly duplicated from `AGENTS.md` are omitted — not
  `@`-referenced — because Cursor already auto-applies root `AGENTS.md`
  (evidence items 1–5 above).
- `.github/copilot-instructions.md` and `.grok/rules/*.md` keep the full
  mirror. For Copilot, the Adjudicator weighed GitHub's documented weaker-adherence
  risk against the duplication cost and chose to keep the stronger, dedicated
  binding. For Grok, the original empirical grounding was not revisited.

These files can drift from each other silently: one file can gain a required
read step that the others do not, and none of them require the
operating-contract files themselves to be reviewed with the same rigor as
application code. The AI Work Trace Log already asks for a trace when
contract files change, but that alone does not name the exact file set,
require Adjudicator review specifically, or get enforced by CI.

This gap is tracked in `docs/collaboration/process-gap-register.md`.

## Decision

Adopt `docs/collaboration/prompt-instruction-change-control.md` as the
canonical definition of the agent operating contract file set.

- Name the exact files and glob patterns that count as the agent operating
  contract.
- Require Adjudicator review, a stated reason, and a cross-file consistency
  check whenever a contract file changes.
- Require an AI work trace under `docs/collaboration/traces/` for every
  contract change, including small wording changes.
- Enforce the trace requirement in CI: a pull request that changes a
  contract file must also add a trace file.
- Per LISS-0015: the consistency check means the five files resolve to
  equivalent effective content, not that they are literal duplicates.
  `CLAUDE.md` resolves through its `@AGENTS.md` import; `.cursor/rules/*.mdc`
  plus Cursor's native root `AGENTS.md` loading together supply the shared
  contract; `copilot-instructions.md` and `.grok/rules/*.md` remain literal
  full mirrors.

## Consequences

Positive:

- Contract drift between `AGENTS.md`, `CLAUDE.md`,
  `.github/copilot-instructions.md`, `.grok/rules/*.md`, and
  `.cursor/rules/*.mdc` becomes visible in review instead of silently
  changing agent behavior.
- Every contract change has a recorded reason and expected behavior change.
- CI gives an automated signal instead of relying only on Adjudicator memory.
- `CLAUDE.md`'s `@AGENTS.md` import removes one full hand-maintained
  duplicate; a future change to `AGENTS.md`'s imported sections no longer
  needs a matching manual edit in `CLAUDE.md`.
- Cursor `.mdc` files no longer carry redundant `@AGENTS.md` references or
  full shared-section mirrors; shared content rides on native `AGENTS.md`
  auto-apply.

Negative:

- Adds a mandatory trace step even for small wording changes to contract
  files.
- Requires keeping the file list in
  `docs/collaboration/prompt-instruction-change-control.md` up to date as new
  contract-like files are introduced.
- The consistency check can no longer be a simple text diff for `CLAUDE.md`
  or for Cursor (`.mdc` + native `AGENTS.md`); a reviewer must confirm the
  effective union still matches `AGENTS.md`, which is a judgment call rather
  than a byte comparison.
- If Cursor ever stopped auto-applying root `AGENTS.md`, shared rules would
  disappear from Cursor sessions unless `.mdc` or another binding were
  restored — watch product docs when upgrading Cursor.

## Enforcement

Code review should reject:

- agent operating contract changes without a stated reason or Adjudicator review.
- agent operating contract changes without an accompanying trace under
  `docs/collaboration/traces/`.
- agent operating contract changes that leave `AGENTS.md`, `CLAUDE.md`,
  `.github/copilot-instructions.md`, `.grok/rules/*.md`, and
  `.cursor/rules/*.mdc` inconsistent with each other in effective content
  (literal text for `copilot-instructions.md` and `.grok/rules/*.md`;
  resolved content via `@AGENTS.md` for `CLAUDE.md`; effective union of
  `.cursor/rules/*.mdc` plus native root `AGENTS.md` for Cursor).

CI should reject:

- a pull request that changes a file listed in
  `docs/collaboration/prompt-instruction-change-control.md` without adding a
  trace file under `docs/collaboration/traces/`.
