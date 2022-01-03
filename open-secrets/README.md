# Open Secrets
A wrapper for the OpenSecrets REST API

# Docs
https://docs.rs/legiscan/latest/open-secrets/

## Quick Start
To get started, you'll need to instantiate a OpenSecretsProxy in your program. You have two options:
```rust
use open_secrets::OpenSecretsProxy;
// If you have a `LEGISCAN_API_KEY` set in your .env or environment
let proxy = OpenSecretsProxy::new().unwrap();
// If you want to pass in the API key from elsewhere
let proxy = OpenSecretsProxy::new_from_key(your_api_key);
```
If you, for some strange reason, want your response output as something besides JSON, you can adjust the output type like so:
```rust
use open_secrets::OutputType::Doc;
proxy.with_output(Doc);
```

Now you're ready to make some calls to OpenSecrets.
```rust
// Lets get Nancy Pelosi's summary
let response = proxy.cand_summary("N00007360", None).await.unwrap();
let json: serde_json::Value = response.json().await.unwrap();
assert_eq!(
        json["response"]["summary"]["@attributes"]["cand_name"],
        "Pelosi, Nancy"
    );
```