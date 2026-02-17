//! Texas office slug and subtitle generators.
//! Mirrors MN generators with TX / Texas naming.

use slugify::slugify;

/// Returns the priority for the given office title. Matches populist-office-titles-map.csv (office_title → priority).
pub fn office_priority(office_title: &str, county: Option<&str>) -> Option<i32> {
    let priority = match office_title.trim() {
        "U.S. President" => 1,
        "U.S. Vice President" => 2,
        "U.S. Senator" => 3,
        "U.S. Representative" => 4,
        "Governor" => 5,
        "Lieutenant Governor" => 6,
        "Secretary of State" => 7,
        "Attorney General" => 8,
        "State Senator" => 10,
        "State Representative" => 11,
        "Board of Education" => 12,
        "Railroad Commissioner"
        | "Commissioner of Agriculture"
        | "Commissioner of the General Land Office"
        | "Comptroller of Public Accounts" => 13,
        "Mayor" => 20,
        "City Council" => 21,
        "County Commissioner" => 22,
        "Justice of the Peace" => 23,
        "County Constable" => 24,
        "County Clerk"
        | "County & District Clerk"
        | "County Attorney"
        | "County Treasurer"
        | "County Surveyor"
        | "County Tax Assessor-Collector"
        | "County Judge"
        | "District Clerk"
        | "Department of Education" => 25,
        "District Attorney" => {
            if county.is_some() {
                25
            } else {
                46
            }
        }
        "Sheriff" => 26,
        "County Chair (D)" | "County Chair (R)" => 27,
        "Precinct Chair (D)" | "Precinct Chair (R)" => 28,
        "Chief Justice - Supreme Court" => 40,
        "Justice - Supreme Court" => 41,
        "Chief Justice - Court of Appeals" => 42,
        "Justice - Court of Appeals" => 43,
        "Judge - Court of Criminal Appeals" => 44,
        "Judge - District Court" => 45,
        "Judge - County Court at Law"
        | "Judge - 1st Multicounty Court at Law"
        | "Judge - County Civil Court at Law"
        | "Judge - County Criminal Court at Law"
        | "Judge - County Criminal Court of Appeals"
        | "Judge - Criminal District"
        | "Judge - Probate Court" => 50,
        _ => return None,
    };
    Some(priority)
}

