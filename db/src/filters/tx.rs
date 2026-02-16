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

    // County commissioner
    if let (Some(cc), Some(clean)) = (county_commissioner_district.clone(), county_cleaned.clone())
    {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'county' \
                      AND o.county = ",
        );
        builder.push_bind(clean);
        builder.push(" AND o.district = ");
        builder.push_bind(cc);
        builder.push(")");
    }

    // Justice of the peace
    if let Some(jp) = justice_of_the_peace_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'justice_of_the_peace' \
                      AND o.district = ",
        );
        builder.push_bind(jp);
        builder.push(")");
    }

    // Constable
    if let Some(constable) = constable_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'constable' \
                      AND o.district = ",
        );
        builder.push_bind(constable);
        builder.push(")");
    }

    // State district courts (one OR per value in the vector)
    if let Some(courts) = state_district_courts.clone() {
        if !courts.is_empty() {
            builder.push(
                " OR (o.election_scope = 'district' \
                          AND o.district_type = 'state_district_courts' AND (",
            );
            for (i, district) in courts.iter().enumerate() {
                if i > 0 {
                    builder.push(" OR ");
                }
                builder.push("o.district = ");
                builder.push_bind(district.clone());
            }
            builder.push("))");
        }
    }

    // Court of appeals (one OR per value in the vector)
    if let Some(court_of_appeals) = court_of_appeals_districts.clone() {
        if !court_of_appeals.is_empty() {
            builder.push(
                " OR (o.election_scope = 'district' \
                          AND o.district_type = 'court_of_appeals' AND (",
            );
            for (i, district) in court_of_appeals.iter().enumerate() {
                if i > 0 {
                    builder.push(" OR ");
                }
                builder.push("o.district = ");
                builder.push_bind(district.clone());
            }
            builder.push("))");
        }
    }

    // Board of education
    if let Some(board_of_education) = board_of_education_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'board_of_education' \
                      AND o.district = ",
        );
        builder.push_bind(board_of_education);
        builder.push(")");
    }

    builder.push("))");
}