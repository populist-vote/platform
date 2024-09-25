use std::error::Error;

use chrono::{Days, NaiveDate, Weekday};
use slugify::slugify;

pub struct GeneralElectionDateGenerator {
    pub year: u16,
}

impl GeneralElectionDateGenerator {
    pub fn new(year: u16) -> Self {
        GeneralElectionDateGenerator { year }
    }

    // Reference: https://en.wikipedia.org/wiki/Election_Day_(United_States)
    // "The Tuesday after the first Monday of November"
    pub fn generate(&self) -> Result<NaiveDate, Box<dyn Error>> {
        let error = || {
            format!(
                "Unable to determine general election date for year: {}",
                self.year
            )
        };

        let first_monday =
            NaiveDate::from_weekday_of_month_opt(self.year as _, 11, Weekday::Mon, 1)
                .ok_or_else(error)?;
        let next_tuesday = first_monday
            .checked_add_days(Days::new(1))
            .ok_or_else(error)?;
        Ok(next_tuesday)
    }
}

pub struct ElectionTitleGenerator<'a> {
    pub r#type: &'a db::RaceType,
    pub year: u16,
}

impl<'a> ElectionTitleGenerator<'a> {
    pub fn new(r#type: &'a db::RaceType, year: u16) -> Self {
        ElectionTitleGenerator { r#type, year }
    }

    pub fn generate(&self) -> (String, String) {
        match self.r#type {
            db::RaceType::General => {
                let title = format!("General Election {}", self.year);
                let slug = slugify!(&title);
                (title, slug)
            }
            db::RaceType::Primary => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    #[test]
    fn general_election_date() {
        let tests: Vec<((u32, u32), u16)> = vec![
            ((11, 7), 2023),
            ((11, 5), 2024),
            ((11, 4), 2025),
            ((11, 3), 2026),
            ((11, 2), 2027),
            ((11, 7), 2028),
        ];

        for (expected, input) in tests {
            let date = GeneralElectionDateGenerator::new(input).generate().unwrap();
            assert_eq!(expected, (date.month(), date.day()));
        }
    }

    #[test]
    fn general_election_title() {
        let tests: Vec<((&'static str, &'static str), u16)> = vec![
            (("General Election 2024", "general-election-2024"), 2024),
            (("General Election 2025", "general-election-2025"), 2025),
        ];

        for (expected, input) in tests {
            let actual = ElectionTitleGenerator::new(&db::RaceType::General, input).generate();
            assert_eq!(expected.0, actual.0);
            assert_eq!(expected.1, actual.1);
        }
    }
}
