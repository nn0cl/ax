#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/init-llm-context.sh [TARGET_REPOSITORY]

Prints a compact initial prompt for an LLM agent after this collaboration
template has been copied into a repository. The script does not call an LLM,
read secrets, or make project architecture decisions.
USAGE
}

target="${1:-.}"

case "$target" in
  -h|--help)
    usage
    exit 0
    ;;
esac

if [ ! -d "$target" ]; then
  echo "Target directory does not exist: $target" >&2
  exit 1
fi

target="$(cd "$target" && pwd)"

required_files=(
  "AGENTS.md"
  "CLAUDE.md"
  ".github/copilot-instructions.md"
  ".grok/rules/01-quickstart.md"
  ".grok/rules/02-architecture-boundaries.md"
  ".grok/rules/03-collaboration-and-completion.md"
  "docs/architecture/agent-quickstart.md"
  "docs/at-tdd/process.md"
  "docs/collaboration/ai-human-scheme.md"
  "docs/architecture/ai-request-routing.md"
  "docs/architecture/io-reasoning-contracts.md"
  "docs/architecture/implementation-readiness.md"
)

missing=false
for rel in "${required_files[@]}"; do
  if [ ! -f "$target/$rel" ]; then
    echo "Missing required file: $rel" >&2
    missing=true
  fi
done

if [ "$missing" = true ]; then
  echo "Install the template files before generating the LLM setup prompt." >&2
  exit 1
fi

cat <<PROMPT
You are working in this repository:
$target

Before implementing anything:
1. Read AGENTS.md.
2. Read docs/architecture/agent-quickstart.md.
3. Select the smallest safe operating path:
   - Fast Path for mechanical, local, deterministic work.
   - Feature Path for AT-TDD Phase 1, 2, or 3 feature work.
   - Architecture Path for ADRs, process changes, prompt changes, privacy-sensitive routing, or boundary decisions.
4. Read only the documents required by that path.
5. Read docs/architecture/io-reasoning-contracts.md when AI or model output is involved.
6. Check docs/architecture/implementation-readiness.md before Phase 1, 2, or 3.

Use a compact design note for Fast Path work. Use the full [THOUGHT] scaffold
for Feature Path and Architecture Path work.
Execute only the phase explicitly requested by the human Adjudicator.
Do not introduce target-project domain behavior, datastore choices, provider
choices, or stack-specific architecture unless an accepted specification or ADR
requires it.

If no target specification or phase has been provided yet, stop after design
intake and ask the Adjudicator for the missing specification or phase approval.

For later sessions and resume patterns, see
docs/collaboration/session-start-and-resume.md.
PROMPT
