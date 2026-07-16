#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/update-ai-collaboration-files.sh --target PATH [options]

Pulls later AI-human collaboration template updates into a repository that
already adopted the template (via copy-ai-collaboration-files.sh).

Template files are split into two tiers:
  Tier 1 (most files): pure process/methodology documents and tooling with no
    adopter-filled placeholders. The template is fully authoritative -- if
    the target's copy differs from the template's current copy, the
    template's version wins, unconditionally.
  Tier 2 (AGENTS.md, CLAUDE.md, .github/copilot-instructions.md,
    .grok/rules/*.md, .cursor/rules/*.mdc): agent persona/contract files that
    carry adopter-filled placeholders (project name, stack, domain
    boundaries, external resources). When both the target and the template
    changed one of these since the last sync, the file is left untouched and
    flagged for AI-assisted reconciliation using
    docs/templates/contract-file-sync-prompt.md, rather than mechanically
    merged or overwritten.

A file the target deleted since the last sync, where the template changed it
again afterward, is not silently resolved either way: with an interactive
terminal, the script asks whether to restore it (default: restore); without
one (or with --non-interactive), it restores by default. The final report
lists the actual outcome for every affected file. Files listed in the
target's .collaboration-template-ignore are never touched, regardless of
tier.

This script never commits to the target's trunk branch. It creates a
dedicated branch, commits the result there, and (when possible) opens a pull
request, per docs/collaboration/branch-commit-pr-discipline.md.

Options:
  --target PATH        Target repository directory. Required.
  --source PATH         Local checkout of the template repository to pull
                        updates from. Defaults to this script's own repo.
  --branch-prefix TEXT  Branch name prefix. Default: process/update-collab-template
  --no-pr               Create the branch and commit locally; skip pushing
                        and opening a PR even if `gh` is available.
  --non-interactive     Never prompt for locally-deleted-but-upstream-changed
                        files; always take the default (restore).
  --dry-run             Report planned actions without changing anything.
  -h, --help            Show this help.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

target=""
source_repo="$repo_root"
branch_prefix="process/update-collab-template"
no_pr=false
dry_run=false
non_interactive=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --target)
      target="${2:-}"
      shift 2
      ;;
    --source)
      source_repo="${2:-}"
      shift 2
      ;;
    --branch-prefix)
      branch_prefix="${2:-}"
      shift 2
      ;;
    --no-pr)
      no_pr=true
      shift
      ;;
    --non-interactive)
      non_interactive=true
      shift
      ;;
    --dry-run)
      dry_run=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$target" ]; then
  echo "--target is required." >&2
  usage >&2
  exit 2
fi

if [ ! -d "$target" ]; then
  echo "Target directory does not exist: $target" >&2
  exit 1
fi

target="$(cd "$target" && pwd)"
source_repo="$(cd "$source_repo" && pwd)"

if ! git -C "$target" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Target is not a git repository: $target" >&2
  exit 1
fi

if ! git -C "$source_repo" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Source is not a git repository: $source_repo" >&2
  exit 1
fi

if [ "$dry_run" != true ] && [ -n "$(git -C "$target" status --porcelain)" ]; then
  echo "Target has uncommitted changes; commit, stash, or clean before syncing." >&2
  exit 1
fi

marker="$target/.collaboration-template-version"
if [ ! -f "$marker" ]; then
  echo "Missing $marker." >&2
  echo "Run scripts/copy-ai-collaboration-files.sh once to adopt the template before updating." >&2
  exit 1
fi

old_ref="$(sed -n 's/^ref:[[:space:]]*//p' "$marker" | head -n1)"
if [ -z "$old_ref" ]; then
  echo "Could not read 'ref:' from $marker." >&2
  exit 1
fi

if ! git -C "$source_repo" cat-file -e "${old_ref}^{commit}" 2>/dev/null; then
  echo "Recorded ref $old_ref is not reachable in $source_repo." >&2
  echo "Fetch full history in the source checkout and retry." >&2
  exit 1
fi

new_ref="$(git -C "$source_repo" rev-parse HEAD)"

if [ "$old_ref" = "$new_ref" ]; then
  echo "Target is already synced to $new_ref. Nothing to do."
  exit 0
fi

ignore_file="$target/.collaboration-template-ignore"
ignore_patterns=()
if [ -f "$ignore_file" ]; then
  while IFS= read -r line; do
    line="${line%%#*}"
    line="$(echo "$line" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [ -z "$line" ] && continue
    ignore_patterns+=("$line")
  done < "$ignore_file"
fi

is_ignored() {
  local rel="$1"
  local pattern
  for pattern in "${ignore_patterns[@]+"${ignore_patterns[@]}"}"; do
    # shellcheck disable=SC2254
    case "$rel" in
      $pattern) return 0 ;;
    esac
  done
  return 1
}

