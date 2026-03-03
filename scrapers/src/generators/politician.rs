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

pub struct PoliticianRefKeyGenerator<'a> {
    source: &'a str,
    election_year: i32,
    office_title: &'a str,
    candidate_name: Option<&'a str>,
}

impl<'a> PoliticianRefKeyGenerator<'a> {
    pub fn new(
        source: &'a str,
        election_year: i32,
        office_title: &'a str,
        candidate_name: Option<&'a str>,
    ) -> Self {
        PoliticianRefKeyGenerator {
            source,
            election_year,
            office_title,
            candidate_name,
        }
    }

    pub fn generate(&self) -> String {
        let mut parts: Vec<String> = vec![self.source.to_string()];
        if self.election_year != 0 {
            parts.push(self.election_year.to_string());
        }
        if !self.office_title.is_empty() {
            parts.push(self.office_title.to_string());
        }
        if let Some(name) = self.candidate_name {
            if !name.is_empty() {
                parts.push(name.to_string());
            }
        }
        let combined: String = parts.join("-");
        slugify!(&combined)
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
        // Source + candidate name only (year=0, office_title="") -> same as legacy source+slug
        let tests: Vec<((&'static str, &'static str), &'static str)> = vec![
            (("CO SOS", "john-smith"), "co-sos-john-smith"),
            (("MN CSV", "john-smith"), "mn-csv-john-smith"),
        ];

        for (input, expected) in tests {
            assert_eq!(
                PoliticianRefKeyGenerator::new(input.0, 0, "", Some(input.1)).generate(),
                expected
            );
        }

        // TX primaries: source + year + office_title + candidate_name
        assert_eq!(
            PoliticianRefKeyGenerator::new(
                "tx-primaries",
                2026,
                "U. S. REPRESENTATIVE DISTRICT 1",
                Some("JANE DOE")
            )
            .generate(),
            "tx-primaries-2026-u-s-representative-district-1-jane-doe"
        );
    }
}
