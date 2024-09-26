use slugify::slugify;

use crate::util::extensions::*;

#[derive(Default)]
pub struct OfficeSlugGenerator<'a> {
    pub name: Option<&'a str>,
    pub state: Option<&'a db::State>,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
    pub election_scope: Option<&'a db::ElectionScope>,
}

impl<'a> OfficeSlugGenerator<'a> {
    pub fn from_source(input: &'a db::UpsertOfficeInput) -> Self {
        OfficeSlugGenerator {
            name: input.name.as_str(),
            state: input.state.as_ref(),
            county: input.county.as_str(),
            district: input.district.as_str(),
            seat: input.seat.as_str(),
            election_scope: input.election_scope.as_ref(),
        }
    }

    pub fn generate(&self) -> String {
        let format = format!(
            "{} {} {}",
            super::optional_state_str(self.state),
            self.name
                .map(|n| n.replace(".", ""))
                .as_str_unwrapped_or_empty(),
            match self.election_scope {
                Some(db::ElectionScope::County) => self.county.unwrap_or_default(),
                _ =>
                    if self.district.is_some() {
                        self.district.unwrap_or_default()
                    } else {
                        self.seat.unwrap_or_default()
                    },
            }
        );
        slugify!(&format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_slug() {
        let tests: Vec<(&'static str, OfficeSlugGenerator)> = vec![
            (
                "co-us-senate-1",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some(&"U.S. Senate"),
                    seat: Some("1"),
                    ..Default::default()
                },
            ),
            (
                "co-us-house-1",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("U.S. House"),
                    election_scope: Some(&db::ElectionScope::District),
                    district: Some("1"),
                    ..Default::default()
                },
            ),
            (
                "co-district-something-at-large",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("District Something"),
                    election_scope: Some(&db::ElectionScope::District),
                    district: None,
                    seat: Some("At Large"),
                    ..Default::default()
                },
            ),
            (
                "co-district-something",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("District Something"),
                    election_scope: Some(&db::ElectionScope::District),
                    // no district or seat specified
                    ..Default::default()
                },
            ),
            (
                "co-county-judge-adams",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("County Judge"),
                    election_scope: Some(&db::ElectionScope::County),
                    county: Some("Adams"),
                    ..Default::default()
                },
            ),
            (
                "co-county-judge",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("County Judge"),
                    election_scope: Some(&db::ElectionScope::County),
                    // no county specified
                    ..Default::default()
                },
            ),
            (
                "co-court-of-appeals-judge",
                OfficeSlugGenerator {
                    state: Some(&db::State::CO),
                    name: Some("Court of Appeals Judge"),
                    ..Default::default()
                },
            ),
        ];

        for (expected, generator) in tests {
            assert_eq!(expected, generator.generate());
        }
    }
}
