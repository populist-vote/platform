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
  path = spec.fetch("paths").fetch("/api/v1/elections/{electionId}/ballot")
  operation = path.fetch("post")
  abort "missing 200 response" unless operation.fetch("responses").key?("200")
  abort "missing ballot request schema" unless spec.fetch("components").fetch("schemas").key?("BallotRequest")
' docs/rest/openapi.yaml

git diff --check
