mod errors;
pub use crate::errors::Error;
use std::{env, fmt, str::FromStr};
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub web_app_url: Url,
    pub root_domain: String,
}

impl Default for Config {
    fn default() -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "local".into());
        let environment = Environment::from_str(&environment).unwrap();
        let web_app_url = match environment {
            Environment::Production => Url::parse("https://www.populist.us").unwrap(),
            Environment::Staging => Url::parse("https://staging.populist.us").unwrap(),
            _ => Url::parse("http://localhost:3030").unwrap(),
        };
        let root_domain = match environment {
            Environment::Production => "populist.us".to_string(),
            Environment::Staging => "staging.populist.us".to_string(),
            _ => "localhost".to_string(),
        };
        Config {
            environment,
            web_app_url,
            root_domain,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Environment {
    Production,
    Staging,
    Development,
    Local,
    Test,
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
            Ok(Environment::Test)
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
                Environment::Test => "test",
                Environment::Unknown => "unknown",
                Environment::Local => "local",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            Environment::from_str("production").unwrap(),
            Environment::Production
        );
        assert_eq!(
            Environment::from_str("staging").unwrap(),
            Environment::Staging
        );
        assert_eq!(
            Environment::from_str("development").unwrap(),
            Environment::Development
        );
        assert_eq!(Environment::from_str("test").unwrap(), Environment::Test);
        assert_eq!(Environment::from_str("local").unwrap(), Environment::Local);
        assert_eq!(
            Environment::from_str("unknown").unwrap(),
            Environment::Unknown
        );
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.environment, Environment::Local);
        assert_eq!(config.web_app_url.to_string(), "http://localhost:3030/");
    }
}