/// Formats district value for precinct subtitle: "1, 2" → "Precinct 1 & 2", "1, 2, 3" → "Precinct 1, 2, 3", "1" → "Precinct 1".
fn format_precinct_subtitle(district: &str) -> String {
    let numbers: Vec<&str> = district
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .collect();
    match numbers.len() {
        0 => format!("{}", district.trim()),
        1 => format!("{}", numbers[0]),
        2 => format!("{} & {}", numbers[0], numbers[1]),
        _ => format!("{}", numbers.join(", ")),
    }
}

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
        use db::{DistrictType, ElectionScope};

        match self.election_scope {
            ElectionScope::State => {
                if self.office_name == Some("U.S. Senate")
                    || (self.district.is_none() && self.seat.is_none())
                {
                    ("Texas".to_string(), "TX".to_string())
                } else if let Some(seat) = self.seat {
                    if seat.to_lowercase().contains("at large") {
                        (format!("TX - {}", seat), format!("TX - {}", seat))
                    } else {
                        (format!("TX - Seat {}", seat), format!("TX - {}", seat))
                    }
                } else {
                    ("Texas".to_string(), "TX".to_string())
                }
            }

            ElectionScope::County => {
                if let Some(county) = self.county {
                    if let Some(district) = self.district {
                        let is_court = self
                            .office_name
                            .map(|n| n.to_lowercase().contains("court"))
                            .unwrap_or(false);
                        if is_court {
                            (
                                format!("{} County, TX - Court {}", county, district),
                                format!("{} County, TX - {}", county, district),
                            )
                        } else {
                            (
                                format!("{} County, TX - District {}", county, district),
                                format!("{} County, TX - {}", county, district),
                            )
                        }
                    } else {
                        (
                            format!("{} County, TX", county),
                            format!("{} County, TX", county),
                        )
                    }
                } else {
                    ("Texas".to_string(), "TX".to_string())
                }
            }

            ElectionScope::District => match self.district_type {
                Some(DistrictType::UsCongressional) => {
                    if let Some(district) = self.district {
                        (
                            format!("TX - District {}", district),
                            format!("TX - {}", district),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::StateHouse) => {
                    if let Some(district) = self.district {
                        (
                            format!("TX - House District {}", district),
                            format!("TX - {}", district),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::StateSenate) => {
                    if let Some(district) = self.district {
                        (
                            format!("TX - Senate District {}", district),
                            format!("TX - {}", district),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::County) => {
                    if let (Some(county), Some(district)) = (self.county, self.district) {
                        let precinct_label = format_precinct_subtitle(district);
                        (
                            format!("{} County, TX - Precinct {}", county, precinct_label),
                            format!("{} County, TX - {}", county, precinct_label),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::Judicial) => {
                    if let Some(district) = self.district {
                        let seat_suffix = self
                            .seat
                            .as_ref()
                            .map(|s| (format!(" - Seat {}", s), format!(" - {}", s)))
                            .unwrap_or_else(|| (String::new(), String::new()));
                        (
                            format!("TX - District {}{}", district, seat_suffix.0),
                            format!("TX - District {}{}", district, seat_suffix.1),
                        )
                    } else if let Some(seat) = self.seat {
                        (format!("TX - Seat {}", seat), format!("TX - {}", seat))
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::School) | Some(DistrictType::BoardOfEducation) => {
                    if let Some(district) = self.district {
                        (
                            format!(
                                "TX - District {}{}",
                                district,
                                self.seat
                                    .as_ref()
                                    .map(|s| format!(" - Place {}", s))
                                    .unwrap_or_default()
                            ),
                            format!(
                                "TX - {}{}",
                                district,
                                self.seat
                                    .as_ref()
                                    .map(|s| format!(" - {}", s))
                                    .unwrap_or_default()
                            ),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::CourtOfAppeals) => {
                    if let Some(district) = self.district {
                        let seat_suffix = self
                            .seat
                            .as_ref()
                            .map(|s| (format!(" - Seat {}", s), format!(" - {}", s)))
                            .unwrap_or_else(|| (String::new(), String::new()));
                        (
                            format!("TX - District {}{}", district, seat_suffix.0),
                            format!("TX - District {}{}", district, seat_suffix.1),
                        )
                    } else if let Some(seat) = self.seat {
                        (format!("TX - Seat {}", seat), format!("TX - {}", seat))
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                Some(DistrictType::JusticeOfThePeace)
                | Some(DistrictType::Constable)
                | Some(DistrictType::VotingPrecinct) => {
                    if let (Some(county), Some(district)) = (self.county, self.district) {
                        let precinct_label = format_precinct_subtitle(district);
                        (
                            format!("{} County, TX - Precinct {}", county, precinct_label),
                            format!("{} County, TX - {}", county, precinct_label),
                        )
                    } else {
                        ("TX".to_string(), "TX".to_string())
                    }
                }
                _ => ("TX".to_string(), "TX".to_string()),
            },

            ElectionScope::National => ("".to_string(), "".to_string()),
            ElectionScope::City => {
                if let Some(municipality) = self.municipality {
                    let seat_suffix = self
                        .seat
                        .as_ref()
                        .map(|s| format!(" - {}", s))
                        .unwrap_or_default();
                    (
                        format!("{}, TX{}", municipality, seat_suffix),
                        format!("{}, TX{}", municipality, seat_suffix),
                    )
                } else {
                    ("TX".to_string(), "TX".to_string())
                }
            }
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
        use db::{DistrictType, ElectionScope};

        let cleaned_name = self.name.replace(".", "").replace("&", "and");

        if self.name == "Judge - 1st Multicounty Court at Law" {
            return slugify!(&format!("tx-{}", cleaned_name));
        }

        let county_with_label = self
            .county
            .filter(|c| !c.is_empty())
            .map(|c| format!("{} county", c))
            .unwrap_or_default();

        let slug_text = match self.election_scope {
            Some(ElectionScope::State) => {
                format!(
                    "{} {} {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            }

            Some(ElectionScope::County) => {
                format!(
                    "{} {} {} {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    county_with_label,
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            }

            Some(ElectionScope::District) => match self.district_type {
                Some(DistrictType::County)
                | Some(DistrictType::JusticeOfThePeace)
                | Some(DistrictType::Constable)
                | Some(DistrictType::VotingPrecinct) => {
                    format!(
                        "{} {} {} {} {}",
                        self.state.as_ref(),
                        cleaned_name,
                        county_with_label,
                        self.district.unwrap_or(""),
                        self.seat.unwrap_or(""),
                    )
                }
                Some(DistrictType::Judicial) => {
                    format!(
                        "{} {} {} {}",
                        self.state.as_ref(),
                        cleaned_name,
                        self.district.unwrap_or(""),
                        self.seat.unwrap_or(""),
                    )
                }
                Some(DistrictType::School) => {
                    format!(
                        "{} {} {} {} {}",
                        self.state.as_ref(),
                        cleaned_name,
                        self.school_district.unwrap_or(""),
                        self.district.unwrap_or(""),
                        self.seat.unwrap_or(""),
                    )
                }
                Some(DistrictType::CourtOfAppeals) | Some(DistrictType::BoardOfEducation) => {
                    format!(
                        "{} {} {} {}",
                        self.state.as_ref(),
                        cleaned_name,
                        self.district.unwrap_or(""),
                        self.seat.unwrap_or(""),
                    )
                }
                _ => {
                    format!(
                        "{} {} {} {}",
                        self.state.as_ref(),
                        cleaned_name,
                        self.district.unwrap_or(""),
                        self.seat.unwrap_or(""),
                    )
                }
            },

            _ => {
                format!(
                    "{} {} {} {} {}",
                    self.state.as_ref(),
                    cleaned_name,
                    self.county.unwrap_or(""),
                    self.district.unwrap_or(""),
                    self.seat.unwrap_or(""),
                )
            }
        };

        let slug_text = slug_text.split_whitespace().collect::<Vec<_>>().join(" ");
        slugify!(&slug_text)
    }
}
