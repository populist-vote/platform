use async_graphql::SimpleObject;
use db::{models::enums::State, Address};

#[derive(SimpleObject, Debug, Clone)]
pub struct AddressResult {
    line_1: String,
    line_2: Option<String>,
    city: String,
    state: State,
    country: String,
    postal_code: String,
}

impl From<Address> for AddressResult {
    fn from(address: Address) -> Self {
        Self {
            line_1: address.line_1,
            line_2: address.line_2,
            city: address.city,
            state: address.state,
            country: address.country,
            postal_code: address.postal_code,
        }
    }
}
