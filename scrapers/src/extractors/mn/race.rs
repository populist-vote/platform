/// Extracts race-related information from Minnesota office titles
/// 
/// This module provides extractors for race metadata that can be derived
/// from the office_title string in Minnesota candidate filings.

use regex::Regex;
use std::sync::OnceLock;

/// Checks if the office title indicates a special election
/// 
/// # Arguments
/// * `input` - The office title string (e.g., "Mayor (Special Election)")
/// 
/// # Returns
/// * `true` if the office title contains "Special Election" (case-insensitive)
/// * `false` otherwise
/// 
/// # Examples
/// ```
/// use scrapers::extractors::mn::race::extract_is_special_election;
/// 
/// assert_eq!(extract_is_special_election("Mayor (Special Election)"), true);
/// assert_eq!(extract_is_special_election("Mayor - Special Election"), true);
/// assert_eq!(extract_is_special_election("SPECIAL ELECTION Mayor"), true);
/// assert_eq!(extract_is_special_election("Mayor"), false);
/// assert_eq!(extract_is_special_election("City Council Member"), false);
/// ```
pub fn extract_is_special_election(input: &str) -> bool {
    input.to_lowercase().contains("special election")
}

/// Extracts the number of positions to be elected from the office title
/// 
/// # Arguments
/// * `input` - The office title string (e.g., "City Council Member (Elect 3)")
/// 
/// # Returns
/// * `Some(number)` if the office title contains "(Elect X)" where X is a number
/// * `None` if the pattern is not found or X is not a valid number
/// 
/// # Examples
/// ```
/// use scrapers::extractors::mn::race::extract_num_elect;
/// 
/// assert_eq!(extract_num_elect("City Council Member (Elect 3)"), Some(3));
/// assert_eq!(extract_num_elect("School Board Member (Elect 2)"), Some(2));
/// assert_eq!(extract_num_elect("Mayor (Elect 1)"), Some(1));
/// assert_eq!(extract_num_elect("Mayor"), None);
/// assert_eq!(extract_num_elect("City Council Member"), None);
/// assert_eq!(extract_num_elect("City Council Member (Elect)"), None);
/// ```
pub fn extract_num_elect(input: &str) -> Option<i32> {
    static ELECT_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = ELECT_REGEX.get_or_init(|| {
        Regex::new(r"\(Elect\s+(\d+)\)").unwrap()
    });
    
    regex.captures(input)
        .and_then(|captures| captures.get(1))
        .and_then(|match_| match_.as_str().parse::<i32>().ok())
}

