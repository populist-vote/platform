+++
title = "Quick Start"
description = "Getting started with the Populist API is easy."
date = 2021-05-01T08:20:00+00:00
updated = 2021-05-01T08:20:00+00:00
draft = false
weight = 20
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "Getting started with the Populist API is quick and painless."
toc = true
top = false
+++

## Authentication

You will need an API key before making calls to our GraphQL API.  Please email us at <a href="mailto:info@populist.us" >info@populist.us</a> to request a free API key.

## Formatting Requests

Requests to the Populist GraphQL API require you to pass an `Authorization` header with your request in the following format:
```
Authorization: "Bearer <YOUR TOKEN HERE>"
```

```typescript
const query = gql`
  {
	politicianBySlug(slug: "nancy-pelosi") {
        fullName
        age
        thumbnailImageUrl
    }  
  }
`

const data = fetch("api.populist.us", {
    headers: {
        "Authorization": "Bearer <YOUR API KEY HERE>",
        ...otherHeaders
    },
    body: JSON.stringify(query)
})
```

