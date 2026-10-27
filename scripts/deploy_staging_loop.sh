#!/usr/bin/env bash
set -euo pipefail

platform_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
web_root="${WEB_REPO:-$(cd "$platform_root/../web" 2>/dev/null && pwd || true)}"
heroku_app="${HEROKU_APP:-populist-api-staging}"
web_deploy_mode="${WEB_DEPLOY_MODE:-git}"
smoke_attempts="${1:-3}"

if [[ ! "$smoke_attempts" =~ ^[1-5]$ ]]; then
  echo "usage: $0 [smoke attempts: 1-5]" >&2
  exit 2
fi
if [[ ! -d "$web_root/.git" ]]; then
  echo "web repository not found; set WEB_REPO" >&2
  exit 2
fi
if [[ "$web_deploy_mode" != "git" && "$web_deploy_mode" != "vercel" ]]; then
  echo "WEB_DEPLOY_MODE must be git or vercel" >&2
  exit 2
fi

require_clean_tree() {
  local repo="$1"
  local name="$2"
  if [[ "${ALLOW_DIRTY:-0}" != "1" ]] &&
    [[ -n "$(git -C "$repo" status --porcelain)" ]]; then
    echo "$name working tree is not clean; commit/stash it or set ALLOW_DIRTY=1" >&2
    exit 2
  fi
}

require_clean_tree "$platform_root" "platform"
require_clean_tree "$web_root" "web"

for command_name in git curl jq heroku; do
  command -v "$command_name" >/dev/null 2>&1 || {
    echo "$command_name is required" >&2
    exit 127
  }
done

heroku auth:whoami >/dev/null

if git -C "$platform_root" diff \
  --name-only origin/main...HEAD |
  grep -Eq '(^|/)db/migrations/'; then
  echo "platform contains migrations; Heroku release phase will validate and apply them" >&2
fi

if [[ "${SKIP_PREFLIGHT:-0}" != "1" ]]; then
  "$platform_root/scripts/check_ballot_rest.sh"
fi

printf 'deploying platform %s to Heroku app %s\n' \
  "$(git -C "$platform_root" rev-parse --short HEAD)" "$heroku_app"
git -C "$platform_root" push \
  "https://git.heroku.com/${heroku_app}.git" \
  HEAD:main

# A successful build/push is not sufficient: release-phase migrations can fail.
# Do not deploy the companion web revision until this exact API release succeeds.
expected_commit="$(git -C "$platform_root" rev-parse --short=8 HEAD)"
release_ready=0
for ((check = 1; check <= 60; check++)); do
  release_json="$(heroku releases --app "$heroku_app" --num 1 --json)"
  release_description="$(jq -r 'max_by(.version).description' <<< "$release_json")"
  release_status="$(jq -r 'max_by(.version).status' <<< "$release_json")"
  if [[ "$release_description" != "Deploy $expected_commit"* ]]; then
    echo "latest release does not match the pushed revision; inspect Heroku before deploying web" >&2
    exit 1
  fi
  if [[ "$release_status" == "succeeded" ]]; then
    release_ready=1
    break
  fi
  if [[ "$release_status" == "failed" ]]; then
    echo "API release phase failed; web deployment stopped. Inspect heroku releases:output." >&2
    exit 1
  fi
  sleep 2
done
if [[ "$release_ready" != "1" ]]; then
  echo "API release is still pending; web deployment stopped" >&2
  exit 1
fi

if [[ "$web_deploy_mode" == "git" ]]; then
  if [[ "$(git -C "$web_root" branch --show-current)" != "main" ]] &&
    [[ "${ALLOW_NON_MAIN_WEB:-0}" != "1" ]]; then
    echo "web must be on main for the documented staging trigger" >&2
    echo "set ALLOW_NON_MAIN_WEB=1 only when intentionally deploying that ref" >&2
    exit 2
  fi
  printf 'triggering Vercel staging from web %s\n' \
    "$(git -C "$web_root" rev-parse --short HEAD)"
  git -C "$web_root" push origin HEAD:main
else
  command -v vercel >/dev/null 2>&1 || {
    echo "vercel CLI is required for WEB_DEPLOY_MODE=vercel" >&2
    exit 127
  }
  vercel_scope="${VERCEL_SCOPE:-populist}"
  vercel_project="${VERCEL_PROJECT:-web}"
  staging_domain="${WEB_STAGING_DOMAIN:-staging.populist.us}"

  (
    cd "$web_root"
    vercel link \
      --yes \
      --scope "$vercel_scope" \
      --project "$vercel_project" >/dev/null
    deployment_url="$(
      vercel deploy --yes --scope "$vercel_scope" |
        grep -Eo 'https://[^[:space:]]+\.vercel\.app' |
        tail -1
    )"
    if [[ -z "$deployment_url" ]]; then
      echo "Vercel did not return a deployment URL" >&2
      exit 1
    fi
    vercel alias set \
      "$deployment_url" \
      "$staging_domain" \
      --scope "$vercel_scope"
  )
fi

for ((attempt = 1; attempt <= smoke_attempts; attempt++)); do
  printf 'staging smoke pass %s/%s\n' "$attempt" "$smoke_attempts"
  if WAIT_ATTEMPTS="${WAIT_ATTEMPTS:-30}" \
    WAIT_SECONDS="${WAIT_SECONDS:-10}" \
    "$platform_root/scripts/check_staging_rest.sh"; then
    exit 0
  fi
  if ((attempt < smoke_attempts)); then
    sleep "${SMOKE_RETRY_SECONDS:-20}"
  fi
done

echo "staging smoke checks failed after $smoke_attempts passes" >&2
exit 1
