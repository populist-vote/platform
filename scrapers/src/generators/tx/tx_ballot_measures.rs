//! Texas ballot measure code and slug.

use crate::extractors::tx::tx_ballot_measures::BallotMeasureGeography;
use db::ElectionScope;
use slugify::slugify;
use std::collections::HashSet;
use std::error::Error;

/// City measures use `"Austin, TX"`. District measures use the district name.
/// Special elections append `" - Special Election"`.
pub fn generate_ballot_measure_code(
    geography: &BallotMeasureGeography,
    is_special_election: bool,
) -> Result<String, Box<dyn Error>> {
    let base = match geography.election_scope {
        ElectionScope::City => {
            let municipality = geography
                .municipality
                .as_deref()
                .filter(|name| !name.is_empty())
                .ok_or("City ballot measure is missing a municipality")?;
            format!("{municipality}, TX")
        }
        ElectionScope::District => geography
            .district
            .as_deref()
            .filter(|name| !name.is_empty())
            .ok_or("District ballot measure is missing a district")?
            .to_string(),
        other => return Err(format!("Unexpected election scope {other:?}").into()),
    };
    if is_special_election {
        Ok(format!("{base} - Special Election"))
    } else {
        Ok(base)
    }
}

/// `"tx"`, the election slug, the ballot measure code, and the measure name.
/// A repeated combination gets a `-2`, `-3`, … suffix.
pub fn generate_ballot_measure_slug(
    election_slug: &str,
    ballot_measure_code: &str,
    measure_name: &str,
    used_slugs: &mut HashSet<String>,
) -> String {
    let slug = slugify!(&format!(
        "tx {election_slug} {ballot_measure_code} {measure_name}"
    ));
    if used_slugs.insert(slug.clone()) {
        return slug;
    }
    let mut suffix = 2u32;
    loop {
        let candidate = format!("{slug}-{suffix}");
        if used_slugs.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_ballot_measure_code, generate_ballot_measure_slug};
    use crate::extractors::tx::tx_ballot_measures::BallotMeasureGeography;
    use db::ElectionScope;
    use std::collections::HashSet;

    fn geography(
        election_scope: ElectionScope,
        municipality: Option<&str>,
        district: Option<&str>,
    ) -> BallotMeasureGeography {
        BallotMeasureGeography {
            election_scope,
            district_type: None,
            district: district.map(str::to_string),
            subdistrict: None,
            municipality: municipality.map(str::to_string),
        }
    }

    #[test]
    fn city_code_uses_the_municipality() {
        let code = generate_ballot_measure_code(
            &geography(ElectionScope::City, Some("Austin"), None),
            false,
        )
        .unwrap();
        assert_eq!(code, "Austin, TX");
    }

    #[test]
    fn district_code_uses_the_district() {
        let code = generate_ballot_measure_code(
            &geography(ElectionScope::District, None, Some("Bartlett ISD")),
            false,
        )
        .unwrap();
        assert_eq!(code, "Bartlett ISD");
    }

    #[test]
    fn special_election_is_appended() {
        let code =
            generate_ballot_measure_code(&geography(ElectionScope::City, Some("Austin"), None), true)
                .unwrap();
        assert_eq!(code, "Austin, TX - Special Election");
    }

    #[test]
    fn slug_uses_the_election_code_and_measure_name() {
        let mut used = HashSet::new();
        let slug = generate_ballot_measure_slug(
            "general-election-2026",
            "Austin, TX",
            "Proposition A",
            &mut used,
        );
        assert_eq!(slug, "tx-general-election-2026-austin-tx-proposition-a");

        let special = generate_ballot_measure_slug(
            "general-election-2026",
            "Austin, TX - Special Election",
            "Proposition C",
            &mut used,
        );
        assert_eq!(
            special,
            "tx-general-election-2026-austin-tx-special-election-proposition-c"
        );
    }
}
