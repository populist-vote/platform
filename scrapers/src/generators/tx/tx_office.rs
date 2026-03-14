//! Texas office slug and subtitle generators.
//! Mirrors MN generators with TX / Texas naming.

use slugify::slugify;

/// Generates an office state id from a source and office title (e.g. for ingest staging).
/// Returns a slugified string: `slugify("{source}-{office_title}")`.
pub fn office_state_id(source: &str, office_title: &str) -> String {
    slugify!(&format!("{}-{}", source, office_title))
}

/// Returns the priority for the given office title. Matches populist-office-titles-map.csv (office_title → priority).
pub fn office_priority(
    office_title: &str,
    _county: Option<&str>,
    district: Option<&str>,
) -> Option<i32> {
    let priority = match office_title.trim() {
        // Federal Offices
        "U.S. President" => 1,
        "U.S. Vice President" => 2,
        "U.S. Senator" => 3,
        "U.S. Representative" => 4,

        // State Executive Offices
        "Governor" => 5,
        "Lieutenant Governor" => 6,
        "Secretary of State" => 7,
        "Attorney General" => 8,
        "Comptroller of Public Accounts" => 9,
        "Commissioner of the General Land Office" => 10,
        "Commissioner of Agriculture" => 11,
        "Railroad Commissioner" => 12,

        "Chief Justice - Supreme Court" => 15,
        "Justice - Supreme Court" => 16,
        "Judge - Court of Criminal Appeals" => 17,

        "State Board of Education" => 18,
        "State Senator" => 19,
        "State Representative" => 20,

        "Chief Justice - Court of Appeals" => {
            if district == Some("15") {
                21
            } else {
                23
            }
        }
        "Justice - Court of Appeals" => {
            if district == Some("15") {
                22
            } else {
                24
            }
        }

        "District Judge" => 25,
        "District Attorney" => 26,
        "Criminal District Judge" => 27,
        "Criminal District Attorney" => 28,
        "County Judge" => 30,
        "Judge - County Court at Law" => 31,
        "Judge - 1st Multicounty Court at Law" => 32,
        "Judge - County Civil Court at Law" => 35,
        "Judge - County Criminal Court of Appeals" => 36,
        "Judge - County Criminal Court at Law" => 37,
        "Judge - Probate Court" => 38,

        "County Attorney" => 40,
        "District Clerk" => 41,
        "County Clerk" => 42,
        "County & District Clerk" => 42,
        "Sheriff" => 44,
        "County Tax Assessor-Collector" => 45,
        "County Treasurer" => 46,
        "County Surveyor" => 47,
        "County School Trustee" => 48,

        "County Commissioner" => 50,
        "Justice of the Peace" => 51,
        "County Constable" => 52,

        "County Chair (D)" => 55,
        "County Chair (R)" => 55,
        "Precinct Chair (D)" | "Precinct Chair (R)" => 56,

        "Mayor" => 60,
        "City Council" => 61,

        _ => return None,
    };
    Some(priority)
}

/// Formats district value for display in a list. Splits only on commas so values like "19-1" stay intact.
/// "1, 2" → "1 & 2", "1, 2, 3" → "1, 2, 3", "19-1" → "19-1".
fn format_district_list(district: &str) -> String {
    let parts: Vec<&str> = district.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    match parts.len() {
        0 => district.trim().to_string(),
        1 => parts[0].to_string(),
        2 => format!("{} & {}", parts[0], parts[1]),
        _ => parts.join(", "),
    }
}

/// Returns (long, short) district suffix. If district is "at large" (case insensitive), omits the label.
fn format_district_subtitle(district: &str, label: &str) -> (String, String) {
    let formatted = format_district_list(district);
    let short = format!(" - {}", formatted);
    if district.trim().eq_ignore_ascii_case("at large") {
        (short.clone(), short)
    } else {
        (format!(" - {} {}", label, formatted), short)
    }
}

