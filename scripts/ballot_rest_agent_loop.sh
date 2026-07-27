#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
iterations="${1:-1}"

if [[ ! "$iterations" =~ ^[1-5]$ ]]; then
  echo "usage: $0 [iterations: 1-5]" >&2
  exit 2
fi

command -v codex >/dev/null 2>&1 || {
  echo "codex CLI is required" >&2
  exit 127
}

if [[ "${ALLOW_DIRTY:-0}" != "1" ]] && {
  ! git -C "$repo_root" diff --quiet ||
    ! git -C "$repo_root" diff --cached --quiet ||
    [[ -n "$(git -C "$repo_root" ls-files --others --exclude-standard)" ]]
}; then
  echo "working tree is not clean; commit/stash it or set ALLOW_DIRTY=1" >&2
  exit 2
fi

for ((iteration = 1; iteration <= iterations; iteration++)); do
  report="$(mktemp "${TMPDIR:-/tmp}/ballot-rest-agent.XXXXXX")"
  trap 'rm -f "$report"' EXIT

  echo "ballot REST agent pass ${iteration}/${iterations}"
  codex exec \
    --ephemeral \
    --cd "$repo_root" \
    --sandbox workspace-write \
    --config 'approval_policy="never"' \
    --output-last-message "$report" \
    - <"$repo_root/docs/rest/agent-loop-prompt.md"

  "$repo_root/scripts/check_ballot_rest.sh"
  printf '%s\n' "agent report ${iteration}/${iterations}:"
  sed -n '1,240p' "$report"
  rm -f "$report"
  trap - EXIT
done
