//! Texas race-related extractors from office title strings.

use regex::Regex;
use std::sync::OnceLock;

/// Returns true if the title indicates a special or unexpired-term election.
pub fn extract_is_special_election(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("special election") || lower.contains("unexpired term")
}

/// Returns the number of positions to elect if present (e.g. "(Elect 3)").
pub fn extract_num_elect(input: &str) -> Option<i32> {
    static ELECT_REGEX: OnceLock<Regex> = OnceLock::new();
    let re = ELECT_REGEX.get_or_init(|| Regex::new(r"\(Elect\s+(\d+)\)").unwrap());
    re.captures(input)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}
