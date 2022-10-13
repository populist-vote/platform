use async_graphql::SimpleObject;
use db::{models::enums::State, Address, AddressExtendedMN};

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

#[derive(SimpleObject, Debug, Clone)]
pub struct AddressExtendedMNResult {
    voting_tabulation_district_id: Option<String>,
    county_code: Option<String>,
    county_name: Option<String>,
    precinct_code: Option<String>,
    precinct_name: Option<String>,
    county_commissioner_district: Option<String>,
    judicial_district: Option<String>,
    school_district_number: Option<String>,
    school_district_name: Option<String>,
    school_subdistrict_code: Option<String>,
    school_subdistrict_name: Option<String>,
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
            judicial_district: address_extended_mn.judicial_district,
            school_district_number: address_extended_mn.school_district_number,
            school_district_name: address_extended_mn.school_district_name,
            school_subdistrict_code: address_extended_mn.school_subdistrict_code,
            school_subdistrict_name: address_extended_mn.school_subdistrict_name,
        }
    }
}
