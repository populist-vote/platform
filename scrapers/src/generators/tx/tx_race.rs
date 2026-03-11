//! Texas race title and slug generator.
//! Follows MN pattern: TX - <office_name> - <subtitle> - [Special Election -] <race_type> [- party] - <year>.

use std::sync::OnceLock;

use regex::Regex;
use slugify::slugify;

pub struct RaceTitleGenerator<'a> {
    pub race_type: &'a db::RaceType,
    pub election_scope: &'a db::ElectionScope,
    pub office_name: Option<&'a str>,
    pub office_subtitle: Option<&'a str>,
    pub state: Option<&'a db::State>,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
    pub is_special_election: bool,
    pub party: Option<&'a str>,
    pub year: i32,
}

impl<'a> RaceTitleGenerator<'a> {
    pub fn from_source(
        r#type: &'a db::RaceType,
        office: &'a db::Office,
        is_special_election: bool,
        party: Option<&'a str>,
        year: i32,
    ) -> Self {
        Self {
            race_type: r#type,
            election_scope: &office.election_scope,
            office_name: office.name.as_deref(),
            office_subtitle: office.subtitle.as_deref(),
            state: office.state.as_ref(),
            county: office.county.as_deref(),
            district: office.district.as_deref(),
            seat: office.seat.as_deref(),
            is_special_election,
            party,
            year,
        }
    }

    pub fn generate(&self) -> (String, String) {
        let mut parts = Vec::new();
        parts.push("TX".to_string());

        if let Some(name) = self.office_name {
            if !name.is_empty() {
                parts.push(name.to_string());
            }
        }

        if let Some(subtitle) = self.office_subtitle {
            if !subtitle.is_empty() {
                let cleaned = if subtitle.starts_with("TX - ") {
                    &subtitle[5..]
                } else {
                    subtitle
                };
                if !cleaned.is_empty() {
                    parts.push(cleaned.to_string());
                }
            }
        }

        if self.is_special_election {
            parts.push("Special Election".to_string());
        }

        let race_type_str = match self.race_type {
            db::RaceType::Primary => match self.party {
                Some("R") | Some("REP") => "Primary - Republican".to_string(),
                Some("D") | Some("DEM") => "Primary - Democratic".to_string(),
                Some(p) if !p.is_empty() => format!("Primary - {}", p),
                _ => "Primary".to_string(),
            },
            db::RaceType::General => "General".to_string(),
        };
        parts.push(race_type_str);
        parts.push(self.year.to_string());

        let title = parts.join(" - ");
        static REGEX: OnceLock<Regex> = OnceLock::new();
        let re = REGEX.get_or_init(|| Regex::new(r"  +").unwrap());
        let title = re.replace_all(&title, " ").trim().to_string();
        let slug = slugify!(&title.replace(".", ""));
        (title, slug)
    }
}
