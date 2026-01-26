use slugify::slugify;

pub struct OfficeSubtitleGenerator<'a> {
    pub state: &'a db::State,
    pub office_name: Option<&'a str>,
    pub election_scope: &'a db::ElectionScope,
    pub district_type: Option<&'a db::DistrictType>,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
    pub school_district: Option<&'a str>,
    pub hospital_district: Option<&'a str>,
    pub municipality: Option<&'a str>,
}

impl<'a> OfficeSubtitleGenerator<'a> {
    pub fn generate(&self) -> (String, String) {
        use db::{ElectionScope, DistrictType};
        
        match self.election_scope {
            // State scope
            ElectionScope::State => {
                // Use full state name when no district/seat, or if U.S. Senate
                if self.office_name == Some("U.S. Senate") || (self.district.is_none() && self.seat.is_none()) {
                    ("Minnesota".to_string(), "Minnesota".to_string())
                } else if let Some(seat) = self.seat {
                    if seat.to_lowercase().contains("at large") {
                        (format!("MN - {}", seat), format!("MN - {}", seat))
                    } else {
                        (format!("MN - Seat {}", seat), format!("MN - {}", seat))
                    }
                } else {
                    ("MN".to_string(), "MN".to_string())
                }
            },
            
            // County scope
            ElectionScope::County => {
                if let Some(county) = self.county {
                    if let Some(district) = self.district {
                        (
                            format!("{} County, MN - District {}", county, district),
                            format!("{} County, MN - {}", county, district),
                        )
                    } else {
                        (
                            format!("{} County, MN", county),
                            format!("{} County, MN", county),
                        )
                    }
                } else {
                    ("MN".to_string(), "MN".to_string())
                }
            },
            
            // City scope
            ElectionScope::City => {
                if let Some(municipality) = self.municipality {
                    // Duplicate municipality names that need county specification
                    let duplicate_municipalities = [
                        "Beaver Township", "Becker Township", "Clover Township",
                        "Cornish Township", "Fairview Township", "Hillman Township",
                        "Lawrence Township", "Long Lake Township", "Louisville Township",
                        "Moose Lake Township", "Stokes Township", "Twin Lakes Township",
                    ];
                    
                    let seat_suffix = match self.seat {
                        None => String::new(),
                        Some(s) if s.to_lowercase().contains("at large") => format!(" - {}", s),
                        Some(s) => format!(" - Seat {}", s),
                    };
                    
                    if duplicate_municipalities.contains(&municipality) {
                        if let Some(county) = self.county {
                            (
                                format!("{} - {} County, MN{}", municipality, county, seat_suffix),
                                format!("{} - {} County, MN{}", municipality, county, seat_suffix),
                            )
                        } else {
                            (format!("{}, MN{}", municipality, seat_suffix), format!("{}, MN{}", municipality, seat_suffix))
                        }
                    } else {
                        (format!("{}, MN{}", municipality, seat_suffix), format!("{}, MN{}", municipality, seat_suffix))
                    }
                } else {
                    ("MN".to_string(), "MN".to_string())
                }
            },
            
            // District scope - depends on district_type
            ElectionScope::District => {
                match self.district_type {
                    Some(DistrictType::UsCongressional) => {
                        if let Some(district) = self.district {
                            (format!("MN - District {}", district), format!("MN - {}", district))
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::StateHouse) => {
                        if let Some(district) = self.district {
                            (format!("MN - House District {}", district), format!("MN - {}", district))
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::StateSenate) => {
                        if let Some(district) = self.district {
                            (format!("MN - Senate District {}", district), format!("MN - {}", district))
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::County) => {
                        if let (Some(county), Some(district)) = (self.county, self.district) {
                            (
                                format!("{} County, MN - District {}", county, district),
                                format!("{} County, MN - {}", county, district),
                            )
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::City) => {
                        if let Some(municipality) = self.municipality {
                            if let Some(district) = self.district {
                                // Check if district is purely numeric
                                if district.chars().all(|c| c.is_numeric()) {
                                    (
                                        format!("{}, MN - District {}", municipality, district),
                                        format!("{}, MN - {}", municipality, district),
                                    )
                                } else {
                                    // For short subtitle, remove specific prefixes: "Ward", "Wards", "Section", "Precinct"
                                    let district_short = district
                                        .replace("Ward ", "")
                                        .replace("Wards ", "")
                                        .replace("Section ", "")
                                        .replace("Precinct ", "")
                                        .trim()
                                        .to_string();
                                    (
                                        format!("{}, MN - {}", municipality, district),
                                        if district_short.is_empty() {
                                            format!("{}, MN", municipality)
                                        } else {
                                            format!("{}, MN - {}", municipality, district_short)
                                        },
                                    )
                                }
                            } else {
                                (format!("{}, MN", municipality), format!("{}, MN", municipality))
                            }
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::School) => {
                        if let Some(school_district) = self.school_district {
                            let seat_suffix = match self.seat {
                                None => String::new(),
                                Some(s) if s.to_lowercase().contains("at large") => format!(" - {}", s),
                                Some(s) => format!(" - Seat {}", s),
                            };
                            
                            if let Some(district) = self.district {
                                // Check if district is purely numeric
                                if district.chars().all(|c| c.is_numeric()) {
                                    (
                                        format!("MN - {} - District {}{}", school_district, district, seat_suffix),
                                        format!("MN - {} - {}{}", school_district, district, seat_suffix),
                                    )
                                } else {
                                    (
                                        format!("MN - {} - {}{}", school_district, district, seat_suffix),
                                        format!("MN - {} - {}{}", school_district, district, seat_suffix),
                                    )
                                }
                            } else {
                                (
                                    format!("MN - {}{}", school_district, seat_suffix),
                                    format!("MN - {}{}", school_district, seat_suffix),
                                )
                            }
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::Judicial) => {
                        if let Some(seat) = self.seat {
                            (format!("MN - Seat {}", seat), format!("MN - {}", seat))
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::Hospital) => {
                        if let Some(hospital_district) = self.hospital_district {
                            let seat_suffix = match self.seat {
                                None => String::new(),
                                Some(s) if s.to_lowercase().contains("at large") => format!(" - {}", s),
                                Some(s) => format!(" - Seat {}", s),
                            };
                            
                            if let Some(district) = self.district {
                                // Check if district is purely numeric
                                if district.chars().all(|c| c.is_numeric()) {
                                    (
                                        format!("{} - District {}{}", hospital_district, district, seat_suffix),
                                        format!("{} - {}{}", hospital_district, district, seat_suffix),
                                    )
                                } else {
                                    (
                                        format!("{}{}", hospital_district, seat_suffix),
                                        format!("{}{}", hospital_district, seat_suffix),
                                    )
                                }
                            } else {
                                (
                                    format!("{}{}", hospital_district, seat_suffix),
                                    format!("{}{}", hospital_district, seat_suffix),
                                )
                            }
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::SoilAndWater) => {
                        if let (Some(county), Some(district)) = (self.county, self.district) {
                            (
                                format!("{} County, MN - District {}", county, district),
                                format!("{} County, MN - {}", county, district),
                            )
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    Some(DistrictType::Park) => {
                        if let Some(municipality) = self.municipality {
                            if let Some(district) = self.district {
                                // Check if district is purely numeric
                                if district.chars().all(|c| c.is_numeric()) {
                                    (
                                        format!("{}, MN - District {}", municipality, district),
                                        format!("{}, MN - {}", municipality, district),
                                    )
                                } else {
                                    (
                                        format!("{}, MN - {}", municipality, district),
                                        format!("{}, MN - {}", municipality, district),
                                    )
                                }
                            } else {
                                (format!("{}, MN", municipality), format!("{}, MN", municipality))
                            }
                        } else {
                            ("MN".to_string(), "MN".to_string())
                        }
                    },
                    _ => ("MN".to_string(), "MN".to_string()),
                }
            },
            
            // National scope
            ElectionScope::National => ("".to_string(), "".to_string()),
        }
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
        use db::{ElectionScope, DistrictType};
        
        // Replace special characters in name
        let cleaned_name = self.name.replace(".", "").replace("&", "and");
        
        // Build slug string based on election scope and district type
        let slug_text = match self.election_scope {
            // State scope: mn + name + district + seat
            Some(ElectionScope::State) => {
                format!(
                    "{} {} {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            },
            
            // County scope: mn + name + county + "county" + district + seat
            Some(ElectionScope::County) => {
                format!(
                    "{} {} {} county {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    self.county.unwrap_or(""),
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            },
            
            // City scope: mn + name + municipality + county + "county" + school_district + district + seat
            Some(ElectionScope::City) => {
                let municipality_cleaned = self.municipality
                    .map(|m| m.replace("Township", "Twp"))
                    .unwrap_or_default();
                
                format!(
                    "{} {} {} {} county {} {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    municipality_cleaned,
                    self.county.unwrap_or(""),
                    self.school_district.unwrap_or(""),
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            },
            
            // District scope - varies by district_type
            Some(ElectionScope::District) => {
                match self.district_type {
                    Some(DistrictType::County) => {
                        format!(
                            "{} {} {} county {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.county.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::City) => {
                        let municipality_cleaned = self.municipality
                            .map(|m| m.replace("Township", "Twp"))
                            .unwrap_or_default();
                        
                        format!(
                            "{} {} {} {} county {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            municipality_cleaned,
                            self.county.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::School) => {
                        format!(
                            "{} {} {} {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.school_district.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::Judicial) => {
                        // Don't include district in slug for judicial - it's in the name
                        format!(
                            "{} {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::Hospital) => {
                        format!(
                            "{} {} {} {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.hospital_district.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::SoilAndWater) => {
                        format!(
                            "{} {} {} county {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.county.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    Some(DistrictType::Park) => {
                        let municipality_cleaned = self.municipality
                            .map(|m| m.replace("Township", "Twp"))
                            .unwrap_or_default();
                        
                        format!(
                            "{} {} {} {} county {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            municipality_cleaned,
                            self.county.unwrap_or(""),
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                    // UsCongressional, StateHouse, StateSenate, Transportation
                    _ => {
                        format!(
                            "{} {} {} {}",
                            self.state.as_ref(),
                            cleaned_name,
                            self.district.unwrap_or(""),
                            self.seat.unwrap_or(""),
                        )
                    },
                }
            },
            
            // National or None
            _ => String::new(),
        };
        
        slugify!(&slug_text)
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
