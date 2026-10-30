//! Helpers for Texas local ballot measures (propositions).
//!
//! The measure name is "Proposition" plus the next word in the source title.
//! District, subdistrict, and election scope come from that subdivision, using the
//! same office extractors as local candidate filings.

use crate::extractors::politician::title_case;
use crate::extractors::tx::tx_office::{
    extract_office_district, extract_office_name, extract_office_scope,
    is_special_district_with_subdivision,
};
use db::{DistrictType, ElectionScope};

/// Geography stored on `ballot_measure` for address matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BallotMeasureGeography {
    pub election_scope: ElectionScope,
    pub district_type: Option<DistrictType>,
    pub district: Option<String>,
    pub subdistrict: Option<String>,
    pub municipality: Option<String>,
}

/// `"Austin Community College District - Annexation Proposition A"` → `"Proposition A"`.
/// A title with no word after "Proposition" returns `"Proposition"`.
pub fn extract_measure_name(title: &str) -> String {
    let title = title.trim();
    let Some(start) = title.to_ascii_lowercase().find("proposition") else {
        return title.to_string();
    };
    let proposition = &title[start..start + "proposition".len()];
    let after = title[start + "proposition".len()..].trim_start();
    match after.split_whitespace().next() {
        Some(word) => format!("{proposition} {word}"),
        None => proposition.to_string(),
    }
}

/// District, subdistrict, scope, and municipality for a political subdivision.
pub fn extract_ballot_measure_geography(political_subdivision: &str) -> BallotMeasureGeography {
    let subdivision = strip_trailing_texas(political_subdivision.trim());

    if let Some(municipality) = extract_municipality(&subdivision) {
        return BallotMeasureGeography {
            election_scope: ElectionScope::City,
            district_type: None,
            district: None,
            subdistrict: None,
            municipality: Some(municipality),
        };
    }

    if subdivision.to_lowercase().contains("school district") {
        return BallotMeasureGeography {
            election_scope: ElectionScope::District,
            district_type: Some(DistrictType::School),
            district: Some(normalize_school_district(&subdivision)),
            subdistrict: None,
            municipality: None,
        };
    }

    if let Some((name, save_name_as_district)) = extract_office_name(&subdivision, None) {
        let district = if save_name_as_district {
            Some(
                name.strip_suffix(" Board")
                    .unwrap_or(&name)
                    .to_string(),
            )
        } else {
            None
        };
        let subdistrict = if is_special_district_with_subdivision(&name) {
            extract_office_district(&subdivision, None, None, Some(&name))
        } else {
            None
        };
        let district_for_scope = district.as_deref();
        let (_, election_scope, district_type) =
            extract_office_scope(&name, None, None, district_for_scope).unwrap_or((
                db::PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::TxSpecial),
            ));
        return BallotMeasureGeography {
            election_scope,
            district_type,
            district,
            subdistrict,
            municipality: None,
        };
    }

    BallotMeasureGeography {
        election_scope: ElectionScope::District,
        district_type: Some(DistrictType::TxSpecial),
        district: Some(subdivision),
        subdistrict: None,
        municipality: None,
    }
}

/// `"Travis; Williamson"` → `"Travis, Williamson"`.
pub fn normalize_counties(county: &str) -> Option<String> {
    let counties: Vec<&str> = county
        .split(';')
        .map(str::trim)
        .filter(|county| !county.is_empty())
        .collect();
    if counties.is_empty() {
        None
    } else {
        Some(counties.join(", "))
    }
}

fn strip_trailing_texas(name: &str) -> String {
    let trimmed = name.trim().trim_end_matches(',').trim();
    trimmed
        .strip_suffix(", Texas")
        .or_else(|| trimmed.strip_suffix(", texas"))
        .unwrap_or(trimmed)
        .trim()
        .to_string()
}

fn extract_municipality(subdivision: &str) -> Option<String> {
    let lower = subdivision.to_ascii_lowercase();
    let rest = if lower.starts_with("city of ") {
        &subdivision["city of ".len()..]
    } else if lower.starts_with("town of ") {
        &subdivision["town of ".len()..]
    } else {
        return None;
    };
    let place = rest.trim();
    if place.is_empty() {
        None
    } else {
        Some(place.to_string())
    }
}

