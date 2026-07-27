# Ballot by Address REST API

This guide describes the version 1 endpoint that resolves the races and ballot
measures available for a US voting address. It is intended for client
engineering, product, support, privacy, and QA teams.

The machine-readable contract is
[OpenAPI 3.1](./openapi.yaml). The checked-in
[request](./examples/ballot-request.json) and
[response](./examples/ballot-response.json) examples can be used as fixtures.

## Before integrating

Obtain these values through your Populist onboarding contact:

- The HTTPS API base URL for your environment.
- An election UUID. This endpoint does not discover elections.
- Any authentication or per-client quota required by the deployment gateway.
  The application route itself does not currently enforce an authorization
  scheme.

The result reflects Populist's dataset at request time. It is not an official
sample ballot, voter-registration check, polling-place lookup, or statement of
voter eligibility. Clients should present it as informational and link users to
the relevant election authority for official information.

## Endpoint

```http
POST /api/v1/elections/{electionId}/ballot
Content-Type: application/json
```

Optional query parameter:

```text
endorserId=<organization UUID>
```

`endorserId` filters the `candidates` array in every race to candidates endorsed
by that organization. It does not remove races, ballot measures, result totals,
or winner references. Therefore, a race can remain in the response with an
empty `candidates` array.

### Request example

```bash
curl --request POST \
  --url "${POPULIST_API_BASE_URL}/api/v1/elections/7f7f1111-2222-4333-8444-555555555555/ballot" \
  --header "Content-Type: application/json" \
  --header "X-Request-Id: client-request-123" \
  --data @docs/rest/examples/ballot-request.json
```

JavaScript:

```js
const electionId = "7f7f1111-2222-4333-8444-555555555555";
const response = await fetch(
  `${baseUrl}/api/v1/elections/${electionId}/ballot`,
  {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Request-Id": crypto.randomUUID(),
    },
    body: JSON.stringify({
      address: {
        line1: "123 Main St",
        city: "Minneapolis",
        state: "MN",
        postalCode: "55401",
        country: "US",
      },
    }),
  },
);

if (!response.ok) {
  const problem = await response.json();
  throw new Error(`${problem.code}: ${problem.detail}`);
}

const { data } = await response.json();
```

### Request fields

The request body limit is 8 KiB. Unknown JSON fields and unknown query
parameters are rejected.

| Field | Required | Rules |
| --- | --- | --- |
| `address.line1` | Yes | Non-empty after trimming; at most 200 Unicode characters; no control characters. |
| `address.line2` | No | At most 200 characters. Accepted for compatibility, but deliberately not used for district resolution, sent to the geocoder, or stored. |
| `address.city` | Yes | Non-empty after trimming; at most 100 characters; no control characters. |
| `address.state` | Yes | Two-letter US state or territory code, case-insensitive. |
| `address.postalCode` | Yes | Five-digit ZIP or ZIP+4, such as `55401` or `55401-1234`. |
| `address.country` | No | `US` or `USA`, case-insensitive. Defaults and normalizes to `US`. |
| `endorserId` | No | Organization UUID supplied as a query parameter. |

The API does not accept caller-provided coordinates or legislative districts.
It derives those values from the address.

## Success response

A successful request returns `200 application/json`:

```json
{
  "data": {
    "election": {},
    "races": [],
    "ballotMeasures": [],
    "coverage": {
      "races": "address_specific",
      "ballotMeasures": "address_specific",
      "warnings": []
    }
  }
}
```

See the complete [response example](./examples/ballot-response.json).

### Election

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | UUID string | Stable election identifier. |
| `slug` | string | Human-readable election key. |
| `title` | string | Display title. |
| `description` | string or `null` | Optional description. |
| `state` | state code or `null` | State associated with the election; `null` for elections without a single state. |
| `electionDate` | `YYYY-MM-DD` | Election date. |

### Races

Each race includes its identity, office, candidates, reported results, and
public related embeds.

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | UUID string | Stable race identifier. |
| `slug` | string | Human-readable race key. |
| `title` | string | Display title. |
| `electionId` | UUID string or `null` | Associated election. |
| `officeId` | UUID string | Associated office. |
| `raceType` | enum | `primary` or `general`. |
| `voteType` | enum | `plurality` or `rankedchoice`. |
| `state` | state code or `null` | Race state. |
| `electionDate` | `YYYY-MM-DD` | Election date inherited from the election. |
| `isSpecialElection` | boolean | Whether this is a special election. |
| `numElect` | integer or `null` | Number of seats selected when known. |
| `party` | party or `null` | Party for a party-specific race. |
| `office` | object | Office and incumbent details. |
| `candidates` | array | Running candidates, optionally filtered by `endorserId`. |
| `results` | object | Reported vote totals and winners. |
| `relatedEmbeds` | array | Public embed references. Administrative origin restrictions are never exposed. |

Office fields describe the office's geographic and political scope.
`electionScope` is one of `national`, `state`, `county`, `city`, or `district`;
`politicalScope` is `local`, `state`, or `federal`. `districtType` is nullable
and can be:

```text
us_congressional, state_senate, state_house, school, city, county,
judicial, hospital, soil_and_water, transportation, park,
board_of_education, court_of_appeals, justice_of_the_peace, constable,
voting_precinct
```

`office.subtitle` and `office.subtitleShort` may be stored values or
deterministically computed display labels. Consumers should not parse
geographic identifiers back out of these labels; use the structured fields.

Candidate objects contain `id`, `slug`, `fullName`, optional contact and image
fields, optional party information, and `assets.thumbnailImage160`. Incumbents
use the same display-oriented subset but do not include `slug`, email, or phone.
Null contact fields mean the dataset has no value; they do not imply that the
candidate has no public contact channel.

`results.totalVotes` and individual `votes` are nullable while results are
unavailable. `precinctReportingPercentage` and `votePercentage` are percentage
points in the range normally understood as 0 through 100, rounded to one
decimal place. They are `null` when their required denominator is missing or
zero. `winners` contains objects with winning candidate IDs and can be empty
before a race is called.

Related embed `embedType` values are:

```text
legislation, legislation_tracker, politician, question, poll, race,
candidate_guide, my_ballot, conversation
```

### Ballot measures

Ballot measures contain `id`, `title`, optional `description`, `state`,
`ballotMeasureCode`, nullable vote totals and precinct counts, and nullable
`electionScope`. A missing total means results are not currently available.

### Coverage

Clients must inspect `data.coverage` before describing the response as a
complete address-specific ballot.

| Address state | Race coverage | Ballot-measure coverage |
| --- | --- | --- |
| Minnesota | `address_specific` | `address_specific` |
| Texas | `address_specific` | `statewide_only` |
| Other supported state/territory codes | `statewide_only` | `statewide_only` |

`warnings` contains human-readable limitations. Its text may evolve; branch on
the stable `races` and `ballotMeasures` values, not warning text.

`statewide_only` means the array can include statewide records but district,
county, city, school, or other local matching is not available. It does not
mean that Populist has every statewide contest or measure.

### Ordering

Array ordering is deterministic so clients can render without additional
sorting:

- Races: office priority, numeric and textual district, numeric and textual
  seat, race title descending, then race ID.
- Candidates and incumbents: last name, first name, then ID.
- Candidate vote rows: candidate ID.
- Related embeds: embed ID.
- Ballot measures: title, then ID.

Ordering is not a pagination or synchronization cursor and may change in a
future version if the documented contract changes.

## Errors

Errors use `application/problem+json` and include:

```json
{
  "type": "about:blank",
  "title": "Unprocessable Entity",
  "status": 422,
  "code": "invalid_address",
  "detail": "The address could not be resolved to a voting location.",
  "instance": "/api/v1/elections/7f7f1111-2222-4333-8444-555555555555/ballot"
}
```

