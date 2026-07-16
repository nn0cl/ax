# Prompt and Instruction Change Control

Agent behavior depends on prompt and instruction files, not only on
application code. A silent change to these files can silently change how
every AI agent behaves on this repository.

This document defines which files are the agent operating contract, and the
review and traceability rules that apply when they change.

## Agent Operating Contract Files

These files are the agent operating contract:

- `AGENTS.md`
- `CLAUDE.md`
- `.github/copilot-instructions.md`
- `.grok/rules/*.md`
- `.cursor/rules/*.mdc`
- `docs/at-tdd/process.md`
- `docs/collaboration/*.md` (except files under `docs/collaboration/traces/`)
- `docs/templates/*.md`

Files under `docs/collaboration/traces/` are records produced by following the
contract, not part of the contract itself.

New files that define agent behavior, phase rules, or cross-cutting
collaboration rules should be added to this list when they are created.

## Review Rule

A pull request that changes an agent operating contract file requires:

- explicit Adjudicator review of the change, not only automated CI.
- a stated reason for the change in the PR description.
- confirmation that `AGENTS.md`, `CLAUDE.md`,
  `.github/copilot-instructions.md`, `.grok/rules/*.md`, and
  `.cursor/rules/*.mdc` still agree with each other in effective content
  after the change, when the change touches shared phase, dependency, or
  read-order rules. Per ADR 0006 (extended by LISS-0015): agreement means
  equivalent effective content, not a literal text match — `CLAUDE.md`
  resolves through its `@AGENTS.md` import; Cursor's effective content is
  the union of `.cursor/rules/*.mdc` (Cursor complements only) and
  Cursor's native root `AGENTS.md` auto-apply (no `@AGENTS.md` inside
  `.mdc`); and `.github/copilot-instructions.md` /
  `.grok/rules/*.md` remain literal full mirrors.

Do not merge an agent operating contract change based only on an AI agent's
self-review.

## Traceability Rule

A pull request that changes an agent operating contract file must include an
AI work trace under `docs/collaboration/traces/` explaining:

- which contract file or files changed.
- why the change was needed.
- what agent behavior is expected to change as a result.

Use `docs/templates/ai-work-trace.md`. This trace is required even for small
wording changes to a contract file; the "tiny documentation-only change"
exception in `docs/collaboration/ai-work-trace-log.md` does not apply to
files in this list.

## Enforcement

CI checks that a pull request touching an agent operating contract file also
adds a trace file under `docs/collaboration/traces/`.

Code review should reject:

- agent operating contract changes without a stated reason.
- agent operating contract changes without an accompanying trace.
- agent operating contract changes that leave `AGENTS.md`, `CLAUDE.md`,
  `.github/copilot-instructions.md`, `.grok/rules/*.md`, and
  `.cursor/rules/*.mdc` inconsistent with each other in effective content.
- agent operating contract changes merged without Adjudicator review.
