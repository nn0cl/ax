# Contract File Sync Prompt

Use this when `scripts/update-ai-collaboration-files.sh` reports a file under
"NEEDS AI-ASSISTED MERGE". Those are Tier 2 agent persona/contract files
(`AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`,
`.grok/rules/*.md`, `.cursor/rules/*.mdc`) where both the adopting project and
the template changed the same file since the last sync. The script
deliberately leaves the file untouched rather than mechanically merging or
overwriting it, because these files carry adopter-filled placeholders
(project name, stack, domain boundaries, external resources) that a text
merge or a blind overwrite can silently destroy or bury.

Do not run this as a mechanical text merge. The point of routing this to an
agent instead of `git merge-file` is to distinguish "the adopter's own project
facts" from "the adopter's incidental edits" and carry only the former
forward.

## Inputs

Fill these in before starting (the sync script's output names them):

- Target repository path:
- File path (relative to repository root):
- Old ref (the template commit the target last synced against):
- New ref (the template's current commit):
- Source (template) repository path:

## Steps

1. Read the three versions of the file:
   - Target's current content (the adopter's file, as it exists now):
     `cat <target repo>/<file path>`
   - Template's old content (what the target originally synced against):
     `git -C <source repo> show <old ref>:<file path>`
   - Template's new content (what the template looks like now):
     `git -C <source repo> show <new ref>:<file path>` (or just
     `cat <source repo>/<file path>` if `<new ref>` is the source repo's
     current `HEAD`)
2. Diff the target's current content against the template's old content to
   identify what the adopter actually changed: filled placeholders (project
   name, stack, domain boundaries, external resources, persona wording),
   added project-specific notes, or removed template content on purpose.
   Treat this as the adopter's facts to preserve, not as noise to discard.
3. Diff the template's old content against its new content to see what the
   template itself changed: new sections, reworded rules, added references.
   Treat this as the structure and rules to adopt.
4. Produce a new version of the file that starts from the template's new
   content and re-applies the adopter's facts from step 2 onto it. Where a
   template change conflicts with an adopter fact (for example, the template
   restructured a section the adopter had customized), do not guess; list it
   as an open question instead of silently picking a side.
5. Show the proposed result and the list of open questions from step 4 to the
   Adjudicator for review before committing. Do not commit an AI-assisted
   contract-file merge without Adjudicator review, per
   `docs/collaboration/prompt-instruction-change-control.md`.
6. Once approved, write the file and record the change like any other
   contract-file edit: a stated reason, and an AI work trace under
   `docs/collaboration/traces/` (see
   `docs/collaboration/prompt-instruction-change-control.md`).

## Output

- Proposed file content: (write it to the target path once approved)
- Open questions needing an explicit Adjudicator decision:
- What was preserved from the adopter's version:
- What was adopted from the new template version:
