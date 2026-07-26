# Populist API

## REST API

The REST API is versioned under `/api/v1`. Its first dataset endpoint is:

```http
GET /api/v1/states?limit=25&offset=0
```

Successful collection responses use a consistent `data` and `meta` envelope:

```json
{
  "data": [
    { "code": "AL", "name": "Alabama" },
    { "code": "AK", "name": "Alaska" }
  ],
  "meta": {
    "count": 2,
    "total": 59,
    "limit": 2,
    "offset": 0
  }
}
```

`limit` defaults to 25 and must be between 1 and 100. `offset` defaults to 0.
Errors use RFC 9457 problem details with the
`application/problem+json` content type and a stable `code` field for clients.
Every REST response includes `X-Request-Id`; a valid caller-provided value is
preserved. Request bodies are limited to 1 MiB by default.

Discovery and liveness endpoints are also available:

```http
GET /api/v1/
GET /api/v1/health
```

## Adding Endorsements to Politicians

You can use the GraphQL playground to run mutations to add new or existing politicians and organizations as endorsements. See [the readme](README.md) to get setup with an authorization token for the playground. A sample mutation to create new _and_ connect existing politicians as politician endorsements looks like this:

```graphql
mutation {
  updatePolitician(
    id: "c94a7204-e436-4449-9964-4b2accbc89ef"
    input: {
      politicianEndorsements: {
        create: [
          {
            slug: "bill-clinton"
            firstName: "Bill"
            lastName: "Clinton"
            homeState: NY
            party: DEMOCRATIC
          }
          {
            slug: "hillary-clinton"
            firstName: "Hillary"
            lastName: "Clinton"
            homeState: NY
            party: DEMOCRATIC
          }
        ]
        connect: ["existing-politician-slug", "joe-neguse"]
      }
    }
  ) {
    fullName
    endorsements {
      politicians {
        fullName
      }
    }
  }
}
```

And likewise with organizations:

```graphql
mutation {
  updatePolitician(
    id: "c94a7204-e436-4449-9964-4b2accbc89ef"
    input: {
      organizationEndorsements: {
        create: [
          { name: "Planned Parenthood" }
          { name: "National Rifle Associate" }
        ]
        connect: ["existing-organization-slug", "planned-parenthood"]
      }
    }
  ) {
    fullName
    endorsements {
      organizations {
        name
      }
    }
  }
}
```
