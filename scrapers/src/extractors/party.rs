use std::sync::OnceLock;

use regex::Regex;

static MATCHERS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();

pub fn extract_party_name(input: &str) -> Option<String> {
    let matchers = MATCHERS.get_or_init(|| {
        [
            (
                r"(?i:(?:Unaffiliated|No Party Affiliation))",
                "Unaffiliated",
            ),
            (r"(?i:Democratic Party)", "Democratic Party"),
            (r"(?i:Republican Party)", "Republican Party"),
            (r"(?i:Libertarian Party)", "Libertarian Party"),
            (
                r"(?i:American Constitution Party)",
                "American Constitution Party",
            ),
            (r"(?i:Center Party)", "Center Party"),
            (r"(?i:Unity Party)", "Unity Party"),
            (r"(?i:Forward Party)", "Forward Party"),
            (r"(?i:Approval Voting Party)", "Approval Voting Party"),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, name) in matchers {
        if matcher.is_match(input) {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::extensions::*;

    #[test]
    fn extract_name() {
        let tests: Vec<(&'static str, Option<&'static str>)> = vec![
            ("Party", None),
            ("Democrat", None),
            ("Democratic", None),
            ("Republican", None),
            ("Libertarian", None),
            ("American", None),
            ("Constitution", None),
            ("Center", None),
            ("Unity", None),
            ("Forward", None),
            ("Approval", None),
            ("Voting", None),
            ("Affiliation", None),
            // ----
            ("unaffiliated", Some("Unaffiliated")),
            ("no party affiliation", Some("Unaffiliated")),
            ("democratic party", Some("Democratic Party")),
            ("republican party", Some("Republican Party")),
            ("libertarian party", Some("Libertarian Party")),
            (
                "american constitution party",
                Some("American Constitution Party"),
            ),
            ("center party", Some("Center Party")),
            ("unity party", Some("Unity Party")),
            ("forward party", Some("Forward Party")),
            ("approval voting party", Some("Approval Voting Party")),
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_party_name(input).as_str(),
                expected,
                "\n\n  Test Case: '{input}'\n"
            );
        }
    }
}
