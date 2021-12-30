# Legiscan
A strongly typed client for the Legiscan REST API

# Docs
https://docs.rs/legiscan/latest/legiscan/

## Quick Start
To get started, you'll need to instantiate a LegiscanProxy in your program. You have two options:
```rust
use legiscan::LegiscanProxy
// If you have a `LEGISCAN_API_KEY` set in your .env or environment
let proxy = LegiscanProxy::new().unwrap();
// If you want to pass in the API key from elsewhere
let proxy = LegiscanProxy::new_from_key(your_api_key);
```

Once you've got your proxy instantiated, you can query Legiscan's API with ease.  All responses are strongly typed with serde and serde_json so you will have easy access to all nested fields in a Legiscan response.  Here's a quick example:

```rust
let bill_id = 1167968 // From the Legiscan docs
let bill = proxy.get_bill(bill_id).await.unwrap();
println!("{}", bill.state) // "MD"
println!("{}", bill.bill_number) // "SB181"
```