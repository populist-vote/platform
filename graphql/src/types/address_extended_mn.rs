use async_graphql::SimpleObject;
use db::{AddressExtendedMN};

#[derive(SimpleObject, Debug, Clone)]
pub struct AddressExtendedMNResult {
    voting_tabulation_district_id: Option<String>,
    county_code: Option<String>,
    county_name: Option<String>,
    precinct_code: Option<String>,
    precinct_name: Option<String>,
    county_commissioner_district: Option<String>,
}

impl From<AddressExtendedMN> for AddressExtendedMNResult {
    fn from(address_extended_mn: AddressExtendedMN) -> Self {
        Self {
            voting_tabulation_district_id: address_extended_mn.voting_tabulation_district_id,
            county_code: address_extended_mn.county_code,
            county_name: address_extended_mn.county_name,
            precinct_code: address_extended_mn.precinct_code,
            precinct_name: address_extended_mn.precinct_name,
            county_commissioner_district: address_extended_mn.county_commissioner_district,
        }
    }
}
