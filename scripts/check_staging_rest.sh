#!/usr/bin/env bash
set -euo pipefail

api_base="${API_BASE_URL:-https://api.staging.populist.us}"
web_base="${WEB_BASE_URL:-https://staging.populist.us}"
wait_attempts="${WAIT_ATTEMPTS:-30}"
wait_seconds="${WAIT_SECONDS:-10}"

api_base="${api_base%/}"
web_base="${web_base%/}"

if [[ ! "$wait_attempts" =~ ^[1-9][0-9]*$ ]] ||
  [[ ! "$wait_seconds" =~ ^[0-9]+$ ]]; then
  echo "WAIT_ATTEMPTS must be positive and WAIT_SECONDS must be non-negative" >&2
  exit 2
fi

for command_name in curl jq; do
  command -v "$command_name" >/dev/null 2>&1 || {
    echo "$command_name is required" >&2
    exit 127
  }
done

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/populist-staging-smoke.XXXXXX")"
trap 'rm -r "$work_dir"' EXIT

curl_with_auth() {
  if [[ -n "${POPULIST_API_KEY:-}" ]]; then
    command curl -H "Authorization: Bearer ${POPULIST_API_KEY}" "$@"
  else
    command curl "$@"
  fi
}

wait_for_200() {
  local name="$1"
  local url="$2"
  local body_file="$3"
  local header_file="$4"
  local response_code=""

  for ((attempt = 1; attempt <= wait_attempts; attempt++)); do
    response_code="$(
      curl_with_auth --silent --show-error \
        --connect-timeout 10 \
        --max-time 60 \
        -D "$header_file" \
        -o "$body_file" \
        -w "%{http_code}" \
        "$url" ||
        true
    )"
    if [[ "$response_code" == "200" ]]; then
      printf 'ok: %s\n' "$name"
      return 0
    fi

    printf 'waiting: %s returned HTTP %s (%s/%s)\n' \
      "$name" "${response_code:-curl_error}" "$attempt" "$wait_attempts"
    if ((attempt < wait_attempts)); then
      sleep "$wait_seconds"
    fi
  done

  echo "failed: $name never returned HTTP 200" >&2
  sed -n '1,80p' "$body_file" >&2 || true
  return 1
}

api_index="$work_dir/api-index.json"
api_index_headers="$work_dir/api-index.headers"
wait_for_200 \
  "REST API index" \
  "$api_base/api/v1/" \
  "$api_index" \
  "$api_index_headers"
jq -e '
  .data.version == "v1" and
  .data.health == "/api/v1/health" and
  .data.states == "/api/v1/states" and
  .data.ballotByAddress == "/api/v1/elections/{electionId}/ballot"
' "$api_index" >/dev/null
grep -Eqi '^x-request-id: .+' "$api_index_headers"

health="$work_dir/health.json"
health_headers="$work_dir/health.headers"
wait_for_200 \
  "REST health" \
  "$api_base/api/v1/health" \
  "$health" \
  "$health_headers"
jq -e '.data.status == "ok" and .data.apiVersion == "v1"' "$health" >/dev/null

states="$work_dir/states.json"
states_headers="$work_dir/states.headers"
wait_for_200 \
  "REST states" \
  "$api_base/api/v1/states?limit=5&offset=0" \
  "$states" \
  "$states_headers"
jq -e '
  (.data | type == "array") and
  (.data | length == 5) and
  .meta.limit == 5 and
  .meta.offset == 0 and
  (.meta.total | type == "number")
' "$states" >/dev/null

election_id="${BALLOT_ELECTION_ID:-}"
if [[ -z "$election_id" ]]; then
  election_response="$work_dir/elections.json"
  # GraphQL variable names are intentionally literal shell dollar expressions.
  # shellcheck disable=SC2016
  curl_with_auth --silent --show-error --fail-with-body \
    --connect-timeout 10 \
    --max-time 60 \
    -H "Content-Type: application/json" \
    --data '{
      "query": "query StagingElections($filter: ElectionFilter) { elections(filter: $filter) { id state electionDate title } }",
      "variables": { "filter": { "state": "MN" } }
    }' \
    "$api_base/" >"$election_response"
  jq -e '.errors == null and (.data.elections | type == "array")' \
    "$election_response" >/dev/null
  election_id="$(
    jq -r '
      [.data.elections[] | select(.state == "MN")]
      | sort_by(.electionDate)
      | last
      | .id // empty
    ' "$election_response"
  )"