/// `"Bartlett Independent School District"` → `"Bartlett ISD"`.
fn normalize_school_district(school_district: &str) -> String {
    title_case(school_district.trim())
        .replace("Independent School District", "ISD")
        .replace("Common School District", "CSD")
}

#[cfg(test)]
mod tests {
    use super::{extract_ballot_measure_geography, extract_measure_name, normalize_counties};
    use db::{DistrictType, ElectionScope};

    #[test]
    fn captures_proposition_and_the_next_word() {
        assert_eq!(
            extract_measure_name("City of Austin Proposition A"),
            "Proposition A"
        );
        assert_eq!(
            extract_measure_name(
                "Austin Community College District - Annexation Proposition A"
            ),
            "Proposition A"
        );
        assert_eq!(
            extract_measure_name("City of China Grove Proposition"),
            "Proposition"
        );
    }

    #[test]
    fn city_and_town_measures_are_citywide() {
        let austin = extract_ballot_measure_geography("City of Austin");
        assert_eq!(austin.election_scope, ElectionScope::City);
        assert_eq!(austin.district_type, None);
        assert_eq!(austin.municipality.as_deref(), Some("Austin"));
        assert_eq!(austin.district, None);

        let town = extract_ballot_measure_geography("Town of Flower Mound");
        assert_eq!(town.election_scope, ElectionScope::City);
        assert_eq!(town.municipality.as_deref(), Some("Flower Mound"));
    }

    #[test]
    fn school_measures_use_the_isd_name_as_the_district() {
        let school = extract_ballot_measure_geography("Bartlett Independent School District");
        assert_eq!(school.election_scope, ElectionScope::District);
        assert_eq!(school.district_type, Some(DistrictType::School));
        assert_eq!(school.district.as_deref(), Some("Bartlett ISD"));
        assert_eq!(school.subdistrict, None);
    }

    #[test]
    fn utility_district_number_stays_on_the_district() {
        let mud =
            extract_ballot_measure_geography("Bexar County Municipal Utility District No. 1");
        assert_eq!(mud.election_scope, ElectionScope::District);
        assert_eq!(mud.district_type, Some(DistrictType::TxSpecial));
        assert_eq!(
            mud.district.as_deref(),
            Some("Bexar County Municipal Utility District 1")
        );
        assert_eq!(mud.subdistrict, None);
    }

    #[test]
    fn county_suffix_after_no_does_not_repeat_the_number() {
        let district = extract_ballot_measure_geography(
            "Cole Ranch Improvement District No. 2 of Denton County, Texas",
        );
        assert_eq!(
            district.district.as_deref(),
            Some("Cole Ranch Improvement District 2")
        );
        assert_eq!(district.subdistrict, None);

        let mud = extract_ballot_measure_geography(
            "Blue Meadow Municipal Utility District No. 2 of Collin County",
        );
        assert_eq!(
            mud.district.as_deref(),
            Some("Blue Meadow Municipal Utility District 2")
        );
    }

    #[test]
    fn emergency_services_district_keeps_a_bare_number() {
        let esd = extract_ballot_measure_geography("Emergency Services District 7");
        assert_eq!(esd.district_type, Some(DistrictType::TxSpecial));
        assert_eq!(esd.district.as_deref(), Some("Emergency Services District 7"));
        assert_eq!(esd.subdistrict, None);

        let lettered = extract_ballot_measure_geography("Emergency Services District 1-A");
        assert_eq!(
            lettered.district.as_deref(),
            Some("Emergency Services District 1-A")
        );
    }

    #[test]
    fn college_measures_use_the_college_district_type() {
        let college = extract_ballot_measure_geography("Austin Community College District");
        assert_eq!(college.district_type, Some(DistrictType::TxCollege));
        assert_eq!(college.district.as_deref(), Some("Austin Community College"));
        assert_eq!(college.subdistrict, None);
    }

    #[test]
    fn counties_separated_by_semicolons_are_stored_with_commas() {
        assert_eq!(
            normalize_counties("Travis; Williamson").as_deref(),
            Some("Travis, Williamson")
        );
        assert_eq!(normalize_counties("  ").as_deref(), None);
    }
}
