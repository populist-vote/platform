use crate::State;
use sqlx::{Postgres, QueryBuilder};

/// County disambiguator in a school-district name, e.g. `"Wylie ISD (Taylor)"` → `"Taylor"`.
fn parenthetical_county(name: &str) -> Option<String> {
    let start = name.rfind('(')?;
    let end = name.rfind(')')?;
    if end <= start {
        return None;
    }
    let county = name[start + 1..end].trim();
    if county.is_empty() {
        None
    } else {
        Some(county.to_string())
    }
}

#[allow(clippy::too_many_arguments)]
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
    school_district: Option<String>,
    school_subdistrict: Option<String>,
    college_district: Option<String>,
    municipality: Option<String>,
    municipality_subdistrict: Option<String>,
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

    // Board of education
    if let Some(boe) = board_of_education_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'board_of_education' \
                      AND o.district = ",
        );
        builder.push_bind(boe);
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

    // Justice of the peace (o.district can be comma-separated; check jp is in that list)
    if let (Some(jp), Some(clean)) = (
        justice_of_the_peace_district.clone(),
        county_cleaned.clone(),
    ) {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'justice_of_the_peace' \
                      AND o.county = ",
        );
        builder.push_bind(clean);
        builder.push(" AND ");
        builder.push_bind(jp);
        builder.push(" = ANY(string_to_array(replace(o.district, ' ', ''), ','))");
        builder.push(")");
    }

    // Constable
    if let (Some(constable), Some(clean)) = (constable_district.clone(), county_cleaned.clone()) {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'constable' \
                      AND o.county = ",
        );
        builder.push_bind(clean);
        builder.push(" AND ");
        builder.push_bind(constable);
        builder.push(" = ANY(string_to_array(replace(o.district, ' ', ''), ','))");
        builder.push(")");
    }

    // State district courts (one OR per value in the vector)
    if let Some(courts) = state_district_courts.clone() {
        if !courts.is_empty() {
            builder.push(
                " OR (o.election_scope = 'district' \
                          AND o.district_type = 'judicial' AND (",
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

    // School district. `name` from tx_map_sd_v1 matches office.district.
    // A name like "Wylie ISD (Taylor)" also requires that county to be one of the
    // comma-separated values in office.county.
    // A subdistrict narrows single-member seats; at-large seats leave office.subdistrict null.
    // Districts with no subdistrict polygon match every seat in that school district.
    if let Some(school_district) = school_district.clone() {
        let county_name = parenthetical_county(&school_district);
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'school' \
                      AND o.district = ",
        );
        builder.push_bind(school_district);
        if let Some(county_name) = county_name {
            builder.push(
                " AND EXISTS ( \
                      SELECT 1 \
                      FROM unnest(string_to_array(o.county, ',')) AS county_name \
                      WHERE btrim(county_name) ILIKE ",
            );
            builder.push_bind(county_name);
            builder.push(")");
        }
        if let Some(subdistrict) = school_subdistrict.clone() {
            builder.push(" AND (o.subdistrict IS NULL OR o.subdistrict = ");
            builder.push_bind(subdistrict);
            builder.push(")");
        }
        builder.push(")");
    }

    // College district. `name` from tx_map_college_v1 matches office.district.
    if let Some(college_district) = college_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'tx_college' \
                      AND o.district = ",
        );
        builder.push_bind(college_district);
        builder.push(")");
    }

    // City, matching Minnesota: the whole municipality, plus a ward when the
    // polygon has a subdistrict. Subdistrict "District 10" is compared as "10".
    if let Some(municipality) = municipality.clone() {
        if let Some(subdistrict) = municipality_subdistrict.clone() {
            builder.push(
                " OR (o.election_scope = 'district' \
                          AND o.district_type = 'city' \
                          AND o.municipality ILIKE ",
            );
            builder.push_bind(municipality.clone());
            builder.push(" AND REGEXP_REPLACE(o.district, '^[^0-9]*', '') = ");
            builder.push_bind(subdistrict);
            builder.push(")");
        }

        builder.push(" OR (o.election_scope = 'city' AND o.municipality ILIKE ");
        builder.push_bind(municipality);
        builder.push(")");
    }

    // Precinct Chairs
    if let (Some(precinct), Some(clean)) = (precinct.clone(), county_cleaned.clone()) {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'voting_precinct' \
                      AND o.county = ",
        );
        builder.push_bind(clean);
        builder.push(" AND o.district = ");
        builder.push_bind(precinct);
        builder.push(")");
    }

    builder.push("))");
}

#[cfg(test)]
mod tests {
    use super::parenthetical_county;

    #[test]
    fn reads_a_county_disambiguator() {
        assert_eq!(
            parenthetical_county("Wylie ISD (Taylor)").as_deref(),
            Some("Taylor")
        );
        assert_eq!(
            parenthetical_county("Wylie ISD (Collin)").as_deref(),
            Some("Collin")
        );
        assert_eq!(parenthetical_county("Alamo Heights ISD"), None);
    }
}