/// Checks if the office title indicates a ranked choice voting race
/// 
/// # Arguments
/// * `input` - The office title string (e.g., "Mayor - First Choice")
/// 
/// # Returns
/// * `true` if the office title contains "First Choice" (case-insensitive)
/// * `false` otherwise
/// 
/// # Examples
/// ```
/// use scrapers::extractors::mn::race::extract_is_ranked_choice;
/// 
/// assert_eq!(extract_is_ranked_choice("Mayor - First Choice"), true);
/// assert_eq!(extract_is_ranked_choice("Mayor (First Choice)"), true);
/// assert_eq!(extract_is_ranked_choice("First Choice Mayor"), true);
/// assert_eq!(extract_is_ranked_choice("Mayor"), false);
/// assert_eq!(extract_is_ranked_choice("City Council Member"), false);
/// ```
pub fn extract_is_ranked_choice(input: &str) -> bool {
    input.to_lowercase().contains("first choice")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_is_special_election() {
        let tests = vec![
            // Special elections - various formats
            ("Mayor (Special Election)", true),
            ("Mayor - Special Election", true),
            ("Special Election Mayor", true),
            ("City Council Member Special Election", true),
            ("County Commissioner (Special Election)", true),
            ("State Representative District 5 (Special Election)", true),
            
            // Case insensitive
            ("SPECIAL ELECTION Mayor", true),
            ("special election mayor", true),
            ("Mayor (SPECIAL ELECTION)", true),
            ("Mayor (SpEcIaL ElEcTiOn)", true),
            
            // Not special elections
            ("Mayor", false),
            ("City Council Member", false),
            ("County Commissioner", false),
            ("State Representative District 5", false),
            ("U.S. Representative District 1", false),
            
            // Edge cases
            ("", false),
            ("Special", false), // Only part of the phrase
            ("Election", false), // Only part of the phrase
            ("Special Elections", true), // Plural form should still match
            ("Mayor (Election)", false), // Just "Election" without "Special"
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_is_special_election(input),
                expected,
                "\n\n  Test Case: '{}'\n  Expected: {}\n  Got: {}\n",
                input,
                expected,
                extract_is_special_election(input)
            );
        }
    }

    #[test]
    fn test_extract_num_elect() {
        let tests = vec![
            // Valid Elect patterns
            ("City Council Member (Elect 3)", Some(3)),
            ("School Board Member (Elect 2)", Some(2)),
            ("Mayor (Elect 1)", Some(1)),
            ("County Commissioner (Elect 5)", Some(5)),
            ("Hospital Board Member (Elect 7)", Some(7)),
            
            // Case variations
            ("City Council Member (elect 3)", Some(3)),
            ("City Council Member (ELECT 3)", Some(3)),
            ("City Council Member (Elect 3)", Some(3)),
            
            // Whitespace variations
            ("City Council Member (Elect  3)", Some(3)),
            ("City Council Member (Elect\t3)", Some(3)),
            ("City Council Member (Elect\n3)", Some(3)),
            
            // No Elect pattern
            ("Mayor", None),
            ("City Council Member", None),
            ("County Commissioner", None),
            ("School Board Member", None),
            
            // Invalid Elect patterns
            ("City Council Member (Elect)", None), // No number
            ("City Council Member (Elect X)", None), // Non-numeric
            ("City Council Member (Elect 3.5)", None), // Decimal
            ("City Council Member (Elect -3)", None), // Negative
            ("City Council Member (Elect 0)", Some(0)), // Zero (valid)
            
            // Edge cases
            ("", None),
            ("(Elect 3)", Some(3)), // Just the pattern
            ("Elect 3", None), // Missing parentheses
            ("City Council Member (Elect 3) (Special Election)", Some(3)), // Multiple patterns
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_num_elect(input),
                expected,
                "\n\n  Test Case: '{}'\n  Expected: {:?}\n  Got: {:?}\n",
                input,
                expected,
                extract_num_elect(input)
            );
        }
    }

    #[test]
    fn test_extract_is_ranked_choice() {
        let tests = vec![
            // Ranked choice elections - various formats
            ("Mayor - First Choice", true),
            ("Mayor (First Choice)", true),
            ("First Choice Mayor", true),
            ("City Council Member First Choice", true),
            ("County Commissioner - First Choice", true),
            ("Mayor First Choice Special Election", true),
            
            // Case insensitive
            ("FIRST CHOICE Mayor", true),
            ("first choice mayor", true),
            ("Mayor (FIRST CHOICE)", true),
            ("Mayor (FiRsT ChOiCe)", true),
            
            // Not ranked choice
            ("Mayor", false),
            ("City Council Member", false),
            ("County Commissioner", false),
            ("State Representative District 5", false),
            ("U.S. Representative District 1", false),
            
            // Edge cases
            ("", false),
            ("First", false), // Only part of the phrase
            ("Choice", false), // Only part of the phrase
            ("Mayor (Second Choice)", false), // Different choice
            ("Mayor (Choice)", false), // Missing "First"
            ("Mayor First", false), // Missing "Choice"
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_is_ranked_choice(input),
                expected,
                "\n\n  Test Case: '{}'\n  Expected: {}\n  Got: {}\n",
                input,
                expected,
                extract_is_ranked_choice(input)
            );
        }
    }
}