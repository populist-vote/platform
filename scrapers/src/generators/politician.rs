use db::Office;
use serde_json::{Map, Value as JsonValue};
use slugify::slugify;

const THUMBNAIL_BASE: &str =
    "https://populist-platform.s3.us-east-2.amazonaws.com/web-assets/politician-thumbnails";

/// Builds the profile picture assets JSON for a politician: thumbnailImage160 and thumbnailImage400
/// URLs using the given slug. Use as the `assets` field when creating/updating a politician.
pub fn politician_thumbnail_assets(slug: &str) -> JsonValue {
    let mut map = Map::new();
    map.insert(
        "thumbnailImage160".to_string(),
        JsonValue::String(format!("{}/{}-160.jpg", THUMBNAIL_BASE, slug)),
    );
    map.insert(
        "thumbnailImage400".to_string(),
        JsonValue::String(format!("{}/{}-400.jpg", THUMBNAIL_BASE, slug)),
    );
    JsonValue::Object(map)
}

pub struct PoliticianSlugGenerator<'a> {
    pub name: &'a str,
    pub state: Option<&'a str>,
}

impl<'a> PoliticianSlugGenerator<'a> {
    pub fn new(name: &'a str) -> Self {
        PoliticianSlugGenerator { name, state: None }
    }

    /// When state is `Some("TX")`, the generated slug has "-tx" appended (e.g. "john-smith-tx").
    pub fn with_state(mut self, state: &'a str) -> Self {
        self.state = Some(state);
        self
    }

    pub fn generate(&self) -> String {
        let base = slugify!(self.name);
        if self
            .state
            .map(|s| s.eq_ignore_ascii_case("TX"))
            .unwrap_or(false)
        {
            format!("{}-tx", base)
        } else {
            base
        }
    }
}

/// `{name}|{subtitle}` used as the office component of a politician / race_candidate ref key.
pub fn office_identity_for_ref_key(office: &Office) -> String {
    format!(
        "{}|{}",
        office.name.as_deref().unwrap_or(""),
        office.subtitle.as_deref().unwrap_or(""),
    )
}

/// Builds politician / race_candidate ref keys.
/// Format (slugified): `{source}-{election_slug}-{office_identity}-{candidate_name}`
pub struct PoliticianRefKeyGenerator<'a> {
    source: &'a str,
    election_slug: &'a str,
    office_identity: String,
    candidate_name: Option<&'a str>,
}

impl<'a> PoliticianRefKeyGenerator<'a> {
    pub fn new(
        source: &'a str,
        election_slug: &'a str,
        office: &Office,
        candidate_name: Option<&'a str>,
    ) -> Self {
        PoliticianRefKeyGenerator {
            source,
            election_slug,
            office_identity: office_identity_for_ref_key(office),
            candidate_name,
        }
    }

    /// String identity path used by scrapers that do not have an `Office` (MN, CO, results).
    pub fn from_office_title(
        source: &'a str,
        election_slug: &'a str,
        office_title: &'a str,
        candidate_name: Option<&'a str>,
    ) -> Self {
        PoliticianRefKeyGenerator {
            source,
            election_slug,
            office_identity: office_title.to_string(),
            candidate_name,
        }
    }

    pub fn generate(&self) -> String {
        slugify!(&format!(
            "{}-{}-{}-{}",
            self.source,
            self.election_slug,
            self.office_identity,
            self.candidate_name.unwrap_or("")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn politician_slug() {
        assert_eq!(
            PoliticianSlugGenerator::new("John Smith").generate(),
            "john-smith"
        );
        assert_eq!(
            PoliticianSlugGenerator::new("John Smith")
                .with_state("TX")
                .generate(),
            "john-smith-tx"
        );
    }

    fn office_for_test(name: &str, subtitle: Option<&str>) -> Office {
        Office {
            id: uuid::Uuid::nil(),
            slug: String::new(),
            title: String::new(),
            subtitle: subtitle.map(str::to_string),
            subtitle_short: None,
            name: Some(name.to_string()),
            office_type: None,
            district: None,
            subdistrict: None,
            district_type: None,
            hospital_district: None,
            school_district: None,
            chamber: None,
            political_scope: db::PoliticalScope::Federal,
            election_scope: db::ElectionScope::National,
            state: None,
            county: None,
            municipality: None,
            term_length: None,
            seat: None,
            priority: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn politician_ref_key() {
        let office = office_for_test("Governor", Some("TX"));
        assert_eq!(
            PoliticianRefKeyGenerator::new(
                "mn-sos",
                "2026-primary-election",
                &office,
                Some("Jane Doe")
            )
            .generate(),
            "mn-sos-2026-primary-election-governor-tx-jane-doe"
        );

        let office = office_for_test("U. S. REPRESENTATIVE DISTRICT 1", Some("TX"));
        assert_eq!(
            PoliticianRefKeyGenerator::new(
                "tx-primaries",
                "2026-general-election",
                &office,
                Some("JANE DOE")
            )
            .generate(),
            "tx-primaries-2026-general-election-u-s-representative-district-1-tx-jane-doe"
        );

        // Empty office title still produces a valid slug
        assert_eq!(
            PoliticianRefKeyGenerator::from_office_title(
                "CO-SOS",
                "2024-general-election",
                "",
                Some("john-smith")
            )
            .generate(),
            "co-sos-2024-general-election-john-smith"
        );
    }

    #[test]
    fn office_identity_uses_name_and_subtitle() {
        let office = office_for_test("School Board", Some("Austin ISD, TX"));
        assert_eq!(
            office_identity_for_ref_key(&office),
            "School Board|Austin ISD, TX"
        );
    }
}
