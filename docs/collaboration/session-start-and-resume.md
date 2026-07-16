# Session Start and Resume

This guide covers how humans and AI agents begin work in a new LLM session.
It applies after the collaboration template is already present in the
repository. It does not replace `docs/collaboration/adoption-guide.md` for
first-time template installation.

For project-level startup (placeholders, first spec, domain modeling), see
`docs/collaboration/project-start-guide.md`.

## Core Idea

Each new LLM session has no prior chat context. The operating contract
(`AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`, and
`.grok/rules/*.md` when present) is reloaded from the repository, but
in-flight decisions from a previous session are not.

Continuity must come from repository artifacts:

1. handoff note or trace cited by the Adjudicator.
2. local issue or work plan.
3. accepted specification under `docs/specs/`.
4. feature branch, PR, or changed files on disk.

Do not treat chat memory, an old session summary, or README prose as
authoritative state.

## Three Session Types

### 1. First Session After Template Adoption

Use once, right after `scripts/copy-ai-collaboration-files.sh` completes.

Adjudicator:

1. Run `scripts/init-llm-context.sh <repo>` or use the Initial Assessment
   Prompt in `docs/templates/examples/adoption-prompts.md`.
2. Paste the output into the first agent session.
3. Expect assessment and placeholder identification, not implementation.

Agent:

1. Read `AGENTS.md` and `docs/architecture/agent-quickstart.md`.
2. Read `docs/collaboration/adoption-guide.md` before changing target-owned
   files.
3. Stop when target specification, phase, or project boundaries are missing.

`init-llm-context.sh` is a bootstrap aid for this session type only. CI does
not require running it.

### 2. New Session, Same Task (Resume)

Use when continuing work that already has specs, a branch, or a handoff.

Adjudicator first message should include:

- operating path: Fast, Feature, or Architecture.
- phase: Phase 0, 1, 2, or 3 when applicable.
- authoritative spec or ADR path.
- issue or work-plan ID when applicable.
- branch name.
- handoff or trace path when resuming mid-task.
- scope and out of scope.

Minimal example:

```markdown
Feature Path / Phase 2 (Green).

Spec: docs/specs/<feature>.md
Issue: docs/issues/LISS-00xx.md
Branch: feature/<name>
Handoff: <paste or path to handoff note>

Phase 1 Red is complete and reviewed. Implement the minimum code to pass the
existing tests. Do not refactor.
```

Agent:

1. Read the Adjudicator message for path, phase, spec, issue, and branch.
2. If a handoff or trace is cited, read it before other documents.
3. Recover progress from repository artifacts, not from assumed chat history.
4. If path, phase, or authoritative spec is missing, stop after design intake
   and ask the Adjudicator.

### 3. New Session, New Task

Use for a different feature or process task.

Adjudicator first message should include path, phase, spec or ADR, issue link,
branch, scope, and out of scope. No handoff is required.

Agent follows the selected operating path in
`docs/architecture/agent-quickstart.md` and reads only the documents that path
requires.

## Adjudicator Checklist

Before sending the first message in any session, confirm:

- [ ] Operating path is stated or obvious from the request type.
- [ ] Phase is stated for Feature Path work.
- [ ] An authoritative spec, ADR, or explicit Architecture Path scope exists.
- [ ] Branch and issue links are present for feature work.
- [ ] A handoff or trace is attached when resuming incomplete work.

## Agent Recovery Order

When the Adjudicator message references ongoing work, read in this order:

1. cited handoff note or trace under `docs/collaboration/traces/`.
2. cited issue or work plan.
3. cited specification or ADR.
4. branch diff or changed files if needed to confirm current state.
5. documents required by the selected operating path in agent-quickstart.

Skip documents not required by the path. Do not reread the entire repository by
default.

## Stopping Before the Next Session

When work pauses before completion, leave resumable evidence:

- use `docs/templates/agent-handoff.md` in the final response, or
- add or update a trace under `docs/collaboration/traces/` when the trace
  policy requires it.

A good handoff states current phase, completed artifacts, changed files,
verification status, blockers, and the next safe action.

## Prompt and Tooling Aids

| Situation | Aid |
|-----------|-----|
| First session after adoption | `scripts/init-llm-context.sh` |
| Deeper first assessment | `docs/templates/examples/adoption-prompts.md` |
| Daily resume or new task | This guide plus a short Adjudicator message |
| Contract reload only | No script required; contract files load per tool |

Generic chat environments that do not auto-load repository contracts still
need the Adjudicator to paste `init-llm-context.sh` output or an equivalent
first message.

## Related Documents

- Adoption: `docs/collaboration/adoption-guide.md`
- Project startup: `docs/collaboration/project-start-guide.md`
- Handoff template: `docs/templates/agent-handoff.md`
- Collaboration loop: `docs/collaboration/ai-human-scheme.md`
- Agent entry: `docs/architecture/agent-quickstart.md`