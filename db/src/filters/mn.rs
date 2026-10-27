use crate::State;
use sqlx::{Postgres, QueryBuilder};

/// A school subdistrict selects that seat. Seats with no subdistrict stay district-wide.
/// ISD 2902 stores names such as "Russell District", matched to the lookup name "Russell".
fn push_school_subdistrict(
    builder: &mut QueryBuilder<Postgres>,
    school_district_number: &str,
    school_subdistrict: Option<String>,
    school_subdistrict_name: Option<String>,
) {
    if school_district_number == "2902" {
        if let Some(name) = school_subdistrict_name {
            builder.push(
                " AND (o.subdistrict IS NULL OR LOWER(regexp_replace(o.subdistrict, ' District$', '')) = LOWER(",
            );
            builder.push_bind(name);
            builder.push("))");
        }
        return;
    }

    if let Some(subdistrict) = school_subdistrict {
        builder.push(" AND (o.subdistrict IS NULL OR o.subdistrict = ");
        builder.push_bind(subdistrict);
        builder.push(")");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_mn_filters(
    builder: &mut QueryBuilder<Postgres>,
    state: State,
    county: Option<&str>,
    city: String,
    congressional_district: Option<String>,
    state_senate_district: Option<String>,
    state_house_district: Option<String>,
    county_commissioner_district: Option<String>,
    judicial_district: Option<String>,
    school_district: Option<String>,
    school_district_type: Option<String>,
    school_subdistrict: Option<String>,
    school_subdistrict_name: Option<String>,
    ward: Option<String>,
    soil_and_water_district: Option<String>,
    hospital_district: Option<String>,
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

    // Judicial
    if let Some(jd) = judicial_district.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'judicial' \
                      AND o.district = ",
        );
        builder.push_bind(jd);
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

    // Soil & water
    if let (Some(swd), Some(clean)) = (soil_and_water_district.clone(), county_cleaned.clone()) {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'soil_and_water' \
                      AND o.county = ",
        );
        builder.push_bind(clean.clone());
        builder.push(" AND (REGEXP_REPLACE(o.district, '.*\\(([^)]+)\\).*', '\\1') = ");
        builder.push_bind(swd.clone());
        builder.push(" OR o.district = ");
        builder.push_bind(swd.clone());
        builder.push("))");
    }

    // Hospital
    if let Some(hosp) = hospital_district.clone() {
        if let Some(clean) = county_cleaned.clone() {
            match hosp.as_str() {
                "Cook County" => {
                    if let Some(commissioner) = county_commissioner_district.clone() {
                        builder.push(
                            " OR (o.election_scope = 'district' \
                                      AND o.district_type = 'hospital' \
                                      AND o.district = ",
                        );
                        builder.push_bind(hosp.clone());
                        builder.push(" AND o.subdistrict = ");
                        builder.push_bind(commissioner);
                        builder.push(")");
                    }
                }
                "Northern Itasca" => {
                    builder.push(
                        " OR (o.election_scope = 'district' \
                                  AND o.district_type = 'hospital' \
                                  AND o.district = ",
                    );
                    builder.push_bind(hosp.clone());
                    builder.push(" AND o.county = ");
                    builder.push_bind(clean);
                    builder.push(")");
                }
                _ => {
                    builder.push(
                        " OR (o.election_scope = 'district' \
                                  AND o.district_type = 'hospital' \
                                  AND o.district = ",
                    );
                    builder.push_bind(hosp);
                    builder.push(")");
                }
            }
        }
    }

    // Unorganized municipalities
    if city == "North Unorg" || city == "Long Lake Unorg" {
        builder.push(" OR (o.election_scope = 'city' AND o.municipality LIKE ");
        builder.push_bind(format!("%{}%", city));
        builder.push(")");
    }

    // City ward
    if let Some(wrd) = ward.clone() {
        builder.push(
            " OR (o.election_scope = 'district' \
                      AND o.district_type = 'city' \
                      AND o.municipality ILIKE ",
        );
        builder.push_bind(city.clone());
        builder.push(" AND REGEXP_REPLACE(o.district, '^[^0-9]*', '') = ");
        builder.push_bind(wrd);
        builder.push(")");
    }

    // Raw city
    builder.push(" OR (o.election_scope = 'city' AND o.municipality ILIKE ");
    builder.push_bind(city.clone());
    builder.push(")");

    // Schools
    if let (Some(sd_num), Some(sd_type)) = (school_district.clone(), school_district_type.clone()) {
        match sd_type.as_str() {
            "01" => {
                builder.push(
                    " OR (o.election_scope = 'district' \
                              AND o.district_type = 'school' \
                              AND REPLACE(o.district, 'ISD #', '') = ",
                );
                let district_number = sd_num.clone();
                builder.push_bind(sd_num);
                push_school_subdistrict(
                    builder,
                    &district_number,
                    school_subdistrict.clone(),
                    school_subdistrict_name.clone(),
                );
                builder.push(")");
            }
            "03" => {
                builder.push(
                    " OR (o.election_scope = 'district' \
                              AND o.district_type = 'school' \
                              AND REPLACE(o.district, 'SSD #', '') = ",
                );
                let district_number = sd_num.clone();
                builder.push_bind(sd_num);
                push_school_subdistrict(
                    builder,
                    &district_number,
                    school_subdistrict.clone(),
                    school_subdistrict_name.clone(),
                );
                builder.push(")");
            }
            _ => {}
        }
    }

    builder.push("))");
}