# shellcheck source=lib/collaboration-template-paths.sh
source "$script_dir/lib/collaboration-template-paths.sh"

added=()
updated=()
overwritten=()
needs_ai_merge=()
restored=()
kept_deleted=()
collisions=()
ignored=()
unchanged_count=0

# Whether stdin/stdout are both a real terminal, i.e. an operator is present
# to answer the restore-or-keep-deleted prompt interactively.
is_interactive_tty() {
  [ "$non_interactive" != true ] && [ -t 0 ] && [ -t 1 ]
}

# Asks whether to restore a target-deleted, template-since-changed file.
# Defaults to "restore" on empty input, non-interactive mode, or no TTY, per
# the Adjudicator's 2026-07-16 decision (LISS-0016).
ask_restore_or_keep_deleted() {
  local rel="$1"
  if is_interactive_tty; then
    local answer=""
    echo "Before deciding: check whether '$rel' was renamed rather than deleted" >&2
    echo "(e.g. to the target's own sequential ADR/local-issue number) elsewhere" >&2
    echo "in the repository." >&2
    read -r -p "Target deleted '$rel' but the template changed it since the last sync. Restore it? [Y/n] " answer || true
    case "$answer" in
      [nN]*) return 1 ;;
      *) return 0 ;;
    esac
  fi
  return 0
}

# Classifies a relative path as a numbered ADR or local-issue file. On match,
# sets numbered_class_dir/_num/_kind and returns 0; otherwise returns 1.
# Two different files "at the same number" (e.g. an unrelated project ADR
# 0007 and this template's own ADR 0007) diff as unrelated adds under plain
# path comparison, so this class needs its own collision check.
classify_numbered_file() {
  local rel="$1"
  numbered_class_dir=""
  numbered_class_num=""
  numbered_class_kind=""
  case "$rel" in
    docs/architecture/adr/[0-9][0-9][0-9][0-9]-*.md)
      numbered_class_dir="docs/architecture/adr"
      numbered_class_num="$(basename "$rel" | cut -c1-4)"
      numbered_class_kind="adr"
      return 0
      ;;
    docs/issues/LISS-[0-9][0-9][0-9][0-9]-*.md)
      numbered_class_dir="docs/issues"
      numbered_class_num="$(basename "$rel" | sed -n 's/^LISS-\([0-9][0-9][0-9][0-9]\)-.*/\1/p')"
      numbered_class_kind="liss"
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

