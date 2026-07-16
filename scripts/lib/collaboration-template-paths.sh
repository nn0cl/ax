# Shared path list for the AI-human collaboration template.
#
# Sourced by scripts/copy-ai-collaboration-files.sh and
# scripts/update-ai-collaboration-files.sh so both scripts stay in sync about
# which paths belong to the reusable template.

collaboration_template_paths=(
  "AGENTS.md"
  "CLAUDE.md"
  ".gitignore"
  ".github/copilot-instructions.md"
  ".grok/rules"
  ".cursor/rules"
  ".github/dependabot.yml"
  ".github/pull_request_template.md"
  ".github/ISSUE_TEMPLATE"
  ".github/workflows/ci.yml"
  "docs/architecture"
  "docs/at-tdd"
  "docs/collaboration"
  "docs/evaluation"
  "docs/issues"
  "docs/specs"
  "docs/templates"
  "docs/work-plans"
  "scripts/copy-ai-collaboration-files.sh"
  "scripts/update-ai-collaboration-files.sh"
  "scripts/init-llm-context.sh"
  "scripts/lib/collaboration-template-paths.sh"
)

# Files that are useful inside this template repository but should not be
# copied into adopting projects as target-owned planning history.
collaboration_template_exclude_paths=(
  "docs/collaboration/traces/*.md"
  "docs/issues/LISS-*.md"
  "docs/specs/template-rollout.md"
)

is_collaboration_template_excluded() {
  local rel="$1"
  local pattern
  for pattern in "${collaboration_template_exclude_paths[@]}"; do
    case "$rel" in
      $pattern) return 0 ;;
    esac
  done
  return 1
}

# Tier 2: the five agent persona/contract files that carry adopter-filled
# placeholders (project name, stack, domain boundaries, external resources).
# A mechanical text merge or blind overwrite on these can destroy or bury
# adopter-specific facts, so update-ai-collaboration-files.sh routes conflicts
# on these to AI-assisted reconciliation instead (see
# docs/templates/contract-file-sync-prompt.md) rather than to `git merge-file`
# or an unconditional overwrite. Everything else in
# collaboration_template_paths is Tier 1: the template is fully authoritative.
is_contract_persona_file() {
  local rel="$1"
  case "$rel" in
    AGENTS.md|CLAUDE.md|.github/copilot-instructions.md) return 0 ;;
    .grok/rules/*|.cursor/rules/*) return 0 ;;
    *) return 1 ;;
  esac
}
