# Ballot REST hardening loop

`scripts/ballot_rest_agent_loop.sh` runs a bounded Codex improvement pass and
then executes the same deterministic validation gate used by humans. Its goal
is to keep implementation, tests, examples, OpenAPI, and client documentation
aligned as this endpoint evolves.

The loop is intentionally not a background service. Run it from a dedicated,
reviewable branch:

```bash
./scripts/ballot_rest_agent_loop.sh
```

Pass a number from 1 through 5 to request multiple review-and-gate iterations:

```bash
./scripts/ballot_rest_agent_loop.sh 2
```

The loop refuses to start with tracked or untracked changes by default. This
prevents an automated pass from obscuring work already in progress. To
deliberately run against a dirty working tree:

```bash
ALLOW_DIRTY=1 ./scripts/ballot_rest_agent_loop.sh
```

Each agent receives the checked-in
[`agent-loop-prompt.md`](./agent-loop-prompt.md), can write only within the
workspace sandbox, and is instructed not to commit, push, deploy, access
credentials, or invoke itself. After the agent exits, the outer script runs
`scripts/check_ballot_rest.sh`. The loop stops immediately if either the agent
or validation fails, leaving the working tree available for diagnosis.

Review every resulting diff before committing:

```bash
git status --short
git diff --check
git diff
```

The deterministic gate can also be run without an agent:

```bash
./scripts/check_ballot_rest.sh
```

It checks formatting, focused REST tests, all workspace tests and targets,
strict Clippy, JSON examples, basic OpenAPI structure, and whitespace errors.
Tests that require live client credentials or production-like GIS datasets
remain an explicit integration-stage responsibility.
