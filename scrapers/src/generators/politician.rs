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

/// Builds politician / race_candidate ref keys.
/// Format (slugified): `{source}-{election_slug}-{office_title}-{candidate_name}`
pub struct PoliticianRefKeyGenerator<'a> {
    source: &'a str,
    election_slug: &'a str,
    office_title: &'a str,
    candidate_name: Option<&'a str>,
}

impl<'a> PoliticianRefKeyGenerator<'a> {
    pub fn new(
        source: &'a str,
        election_slug: &'a str,
        office_title: &'a str,
        candidate_name: Option<&'a str>,
    ) -> Self {
        PoliticianRefKeyGenerator {
            source,
            election_slug,
            office_title,
            candidate_name,
        }
    }

    pub fn generate(&self) -> String {
        slugify!(&format!(
            "{}-{}-{}-{}",
            self.source,
            self.election_slug,
            self.office_title,
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

    #[test]
    fn politician_ref_key() {
        assert_eq!(
            PoliticianRefKeyGenerator::new(
                "mn-sos",
                "2026-primary-election",
                "Governor",
                Some("Jane Doe")
            )
            .generate(),
            "mn-sos-2026-primary-election-governor-jane-doe"
        );

        assert_eq!(
            PoliticianRefKeyGenerator::new(
                "tx-primaries",
                "2026-general-election",
                "U. S. REPRESENTATIVE DISTRICT 1",
                Some("JANE DOE")
            )
            .generate(),
            "tx-primaries-2026-general-election-u-s-representative-district-1-jane-doe"
        );

        // Empty office title still produces a valid slug
        assert_eq!(
            PoliticianRefKeyGenerator::new("CO-SOS", "2024-general-election", "", Some("john-smith"))
                .generate(),
            "co-sos-2024-general-election-john-smith"
        );
    }
}
