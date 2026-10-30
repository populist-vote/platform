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

/// School-district name used to match `office.district`. A trailing county
/// disambiguator is removed: `"Wylie ISD (Taylor)"` → `"Wylie ISD"`.
fn school_district_match_name(name: &str) -> String {
    let Some(start) = name.rfind('(') else {
        return name.trim().to_string();
    };
    let Some(end) = name.rfind(')') else {
        return name.trim().to_string();
    };
    if end <= start || name[start + 1..end].trim().is_empty() {
        return name.trim().to_string();
    }
    let mut stripped = String::new();
    stripped.push_str(name[..start].trim_end());
    stripped.push_str(name[end + 1..].trim_start());
    let stripped = stripped.trim();
    if stripped.is_empty() {
        name.trim().to_string()
    } else {
        stripped.to_string()
    }
}

/// Which table the Texas location clauses are written against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxFilterSubject {
    /// Race query: alias `o`, wrapped as `OR (o.state = $state AND ( … ))`.
    Office,
    /// Ballot-measure query: alias `bm`, each clause joined with `OR`.
    BallotMeasure,
}

#[allow(clippy::too_many_arguments)]
pub fn apply_tx_filters(
    builder: &mut QueryBuilder<Postgres>,
    subject: TxFilterSubject,
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
    let (alias, prefix_county_with_or) = match subject {
        TxFilterSubject::Office => {
            builder.push(" OR (o.state = ");
            builder.push_bind(state);
            builder.push(" AND (");
            ("o", false)
        }
        TxFilterSubject::BallotMeasure => ("bm", true),
    };

    // Normalize county → owned String
    let county_cleaned = county.map(|c| c.replace(" County", ""));

    // County
    if let Some(clean) = county_cleaned.clone() {
        if prefix_county_with_or {
            builder.push(" OR ");
        } else {
            builder.push(" ");
        }
        builder.push(format!(
            "({alias}.election_scope = 'county' AND {alias}.county = "
        ));
        builder.push_bind(clean);
        builder.push(")");
    }

    // Congressional
    if let Some(cd) = congressional_district.clone() {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'us_congressional' \
                      AND {alias}.district = "
        ));
        builder.push_bind(cd);
        builder.push(")");
    }

    // State senate
    if let Some(sd) = state_senate_district.clone() {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'state_senate' \
                      AND {alias}.district = "
        ));
        builder.push_bind(sd);
        builder.push(")");
    }

    // State house
    if let Some(hd) = state_house_district.clone() {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'state_house' \
                      AND {alias}.district = "
        ));
        builder.push_bind(hd);
        builder.push(")");
    }

    // Board of education
    if let Some(boe) = board_of_education_district.clone() {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'board_of_education' \
                      AND {alias}.district = "
        ));
        builder.push_bind(boe);
        builder.push(")");
    }

    // County commissioner
    if let (Some(cc), Some(clean)) = (county_commissioner_district.clone(), county_cleaned.clone())
    {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'county' \
                      AND {alias}.county = "
        ));
        builder.push_bind(clean);
        builder.push(format!(" AND {alias}.district = "));
        builder.push_bind(cc);
        builder.push(")");
    }

    // Justice of the peace (district can be comma-separated; check jp is in that list)
    if let (Some(jp), Some(clean)) = (
        justice_of_the_peace_district.clone(),
        county_cleaned.clone(),
    ) {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'justice_of_the_peace' \
                      AND {alias}.county = "
        ));
        builder.push_bind(clean);
        builder.push(" AND ");
        builder.push_bind(jp);
        builder.push(format!(
            " = ANY(string_to_array(replace({alias}.district, ' ', ''), ','))"
        ));
        builder.push(")");
    }

    // Constable
    if let (Some(constable), Some(clean)) = (constable_district.clone(), county_cleaned.clone()) {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'constable' \
                      AND {alias}.county = "
        ));
        builder.push_bind(clean);
        builder.push(" AND ");
        builder.push_bind(constable);
        builder.push(format!(
            " = ANY(string_to_array(replace({alias}.district, ' ', ''), ','))"
        ));
        builder.push(")");
    }

    // State district courts (one OR per value in the vector)
    if let Some(courts) = state_district_courts.clone() {
        if !courts.is_empty() {
            builder.push(format!(
                " OR ({alias}.election_scope = 'district' \
                          AND {alias}.district_type = 'judicial' AND ("
            ));
            for (i, district) in courts.iter().enumerate() {
                if i > 0 {
                    builder.push(" OR ");
                }
                builder.push(format!("{alias}.district = "));
                builder.push_bind(district.clone());
            }
            builder.push("))");
        }
    }

    // Court of appeals (one OR per value in the vector)
    if let Some(court_of_appeals) = court_of_appeals_districts.clone() {
        if !court_of_appeals.is_empty() {
            builder.push(format!(
                " OR ({alias}.election_scope = 'district' \
                          AND {alias}.district_type = 'court_of_appeals' AND ("
            ));
            for (i, district) in court_of_appeals.iter().enumerate() {
                if i > 0 {
                    builder.push(" OR ");
                }
                builder.push(format!("{alias}.district = "));
                builder.push_bind(district.clone());
            }
            builder.push("))");
        }
    }

    // School district. `name` from tx_map_sd_v1 matches district.
    // A name like "Wylie ISD (Taylor)" is matched as "Wylie ISD", and also requires
    // that county to be one of the comma-separated values in county.
    // A subdistrict narrows single-member seats; at-large seats leave subdistrict null.
    // Districts with no subdistrict polygon match every seat in that school district.
    if let Some(school_district) = school_district.clone() {
        let county_name = parenthetical_county(&school_district);
        let district_name = school_district_match_name(&school_district);
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'school' \
                      AND {alias}.district = "
        ));
        builder.push_bind(district_name);
        if let Some(county_name) = county_name {
            builder.push(format!(
                " AND EXISTS ( \
                      SELECT 1 \
                      FROM unnest(string_to_array({alias}.county, ',')) AS county_name \
                      WHERE btrim(county_name) ILIKE "
            ));
            builder.push_bind(county_name);
            builder.push(")");
        }
        if let Some(subdistrict) = school_subdistrict.clone() {
            builder.push(format!(
                " AND ({alias}.subdistrict IS NULL OR {alias}.subdistrict = "
            ));
            builder.push_bind(subdistrict);
            builder.push(")");
        }
        builder.push(")");
    }

    // College district. `name` from tx_map_college_v1 matches district.
    if let Some(college_district) = college_district.clone() {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'tx_college' \
                      AND {alias}.district = "
        ));
        builder.push_bind(college_district);
        builder.push(")");
    }

    // City: the whole municipality, plus a ward when the
    // polygon has a subdistrict. Subdistrict "District 10" is compared as "10".
    if let Some(municipality) = municipality.clone() {
        if let Some(subdistrict) = municipality_subdistrict.clone() {
            builder.push(format!(
                " OR ({alias}.election_scope = 'district' \
                          AND {alias}.district_type = 'city' \
                          AND {alias}.municipality ILIKE "
            ));
            builder.push_bind(municipality.clone());
            builder.push(format!(
                " AND REGEXP_REPLACE({alias}.district, '^[^0-9]*', '') = "
            ));
            builder.push_bind(subdistrict);
            builder.push(")");
        }

        builder.push(format!(
            " OR ({alias}.election_scope = 'city' AND {alias}.municipality ILIKE "
        ));
        builder.push_bind(municipality);
        builder.push(")");
    }

    // Precinct Chairs
    if let (Some(precinct), Some(clean)) = (precinct.clone(), county_cleaned.clone()) {
        builder.push(format!(
            " OR ({alias}.election_scope = 'district' \
                      AND {alias}.district_type = 'voting_precinct' \
                      AND {alias}.county = "
        ));
        builder.push_bind(clean);
        builder.push(format!(" AND {alias}.district = "));
        builder.push_bind(precinct);
        builder.push(")");
    }

    if subject == TxFilterSubject::Office {
        builder.push("))");
    }
}

#[cfg(test)]
mod tests {
    use super::{parenthetical_county, school_district_match_name};

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

    #[test]
    fn strips_a_parenthetical_county_before_district_match() {
        assert_eq!(
            school_district_match_name("Wylie ISD (Taylor)"),
            "Wylie ISD"
        );
        assert_eq!(
            school_district_match_name("Wylie ISD (Collin)"),
            "Wylie ISD"
        );
        assert_eq!(
            school_district_match_name("Alamo Heights ISD"),
            "Alamo Heights ISD"
        );
    }
}
