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
    pub school_district: Option<&'a str>,
    pub hospital_district: Option<&'a str>,
    pub municipality: Option<&'a str>,
    pub election_scope: Option<&'a db::ElectionScope>,
    pub district_type: Option<&'a db::DistrictType>,
}

impl<'a> OfficeSlugGenerator<'a> {
    pub fn generate(&self) -> String {
        let mut parts = vec![
            self.state.as_ref(),
            self.name.replace(".", ""),
        ];

        if let Some(county) = self.county {
            parts.push(county);
            parts.push("county");
        }

        if let Some(district) = self.district {
            parts.push(district);
        }

        if let Some(seat) = self.seat {
            parts.push(seat);
        }

        if let Some(school_district) = self.school_district {
            parts.push(school_district);
        }

        if let Some(hospital_district) = self.hospital_district {
            parts.push(hospital_district);
        }

        if let Some(municipality) = self.municipality {
            parts.push(municipality);
        }

        slugify!(&parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_slug() {
        let tests: Vec<(&'static str, OfficeSlugGenerator)> = vec![
            (
                "mn-us-senate-1",
                OfficeSlugGenerator {
                    name: "U.S. Senate",
                    state: &db::State::MN,
                    county: None,
                    district: Some("1"),
                    seat: None,
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-district-something-at-large",
                OfficeSlugGenerator {
                    name: "District Something",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: Some("At Large"),
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-district-something",
                OfficeSlugGenerator {
                    name: "District Something",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-county-judge-hennepin-county",
                OfficeSlugGenerator {
                    name: "County Judge",
                    state: &db::State::MN,
                    county: Some("Hennepin"),
                    district: None,
                    seat: None,
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-county-judge",
                OfficeSlugGenerator {
                    name: "County Judge",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-court-of-appeals-judge",
                OfficeSlugGenerator {
                    name: "Court of Appeals Judge",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-county-commissioner-hennepin-county-1-2",
                OfficeSlugGenerator {
                    name: "County Commissioner",
                    state: &db::State::MN,
                    county: Some("Hennepin"),
                    district: Some("1"),
                    seat: Some("2"),
                    school_district: None,
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-school-board-member-isd-535",
                OfficeSlugGenerator {
                    name: "School Board Member",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                    school_district: Some("ISD #535"),
                    hospital_district: None,
                    municipality: None,
                    election_scope: None,
                    district_type: None,
                },
            ),
            (
                "mn-hospital-district-board-member-northern-itasca-koochiching",
                OfficeSlugGenerator {
                    name: "Hospital District Board Member",
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                    school_district: None,
                    hospital_district: Some("Northern Itasca - Koochiching"),
                    municipality: None,
                    election_scope: None,
                    district_type: None,
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
                ("MN", "MN"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: None,
                },
            ),
            (
                ("Hennepin, MN", "Hennepin, MN"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: Some("Hennepin"),
                    district: None,
                    seat: None,
                },
            ),
            (
                ("MN - District 1", "MN - 1"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: None,
                    district: Some("1"),
                    seat: None,
                },
            ),
            (
                ("MN - Seat 2", "MN - 2"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: Some("2"),
                },
            ),
            (
                ("MN - At Large", "MN - At Large"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: None,
                    district: None,
                    seat: Some("At Large"),
                },
            ),
            (
                ("Hennepin, MN - District 1 - Seat 2", "Hennepin, MN - 1 - 2"),
                OfficeSubtitleGenerator {
                    state: &db::State::MN,
                    county: Some("Hennepin"),
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