| HTTP | Stable `code` | Typical cause | Client action |
| --- | --- | --- | --- |
| 400 | `invalid_path_parameter` | `electionId` is not a UUID. | Correct the URL. |
| 400 | `invalid_parameter` | `endorserId` is not a UUID. | Correct the query parameter. |
| 400 | `malformed_query` | Unknown or malformed query parameter. | Send only documented parameters. |
| 400 | `invalid_body` | Invalid JSON, missing required structure, wrong types, or unknown fields. | Correct the request body. |
| 404 | `resource_not_found` | The election UUID does not exist. | Refresh the configured election ID. |
| 405 | `method_not_allowed` | A method other than `POST` was used. | Use `POST`. |
| 413 | `payload_too_large` | Body exceeds 8 KiB. | Reduce the body. |
| 415 | `unsupported_media_type` | Missing or incorrect content type. | Send `Content-Type: application/json`. |
| 422 | `invalid_address` | Invalid fields, no geocoder result, or a result in a different state. | Ask the user to review the address. Do not retry unchanged input. |
| 429 | `rate_limited` | Ballot lookup capacity is exhausted. | Retry after the `Retry-After` delay with jitter. |
| 500 | `internal_error` | Unexpected application or database failure. | Retry cautiously; report the request ID if persistent. |
| 503 | `address_service_unavailable` | The geocoding service or configuration is unavailable. | Retry after the `Retry-After` delay with jitter. |
| 504 | `request_timeout` | Lookup exceeded the 20-second application deadline. | Retry once with jitter; report persistent failures. |

`Retry-After` is currently `5` seconds on 429 and 503 responses. Cap retries and
use exponential backoff with jitter. Do not automatically retry 400, 404, 405,
413, 415, or 422 responses.

## HTTP behavior

- Every REST response includes `X-Request-Id`. A valid caller value of at most
  128 bytes is preserved; otherwise the server generates a UUID. Never put an
  address, email address, token, or other personal data in this header.
- Successful ballot responses include `Cache-Control: no-store` and
  `Pragma: no-cache`. Error responses include `Cache-Control: no-store`.
- Every response includes `X-Content-Type-Options: nosniff`.
- The application allows at most 32 concurrent ballot lookups per server
  process and rejects excess work with 429. Deployment gateways may impose
  additional authentication, quota, or rate limits.
- The endpoint is safe to retry from a response-semantics perspective, but it
  does not implement `Idempotency-Key`. A cache miss can repeat a geocoder call.

## Address privacy and processing

Use HTTPS and avoid application or proxy logs that record request bodies.

The API never returns the submitted address. `line2` is discarded during
validation and is not sent to geocoding or storage. On an exact cached address
match, the existing geographic record is reused without calling the geocoder.
On a cache miss, street, city, state, and ZIP are sent to the configured
Geocodio service to derive coordinates and districts.

The application creates a temporary address row while selecting ballot data and
attempts to remove a newly created row immediately after processing, provided
it is not linked to a user profile. Existing address rows are not deleted.
Because abnormal process termination, cancellation, or cleanup failure can
interrupt best-effort removal, clients should not treat this implementation
detail as a zero-retention guarantee. Confirm contractual retention,
subprocessor, and deletion requirements with the Populist onboarding contact
before sending production personal data.

## Versioning and compatibility

The path major version is `/api/v1`. Clients should:

- Ignore response fields they do not understand.
- Treat unknown enum values defensively.
- Use IDs, not slugs or titles, as record identity.
- Expect nullable values to remain absent until Populist has the data.
- Pin tests to the OpenAPI contract and representative examples rather than to
  a single production ballot.

Breaking changes require a new path major version. Additive fields and new enum
values can be introduced within v1.

## Client acceptance checklist

- Test a known Minnesota address and assert both coverage fields are
  `address_specific`.
- If applicable, test Texas and confirm the ballot-measure warning is handled.
- Test an invalid UUID, malformed JSON, invalid ZIP, unknown election, and
  unresolvable address.
- Verify that request and response logs do not contain the address.
- Verify that UI copy distinguishes informational data from an official ballot.
- Propagate `X-Request-Id` into support telemetry without personal data.
- Implement bounded 429/503/504 retries with jitter.
- Decide how empty candidate, race, or ballot-measure arrays display.
- Re-run contract tests whenever `openapi.yaml` changes.

For support, provide the environment, UTC timestamp, election ID, HTTP status,
problem `code`, and `X-Request-Id`. Do not send the voter's full address unless
the approved support channel explicitly requires it.