/// Returns (long, short) seat suffix. If seat is "at large" (case insensitive), omits the label (Seat/Place).
fn format_seat_subtitle(seat: &str, label: &str) -> (String, String) {
    let short = format!(" - {}", seat);
    if seat.trim().eq_ignore_ascii_case("at large") {
        (short.clone(), short)
    } else {
        (format!(" - {} {}", label, seat), short)
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

        // County School Trustee: county + " County, TX" + format_seat_subtitle(seat, "Position") + district (before election_scope match)
        if self.office_name == Some("County School Trustee") {
            if let Some(county) = self.county {
                let (seat_long, seat_short) = self.seat
                    .as_ref()
                    .map(|s| format_seat_subtitle(s, "Position"))
                    .unwrap_or_default();
                let (district_long, district_short) = self.district
                    .map(|d| format_district_subtitle(d, "Precinct"))
                    .unwrap_or_default();
                let long = format!("{} County, TX{}{}", county, seat_long, district_long).trim_end().to_string();
                let short = format!("{} County, TX{}{}", county, seat_short, district_short).trim_end().to_string();
                return (long, short);
            }
        }

        match self.election_scope {
            ElectionScope::State => {
                if self.office_name == Some("U.S. Senate")
                    || (self.district.is_none() && self.seat.is_none())
                {
                    ("Texas".to_string(), "TX".to_string())
                } else if let (Some(district), Some(seat)) = (self.district, self.seat) {
                    let (district_long, district_short) = format_district_subtitle(district, "District");
                    let (seat_long, seat_short) = format_seat_subtitle(seat, "Seat");
                    (
                        format!("TX{}{}", district_long, seat_long),
                        format!("TX{}{}", district_short, seat_short),
                    )
                } else if let Some(seat) = self.seat {
                    let (long, short) = format_seat_subtitle(seat, "Seat");
                    (format!("TX{}", long), format!("TX{}", short))
                } else {
                    ("Texas".to_string(), "TX".to_string())
                }
            }

            ElectionScope::County => {
                let (seat_long, seat_short) = self.seat
                    .as_ref()
                    .map(|s| format_seat_subtitle(s, "Seat"))
                    .unwrap_or_default();
                if let Some(county) = self.county {
                    if let Some(district) = self.district {
                        let is_court = self
                            .office_name
                            .map(|n| n.to_lowercase().contains("court"))
                            .unwrap_or(false);
                        let (district_long, district_short) = format_district_subtitle(
                            district,
                            if is_court { "Court" } else { "District" },
                        );
                        (
                            format!("{} County, TX{}{}", county, district_long, seat_long),
                            format!("{} County, TX{}{}", county, district_short, seat_short),
                        )
                    } else {
                        (
                            format!("{} County, TX{}", county, seat_long),
                            format!("{} County, TX{}", county, seat_short),
                        )
                    }
                } else {
                    (
                        format!("Texas{}", seat_long),
                        format!("TX{}", seat_short),
                    )
                }
            }

            ElectionScope::District => {
                match self.district_type {
                    Some(DistrictType::UsCongressional) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            (format!("TX{}", district_long), format!("TX{}", district_short))
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::StateHouse) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "House District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            (format!("TX{}", district_long), format!("TX{}", district_short))
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::StateSenate) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "Senate District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            (format!("TX{}", district_long), format!("TX{}", district_short))
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::County) => {
                        if let Some(county) = self.county {
                            let (district_long, district_short) = self.district
                                .map(|d| format_district_subtitle(d, "Precinct"))
                                .unwrap_or_default();
                            let (seat_long, seat_short) = self.seat
                                .as_ref()
                                .map(|s| format_seat_subtitle(s, "Place"))
                                .unwrap_or_default();
                            if self.district.is_some() {
                                (
                                    format!("{} County, TX{}{}", county, district_long, seat_long),
                                    format!("{} County, TX{}{}", county, district_short, seat_short),
                                )
                            } else {
                                (
                                    format!("{} County, TX{}", county, seat_long),
                                    format!("{} County, TX{}", county, seat_short),
                                )
                            }
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::Judicial) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            let (seat_long, seat_short) = self.seat
                                .as_ref()
                                .map(|s| format_seat_subtitle(s, "Seat"))
                                .unwrap_or_else(|| (String::new(), String::new()));
                            (
                                format!("TX{}{}", district_long, seat_long),
                                format!("TX{}{}", district_short, seat_short),
                            )
                        } else if let Some(seat) = self.seat {
                            let (long, short) = format_seat_subtitle(seat, "Seat");
                            (format!("TX{}", long), format!("TX{}", short))
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::School) | Some(DistrictType::BoardOfEducation) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            let (seat_long, seat_short) = self.seat
                                .as_ref()
                                .map(|s| format_seat_subtitle(s, "Place"))
                                .unwrap_or_default();
                            (
                                format!("TX{}{}", district_long, seat_long),
                                format!("TX{}{}", district_short, seat_short),
                            )
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::CourtOfAppeals) => {
                        let (district_long, district_short) = self.district
                            .map(|d| format_district_subtitle(d, "District"))
                            .unwrap_or_default();
                        if self.district.is_some() {
                            let (seat_long, seat_short) = self.seat
                                .as_ref()
                                .map(|s| format_seat_subtitle(s, "Seat"))
                                .unwrap_or_else(|| (String::new(), String::new()));
                            (
                                format!("TX{}{}", district_long, seat_long),
                                format!("TX{}{}", district_short, seat_short),
                            )
                        } else if let Some(seat) = self.seat {
                            let (long, short) = format_seat_subtitle(seat, "Seat");
                            (format!("TX{}", long), format!("TX{}", short))
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    Some(DistrictType::JusticeOfThePeace) | Some(DistrictType::Constable) | Some(DistrictType::VotingPrecinct) => {
                        if let Some(county) = self.county {
                            let base = format!("{} County, TX", county);
                            let (district_long, district_short) = self.district
                                .map(|d| format_district_subtitle(d, "Precinct"))
                                .unwrap_or_default();
                            let (seat_long, seat_short) = self.seat
                                .as_ref()
                                .map(|s| format_seat_subtitle(s, "Place"))
                                .unwrap_or_default();
                            (
                                format!("{}{}{}", base, district_long, seat_long),
                                format!("{}{}{}", base, district_short, seat_short),
                            )
                        } else {
                            ("TX".to_string(), "TX".to_string())
                        }
                    }
                    _ => ("TX".to_string(), "TX".to_string()),
                }
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

        if self.name == "County School Trustee" {
            let slug_text = format!(
                "{} {} {} {} {}",
                self.state.as_ref(),
                cleaned_name,
                county_with_label,
                self.seat.unwrap_or(""),
                self.district.unwrap_or(""),
            );
            return slugify!(&slug_text.split_whitespace().collect::<Vec<_>>().join(" "));
        }

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
