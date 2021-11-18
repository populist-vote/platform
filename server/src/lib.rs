mod errors;
pub use crate::errors::Error;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Environment {
    Production,
    Staging,
    Development,
    Local,
    IntegrationTest,
    Unknown,
}

impl FromStr for Environment {
    type Err = crate::errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = s.to_lowercase();
        if e.contains("production") {
            Ok(Environment::Production)
        } else if e.contains("staging") {
            Ok(Environment::Staging)
        } else if e.contains("dev") {
            Ok(Environment::Development)
        } else if e.contains("test") {
            Ok(Environment::IntegrationTest)
        } else if e.contains("local") {
            Ok(Environment::Local)
        } else {
            //no need to crash if we set an arbitrary value for some reason
            println!(
                "Unable to resolve environment from {}. Setting to unknown",
                e
            );
            Ok(Environment::Unknown)
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Environment::Staging => "staging",
                Environment::Production => "production",
                Environment::Development => "development",
                Environment::IntegrationTest => "test",
                Environment::Unknown => "unknown",
                Environment::Local => "local",
            }
        )
    }
}
