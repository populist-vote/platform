use async_graphql::{SimpleObject, ID};
use db::{models::enums::State, Address};

#[derive(SimpleObject, Debug, Clone)]
pub struct AddressResult {
    id: ID,
    line_1: String,
    line_2: Option<String>,
    city: String,
    state: String,
    country: String,
    postal_code: String,
}

impl From<Address> for AddressResult {
    fn from(address: Address) -> Self {
        Self {
            id: ID::from(address.id),
            line_1: address.line_1,
            line_2: address.line_2,
            city: address.city,
            state: address.state,
            country: address.country,
            postal_code: address.postal_code,
        }
    }
}
