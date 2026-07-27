You are maintaining the Populist ballot-by-address REST flow.

Work only in this repository. Do not commit, push, deploy, modify credentials,
or invoke this agent loop recursively. Preserve unrelated user changes.

Inspect the current diff and the implementation behind:

- `POST /api/v1/elections/{electionId}/ballot`
- `server/src/rest/ballots.rs`
- `server/src/rest/error.rs`
- `server/src/rest/mod.rs`
- the shared ballot selection and address processing in
  `graphql/src/types/election.rs`
- `docs/rest/ballot-by-address.md`
- `docs/rest/openapi.yaml`
- `docs/rest/examples/`

Make one bounded improvement pass:

1. Harden correctness, privacy, availability, and stable response behavior.
2. Add or improve tests for every behavior you change, prioritizing realistic
   boundary and failure cases.
3. Keep the OpenAPI schema, examples, and client guide exactly aligned with
   executable behavior. Do not invent deployment URLs, credentials, contractual
   retention promises, support contacts, or nationwide address-level coverage.
4. Preserve the GraphQL `myBallotByAddress` behavior unless a shared fix is
   clearly required and tested.
5. Run `./scripts/check_ballot_rest.sh`. Fix failures within scope.

At the end, report the files changed, risks reduced, checks run, and any
remaining limitation that needs product, infrastructure, production data, or
client input. If the flow is already strong and the contract is aligned, make
no speculative changes and say so.
