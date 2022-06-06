# Populist API

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
