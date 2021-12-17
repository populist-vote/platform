# Votesmart 
A rust wrapper around the Votesmart REST API

## Getting Started
To get started, you'll need to instantiate a VotesmartProxy in your program. You have two options:
```rust
use votesmart::VotesmartProxy
// If you have a `VOTESMART_API_KEY` set in your .env or environment
let proxy = VotesmartProxy::new().unwrap();
// If you want to pass in the API key from elsewhere
let proxy = VotesmartProxy::new_from_key(your_api_key);
```

From there, each of Votesmarts Objects are namespaced from the proxy you just instantiated so you can run queries like this:
```rust
let candidate_id = 53279 // Joe Biden
let response = proxy.candidate_bio().get_detailed_bio(candidate_id).await?;
if response.status().is_success() {
    let json: serde_json::Value = response.json().await?;
    // Do whatever you want with this data
} else {
    panic!("Something went wrong fetching Joe Biden's bio");
}
```