fi

if [[ ! "$election_id" =~ ^[0-9a-fA-F-]{36}$ ]]; then
  echo "No Minnesota election UUID was available; set BALLOT_ELECTION_ID" >&2
  exit 1
fi
printf 'using election: %s\n' "$election_id"

ballot_request="$work_dir/ballot-request.json"
cat >"$ballot_request" <<'JSON'
{
  "address": {
    "line1": "350 S 5th St",
    "city": "Minneapolis",
    "state": "MN",
    "postalCode": "55415",
    "country": "US"
  }
}
JSON

ballot="$work_dir/ballot.json"
ballot_headers="$work_dir/ballot.headers"
ballot_code="$(
  curl_with_auth --silent --show-error \
    --connect-timeout 10 \
    --max-time 120 \
    -D "$ballot_headers" \
    -o "$ballot" \
    -w "%{http_code}" \
    -H "Content-Type: application/json" \
    -H "X-Request-Id: staging-smoke-ballot" \
    --data-binary "@$ballot_request" \
    "$api_base/api/v1/elections/$election_id/ballot" ||
    true
)"
if [[ "$ballot_code" != "200" ]]; then
  echo "Ballot lookup returned HTTP $ballot_code" >&2
  sed -n '1,120p' "$ballot" >&2 || true
  exit 1
fi
jq -e --arg election_id "$election_id" '
  .data.election.id == $election_id and
  (.data.races | type == "array") and
  (.data.ballotMeasures | type == "array") and
  (.data.coverage.races | IN("address_specific", "statewide_only")) and
  (.data.coverage.ballotMeasures | IN("address_specific", "statewide_only")) and
  ([paths | map(tostring) | join(".") | select(test("address"; "i"))] | length == 0)
' "$ballot" >/dev/null
grep -Eqi '^cache-control: no-store' "$ballot_headers"
grep -Eqi '^x-request-id: staging-smoke-ballot' "$ballot_headers"
printf 'ok: ballot lookup and address privacy\n'

invalid_request="$work_dir/invalid-request.json"
cat >"$invalid_request" <<'JSON'
{
  "address": {
    "line1": "350 S 5th St",
    "city": "Minneapolis",
    "state": "MN",
    "postalCode": "not-a-zip"
  }
}
JSON

invalid_response="$work_dir/invalid-response.json"
invalid_headers="$work_dir/invalid.headers"
invalid_code="$(
  curl_with_auth --silent --show-error \
    --connect-timeout 10 \
    --max-time 60 \
    -D "$invalid_headers" \
    -o "$invalid_response" \
    -w "%{http_code}" \
    -H "Content-Type: application/json" \
    --data-binary "@$invalid_request" \
    "$api_base/api/v1/elections/$election_id/ballot" ||
    true
)"
[[ "$invalid_code" == "422" ]]
jq -e '.status == 422 and .code == "invalid_address"' \
  "$invalid_response" >/dev/null
grep -Eqi '^content-type: application/problem\+json' "$invalid_headers"
printf 'ok: invalid address problem response\n'

rest_docs="$work_dir/rest-docs.html"
rest_docs_headers="$work_dir/rest-docs.headers"
wait_for_200 \
  "REST overview documentation" \
  "$web_base/docs/api/rest" \
  "$rest_docs" \
  "$rest_docs_headers"
grep -Fq "REST API" "$rest_docs"
grep -Fq "Ballot by Address" "$rest_docs"

ballot_docs="$work_dir/ballot-docs.html"
ballot_docs_headers="$work_dir/ballot-docs.headers"
wait_for_200 \
  "ballot-by-address documentation" \
  "$web_base/docs/api/ballot-by-address" \
  "$ballot_docs" \
  "$ballot_docs_headers"
grep -Fq "Geographic coverage" "$ballot_docs"
grep -Fq "Address privacy" "$ballot_docs"

openapi="$work_dir/rest-v1.yaml"
openapi_headers="$work_dir/openapi.headers"
wait_for_200 \
  "published OpenAPI contract" \
  "$web_base/openapi/rest-v1.yaml" \
  "$openapi" \
  "$openapi_headers"
grep -Eq '^openapi: 3\.1\.' "$openapi"
grep -Fq '/api/v1/elections/{electionId}/ballot:' "$openapi"

printf 'staging verification passed: %s and %s\n' "$api_base" "$web_base"