# Prints the basename of an existing target file with the same number but a
# different slug, if any, and returns 0. Returns 1 if none is found.
find_number_collision() {
  local dir="$1" num="$2" kind="$3" own_basename="$4"
  local f bn
  [ -d "$target/$dir" ] || return 1
  for f in "$target/$dir"/*; do
    [ -f "$f" ] || continue
    bn="$(basename "$f")"
    [ "$bn" = "$own_basename" ] && continue
    case "$kind" in
      adr)
        case "$bn" in
          "$num"-*.md) echo "$bn"; return 0 ;;
        esac
        ;;
      liss)
        case "$bn" in
          LISS-"$num"-*.md) echo "$bn"; return 0 ;;
        esac
        ;;
    esac
  done
  return 1
}

# Prints the lowest unused number (zero-padded to 4 digits) in the target's
# own sequence for the given numbered-file class.
next_free_number() {
  local dir="$1" kind="$2"
  local max=0 f bn n
  if [ -d "$target/$dir" ]; then
    for f in "$target/$dir"/*; do
      [ -f "$f" ] || continue
      bn="$(basename "$f")"
      case "$kind" in
        adr) n="$(echo "$bn" | sed -n 's/^\([0-9][0-9][0-9][0-9]\)-.*/\1/p')" ;;
        liss) n="$(echo "$bn" | sed -n 's/^LISS-\([0-9][0-9][0-9][0-9]\)-.*/\1/p')" ;;
      esac
      [ -z "$n" ] && continue
      n=$((10#$n))
      [ "$n" -gt "$max" ] && max=$n
    done
  fi
  printf '%04d' $((max + 1))
}

list_files() {
  local base="$1"
  local rel="$2"
  local full="$base/$rel"
  [ -e "$full" ] || return 0
  if [ -d "$full" ]; then
    while IFS= read -r file_rel; do
      if is_collaboration_template_excluded "$file_rel"; then
        continue
      fi
      echo "$file_rel"
    done < <(cd "$base" && find "$rel" -type f)
  else
    if ! is_collaboration_template_excluded "$rel"; then
      echo "$rel"
    fi
  fi
}

process_file() {
  local rel="$1"

  if is_ignored "$rel"; then
    ignored+=("$rel")
    return
  fi

  local theirs_file="$source_repo/$rel"
  local ours_file="$target/$rel"
  local base_content=""
  local base_missing=false

  if ! base_content="$(git -C "$source_repo" show "$old_ref:$rel" 2>/dev/null)"; then
    base_missing=true
  fi

  if [ ! -e "$theirs_file" ]; then
    return
  fi

  if [ ! -e "$ours_file" ]; then
    if [ "$base_missing" = true ]; then
      added+=("$rel")
      if classify_numbered_file "$rel"; then
        local own_bn collision_bn
        own_bn="$(basename "$rel")"
        if collision_bn="$(find_number_collision "$numbered_class_dir" "$numbered_class_num" "$numbered_class_kind" "$own_bn")"; then
          local suggestion
          suggestion="$(next_free_number "$numbered_class_dir" "$numbered_class_kind")"
          collisions+=("$rel collides with existing $numbered_class_dir/$collision_bn (same number, different document) -- renumber one of them; next free number in target's sequence: $suggestion")
        fi
      fi
      if [ "$dry_run" != true ]; then
        mkdir -p "$(dirname "$ours_file")"
        cp "$theirs_file" "$ours_file"
      fi
    else
      if [ "$base_content" = "$(cat "$theirs_file")" ]; then
        : # target deleted it on purpose, upstream never changed it since: respect deletion.
      else
        if ask_restore_or_keep_deleted "$rel"; then
          restored+=("$rel")
          if [ "$dry_run" != true ]; then
            mkdir -p "$(dirname "$ours_file")"
            cp "$theirs_file" "$ours_file"
          fi
        else
          kept_deleted+=("$rel")
        fi
      fi
    fi
    return
  fi

  local ours_content theirs_content
  ours_content="$(cat "$ours_file")"
  theirs_content="$(cat "$theirs_file")"

  if [ "$base_missing" = false ] && [ "$ours_content" = "$base_content" ] && [ "$theirs_content" = "$base_content" ]; then
    unchanged_count=$((unchanged_count + 1))
    return
  fi

  if [ "$ours_content" = "$theirs_content" ]; then
    unchanged_count=$((unchanged_count + 1))
    return
  fi

  if [ "$base_missing" = false ] && [ "$ours_content" = "$base_content" ]; then
    # Target had not diverged from the template: a plain copy is safe
    # regardless of tier, since there is no adopter customization to lose.
    updated+=("$rel")
    if [ "$dry_run" != true ]; then
      cp "$theirs_file" "$ours_file"
    fi
    return
  fi

  if [ "$base_missing" = false ] && [ "$theirs_content" = "$base_content" ]; then
    unchanged_count=$((unchanged_count + 1))
    return
  fi

  # Both sides changed since the marker commit.
  if is_contract_persona_file "$rel"; then
    # Tier 2: never mechanically merge or overwrite a file that can carry
    # adopter-filled placeholders. Leave the target's current file in place
    # and flag it for AI-assisted reconciliation.
    needs_ai_merge+=("$rel")
    return
  fi

  # Tier 1: the template is fully authoritative. No merge is attempted.
  overwritten+=("$rel")
  if [ "$dry_run" != true ]; then
    mkdir -p "$(dirname "$ours_file")"
    cp "$theirs_file" "$ours_file"
  fi
}

for rel in "${collaboration_template_paths[@]}"; do
  while IFS= read -r file_rel; do
    [ -z "$file_rel" ] && continue
    process_file "$file_rel"
  done < <(list_files "$source_repo" "$rel")
done

print_list() {
  local title="$1"
  shift
  [ "$#" -eq 0 ] && return
  echo "$title"
  local item
  for item in "$@"; do
    echo "  - $item"
  done
}

echo "Source: $source_repo ($old_ref -> $new_ref)"
echo "Target: $target"
echo
print_list "Added (new upstream files):" "${added[@]+"${added[@]}"}"
print_list "Updated (Tier 1 or 2, target had not diverged from the template):" "${updated[@]+"${updated[@]}"}"
print_list "Overwritten (Tier 1, template is authoritative -- target had diverged):" "${overwritten[@]+"${overwritten[@]}"}"
print_list "NEEDS AI-ASSISTED MERGE (Tier 2 persona/contract file, both sides changed):" "${needs_ai_merge[@]+"${needs_ai_merge[@]}"}"
if [ "${#needs_ai_merge[@]}" -gt 0 ]; then
  echo "  Left untouched. Run docs/templates/contract-file-sync-prompt.md with an"
  echo "  agent for each file above, using old ref $old_ref and new ref $new_ref in"
  echo "  $source_repo, before merging this branch."
fi
print_list "Restored (was deleted locally; template changed it since last sync):" "${restored[@]+"${restored[@]}"}"
print_list "Kept deleted (operator decision):" "${kept_deleted[@]+"${kept_deleted[@]}"}"
print_list "NUMBER COLLISIONS (manual renumbering required):" "${collisions[@]+"${collisions[@]}"}"
print_list "Ignored (per .collaboration-template-ignore):" "${ignored[@]+"${ignored[@]}"}"
echo "Unchanged: $unchanged_count file(s)"

total_changes=$(( ${#added[@]} + ${#updated[@]} + ${#overwritten[@]} + ${#restored[@]} ))

if [ "$dry_run" = true ]; then
  echo
  echo "Dry run: no branch, commit, or PR created."
  exit 0
fi

if [ "$total_changes" -eq 0 ]; then
  echo
  echo "No file changes to apply; only advancing the sync marker."
fi

branch_name="${branch_prefix}-$(date +%Y%m%d)-${new_ref:0:8}"
if git -C "$target" show-ref --verify --quiet "refs/heads/$branch_name"; then
  echo "Branch $branch_name already exists in target; delete it or rerun with a different --branch-prefix." >&2
  exit 1
fi

git -C "$target" switch -c "$branch_name"

source_origin="$(git -C "$source_repo" remote get-url origin 2>/dev/null || echo "$source_repo")"
cat > "$marker" <<MARKER
# Records which commit of the AI-human collaboration template this project
# last synced against. Read by scripts/update-ai-collaboration-files.sh.
# Do not edit by hand except to correct the source.
source: $source_origin
ref: $new_ref
MARKER

git -C "$target" add -A
git -C "$target" commit -m "chore: sync collaboration template to ${new_ref:0:8}

Added: ${#added[@]}, updated: ${#updated[@]}, overwritten: ${#overwritten[@]}, needs AI-assisted merge: ${#needs_ai_merge[@]}, restored: ${#restored[@]}, kept deleted: ${#kept_deleted[@]}, number collisions: ${#collisions[@]}.
See PR description or this commit's file list for details." >/dev/null

echo
echo "Committed sync on branch $branch_name."

if [ "${#needs_ai_merge[@]}" -gt 0 ] || [ "${#collisions[@]}" -gt 0 ]; then
  echo "Manual resolution needed before merging (see NEEDS AI-ASSISTED MERGE / NUMBER COLLISIONS above)."
fi

if [ "$no_pr" = true ]; then
  echo "Skipping PR creation (--no-pr). Push and open a PR manually per docs/collaboration/branch-commit-pr-discipline.md."
  exit 0
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI not found. Push and open a PR manually:"
  echo "  git -C \"$target\" push -u origin $branch_name"
  exit 0
fi

if ! git -C "$target" remote get-url origin >/dev/null 2>&1; then
  echo "Target has no 'origin' remote. Push and open a PR manually once one is configured."
  exit 0
fi

git -C "$target" push -u origin "$branch_name"

pr_body="$(cat <<BODY
Sync from collaboration template ${old_ref:0:8} -> ${new_ref:0:8}.

- Added: ${#added[@]}
- Updated (Tier 1/2, target had not diverged): ${#updated[@]}
- Overwritten (Tier 1, template authoritative): ${#overwritten[@]}
- Needs AI-assisted merge (Tier 2 persona/contract files): ${#needs_ai_merge[@]}
- Restored (was deleted locally, template changed since): ${#restored[@]}
- Kept deleted (operator decision): ${#kept_deleted[@]}
- Number collisions needing manual renumbering: ${#collisions[@]}
- Ignored: ${#ignored[@]}

This branch follows docs/collaboration/branch-commit-pr-discipline.md: it
must pass CI before merge and should not be merged with unresolved NEEDS
AI-ASSISTED MERGE or NUMBER COLLISIONS items. For each file needing
AI-assisted merge, run docs/templates/contract-file-sync-prompt.md with an
agent (old ref ${old_ref}, new ref ${new_ref}) before merging.
BODY
)"

(cd "$target" && gh pr create --title "chore: sync collaboration template to ${new_ref:0:8}" --body "$pr_body")
