use std::sync::OnceLock;

use regex::Regex;

use super::owned_capture;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoliticianName {
    pub first: String,
    pub last: Option<String>,
    pub middle: Option<String>,
    pub preferred: Option<String>,
    pub suffix: Option<String>,
}

static NAME_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static SUFFIX_REGEX: OnceLock<Regex> = OnceLock::new();
static LAST_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

/// Extracts and removes suffix from the end of a name string
/// Returns (name_without_suffix, suffix_if_found)
fn extract_suffix(input: &str) -> (String, Option<String>) {
    let suffix_regex = SUFFIX_REGEX.get_or_init(|| {
        Regex::new(r#",?\s+(?<suffix>[iIvVxX]+|[jJsS][rR]\.?|[jJsS][rR])$"#).unwrap()
    });
    
    if let Some(captures) = suffix_regex.captures(input) {
        if let Some(suffix_match) = captures.name("suffix") {
            let mut suffix = suffix_match.as_str().to_string();
            
            // Normalize "Jr" and "Sr" to "Jr." and "Sr." if they don't have periods
            let suffix_lower = suffix.to_lowercase();
            if (suffix_lower == "jr" || suffix_lower == "sr") && !suffix.ends_with('.') {
                suffix.push('.');
            }
            
            // Find the start of the entire match (including comma and space)
            let match_start = captures.get(0).unwrap().start();
            let name_without_suffix = input[..match_start].trim().to_string();
            return (name_without_suffix, Some(suffix));
        }
    }
    (input.to_string(), None)
}

/// Extracts and removes last name from the end of a name string
/// Handles compound last names with "Van" or "St."/"St" (case-insensitive)
/// Returns (name_without_last, last_name_if_found)
fn extract_last_name(input: &str) -> (String, Option<String>) {
    let compound_regex = LAST_NAME_REGEX.get_or_init(|| {
        // Pattern matches compound last names: "Van Last" or "St. Last" or "St Last" (case-insensitive)
        Regex::new(r#"(?i)\s+(van|st\.?)\s+([\w\.'-]+)$"#).unwrap()
    });
    
    // First, check for compound last names (Van or St./St)
    if let Some(captures) = compound_regex.captures(input) {
        if let (Some(prefix), Some(last_part)) = (captures.get(1), captures.get(2)) {
            let prefix_str = prefix.as_str();
            let last_part_str = last_part.as_str();
            // Reconstruct with proper formatting for prefix
            let prefix_formatted = if prefix_str.eq_ignore_ascii_case("van") {
                // Preserve original casing of "van"
                prefix_str.to_string()
            } else if prefix_str.ends_with('.') {
                // "St." already has period, preserve it
                prefix_str.to_string()
            } else {
                // "St" without period, add period
                format!("{}.", prefix_str)
            };
            let last_name = format!("{} {}", prefix_formatted, last_part_str);
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }
    
    // If no compound name, just extract the last word (excluding middle initials like "H.")
    let words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last_word) = words.last() {
        // Exclude middle initials (single letter + period)
        if last_word.len() > 2 || (last_word.len() == 2 && !last_word.ends_with('.')) {
            let last_name = last_word.to_string();
            let name_without_last = words[..words.len() - 1].join(" ");
            return (name_without_last, Some(last_name));
        }
    }
    
    (input.to_string(), None)
}

pub fn extract_politician_name(input: &str) -> Option<PoliticianName> {
    // First, extract and remove suffix
    let (name_without_suffix, suffix) = extract_suffix(input);
    
    // Then, extract and remove last name
    let (name_without_last, last_name) = extract_last_name(&name_without_suffix);
    
    let extractors = NAME_EXTRACTORS.get_or_init(|| {
        // Regular expressions are broken into multiple lines for readability
        [
            // Reference: https://regex101.com/r/xKvi7n/3
            // Adapted from https://regex101.com/library/7zjSTN
            [
                r#"^"#,
                r#"(?<first>[\w\.']+)"#,
                r#"(?: +(?<middle1>[\w\.']+))?"#,
                r#"(?: *["\(](?<preferred>[\w\.' ]+)["\)])?"#,
                r#"(?: +(?<middle2>[\w\.']+))?"#,
                r#"$"#,
            ],
        ]
        .into_iter()
        .map(|r| Regex::new(&r.join("")).unwrap())
        .collect()
    });

    for extractor in extractors {
        if let Some(captures) = extractor.captures(&name_without_last) {
            if let Some(first) = captures.name("first").map(owned_capture) {
                // Extract and normalize middle name
                let middle = captures
                    .name("middle2")
                    .or(captures.name("middle1"))
                    .map(owned_capture)
                    .map(|m| {
                        // If middle name is a single letter without period, add period
                        if m.len() == 1 && !m.ends_with('.') {
                            format!("{}.", m)
                        } else {
                            m
                        }
                    });
                
                return Some(PoliticianName {
                    first,
                    last: last_name,
                    middle,
                    preferred: captures.name("preferred").map(owned_capture),
                    suffix,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn extract() {
        let tests: Vec<(&'static str, Option<PoliticianName>)> = vec![
            //("John", None),
            (
                "John Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Jingleheimer Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("Jingleheimer".into()),
                    ..Default::default()
                }),
            ),
            (
                "John \"Jacob\" Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    preferred: Some("Jacob".into()),
                    ..Default::default()
                }),
            ),
            (
                "John \"Jacob\" Jingleheimer Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("Jingleheimer".into()),
                    preferred: Some("Jacob".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Jingleheimer \"Jacob\" O'Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("O'Schmidt".into()),
                    middle: Some("Jingleheimer".into()),
                    preferred: Some("Jacob".into()),
                    ..Default::default()
                }),
            ),
            (
                "J. Schmidt",
                Some(PoliticianName {
                    first: "J.".into(),
                    last: Some("Schmidt".into()),
                    ..Default::default()
                }),
            ),
            (
                "John J. Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("J.".into()),
                    ..Default::default()
                }),
            ),
            (
                "J.J. Schmidt",
                Some(PoliticianName {
                    first: "J.J.".into(),
                    last: Some("Schmidt".into()),
                    ..Default::default()
                }),
            ),
            (
                "John J.J. Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("J.J.".into()),
                    ..Default::default()
                }),
            ),
            (
                "John O'Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("O'Schmidt".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Jingleheimer-Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Jingleheimer-Schmidt".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Jingleheimer of Johannesburg Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("Jingleheimer of Johannesburg".into()),
                    ..Default::default()
                }),
            ),
            (
                "John \"Jacob\" Jingleheimer of Johannesburg Schmidt",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("Jingleheimer of Johannesburg".into()),
                    preferred: Some("Jacob".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Schmidt, Jr.",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    suffix: Some("Jr.".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Schmidt jr",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    suffix: Some("Jr".into()),
                    ..Default::default()
                }),
            ),
            (
                "John Schmidt, sr.",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    suffix: Some("sr".into()),
                    ..Default::default()
                }),
            ),
            (
                "John \"Jacob\" Jingleheimer of Johannesburg Schmidt, VII",
                Some(PoliticianName {
                    first: "John".into(),
                    last: Some("Schmidt".into()),
                    middle: Some("Jingleheimer of Johannesburg".into()),
                    preferred: Some("Jacob".into()),
                    suffix: Some("VII".into()),
                }),
            ),
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_politician_name(input),
                expected,
                "\n\n  Test Case: '{input}'\n"
            );
        }
    }
}
