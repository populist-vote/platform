use crate::State;
use sqlx::{Postgres, QueryBuilder};

pub fn apply_tx_filters(
    builder: &mut QueryBuilder<Postgres>,
    state: State,
    county: Option<&str>,
    precinct: Option<String>,
    congressional_district: Option<String>,
    state_senate_district: Option<String>,
    state_house_district: Option<String>,
    county_commissioner_district: Option<String>,
    justice_of_the_peace_district: Option<String>,
    constable_district: Option<String>,
    state_district_courts: Option<Vec<String>>,
    court_of_appeals_districts: Option<Vec<String>>,
    board_of_education_district: Option<String>,
) {
    // Normalize county → owned String
    let county_cleaned = county.map(|c| c.replace(" County", ""));

    builder.push(" OR (o.state = ");
    builder.push_bind(state);
    builder.push(" AND (");

    // County
    if let Some(clean) = county_cleaned.clone() {
        builder.push(" (o.election_scope = 'county' AND o.county = ");
        builder.push_bind(clean);
        builder.push(")");
    }

    // Congressional
    if let Some(cd) = congressional_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'us_congressional' \
                      AND o.district = ",
        );
        builder.push_bind(cd);
        builder.push(")");
    }

    // State senate
    if let Some(sd) = state_senate_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'state_senate' \
                      AND o.district = ",
        );
        builder.push_bind(sd);
        builder.push(")");
    }

    // State house
    if let Some(hd) = state_house_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'state_house' \
                      AND o.district = ",
        );
        builder.push_bind(hd);
        builder.push(")");
    }

    builder.push("))");
}