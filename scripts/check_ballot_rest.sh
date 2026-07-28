#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0

cargo fmt --all -- --check
cargo test -p server rest::
cargo test --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --no-deps -- -D warnings

jq empty \
  docs/rest/examples/ballot-request.json \
  docs/rest/examples/ballot-response.json

ruby -ryaml -e '
  spec = YAML.load_file(ARGV.fetch(0))
  abort "expected OpenAPI 3.1" unless spec.fetch("openapi").start_with?("3.1.")
  paths = spec.fetch("paths")
  operations = {
    "/api/v1/elections" => "get",
    "/api/v1/elections/{electionId}" => "get",
    "/api/v1/elections/{electionId}/races" => "get",
    "/api/v1/elections/{electionId}/races/{raceId}" => "get",
    "/api/v1/elections/{electionId}/results" => "get",
    "/api/v1/elections/{electionId}/ballot-measures" => "get",
    "/api/v1/elections/{electionId}/ballot" => "post",
  }
  operations.each do |path, method|
    operation = paths.fetch(path).fetch(method)
    abort "missing 200 response for #{method.upcase} #{path}" unless operation.fetch("responses").key?("200")
  end
  schemas = spec.fetch("components").fetch("schemas")
  %w[
    ElectionCollection ElectionResponse RaceCollection RaceResponse
    ElectionResultCollection BallotMeasureCollection BallotRequest
  ].each do |schema|
    abort "missing #{schema} schema" unless schemas.key?(schema)
  end
  headers = spec.fetch("components").fetch("headers")
  %w[ElectionCache ElectionDataCache ResultsCache NoStore].each do |header|
    abort "missing #{header} header" unless headers.key?(header)
  end
' docs/rest/openapi.yaml

git diff --check
