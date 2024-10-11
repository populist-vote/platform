use slugify::slugify;

pub struct OfficeSubtitleGenerator<'a> {
    pub state: &'a db::State,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
}

impl<'a> OfficeSubtitleGenerator<'a> {
    pub fn generate(&self) -> (String, String) {
        let mut subtitle = self.state.to_string();
        let mut subtitle_short = self.state.to_string();
        if let Some(county) = self.county {
            subtitle = format!("{}, {}", county, subtitle);
            subtitle_short = format!("{}, {}", county, subtitle_short);
        }
        if let Some(district) = self.district {
            subtitle = format!("{} - District {}", subtitle, district);
            subtitle_short = format!("{} - {}", subtitle_short, district);
        }
        if let Some(seat) = self.seat {
            if seat == "At Large" {
                subtitle = format!("{} - {}", subtitle, seat);
            } else {
                subtitle = format!("{} - Seat {}", subtitle, seat);
            }
            subtitle_short = format!("{} - {}", subtitle_short, seat);
        }
        (subtitle, subtitle_short)
    }
}

pub struct OfficeSlugGenerator<'a> {
    pub state: &'a db::State,
    pub name: &'a str,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
}

impl<'a> OfficeSlugGenerator<'a> {
    pub fn generate(&self) -> String {
        let format = format!(
            "{} {} {} {} {} {}",
            self.state.as_ref(),
            self.name.replace(".", ""),
            self.county.unwrap_or_default(),
            self.county.as_ref().map(|_| "county").unwrap_or_default(),
            self.district.unwrap_or_default(),
            self.seat.unwrap_or_default(),
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
                    name: "U.S. Senate",
                    state: &db::State::CO,
                    county: None,
                    district: Some("1"),
                    seat: None,
                },
            ),
            (
                "co-district-something-at-large",
                OfficeSlugGenerator {
                    name: "District Something",
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: Some("At Large"),
                },
            ),
            (
                "co-district-something",
                OfficeSlugGenerator {
                    name: "District Something",
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: None,
                },
            ),
            (
                "co-county-judge-adams-county",
                OfficeSlugGenerator {
                    name: "County Judge",
                    state: &db::State::CO,
                    county: Some("Adams"),
                    district: None,
                    seat: None,
                },
            ),
            (
                "co-county-judge",
                OfficeSlugGenerator {
                    name: "County Judge",
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: None,
                },
            ),
            (
                "co-court-of-appeals-judge",
                OfficeSlugGenerator {
                    name: "Court of Appeals Judge",
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: None,
                },
            ),
            (
                "co-county-commissioner-adams-county-1-2",
                OfficeSlugGenerator {
                    name: "County Commissioner",
                    state: &db::State::CO,
                    county: Some("Adams"),
                    district: Some("1"),
                    seat: Some("2"),
                },
            ),
        ];

        for (expected, generator) in tests {
            assert_eq!(expected, generator.generate());
        }
    }

    #[test]
    fn office_subtitle() {
        let tests: Vec<((&'static str, &'static str), OfficeSubtitleGenerator)> = vec![
            (
                ("CO", "CO"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: None,
                },
            ),
            (
                ("Adams, CO", "Adams, CO"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: Some("Adams"),
                    district: None,
                    seat: None,
                },
            ),
            (
                ("CO - District 1", "CO - 1"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: None,
                    district: Some("1"),
                    seat: None,
                },
            ),
            (
                ("CO - Seat 2", "CO - 2"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: Some("2"),
                },
            ),
            (
                ("CO - At Large", "CO - At Large"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: None,
                    district: None,
                    seat: Some("At Large"),
                },
            ),
            (
                ("Adams, CO - District 1 - Seat 2", "Adams, CO - 1 - 2"),
                OfficeSubtitleGenerator {
                    state: &db::State::CO,
                    county: Some("Adams"),
                    district: Some("1"),
                    seat: Some("2"),
                },
            ),
        ];

        for (expected, generator) in tests {
            let subtitle = generator.generate();
            assert_eq!(expected.0, subtitle.0);
            assert_eq!(expected.1, subtitle.1);
        }
    }
}
