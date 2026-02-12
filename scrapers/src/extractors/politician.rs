use std::sync::OnceLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use super::owned_capture;

// --- title_case (for name/office string normalization) ---

/// Roman numeral letter strings that we do not treat as roman numerals (e.g. initials), case-insensitive.
const ROMAN_NUMERAL_EXCLUSIONS: &[&str] = &["vi", "xi", "di", "mi", "im", "id", "ci", "li"];

/// True if the string is non-empty and only contains roman numeral letters (I, V, X, L, C, D, M), case-insensitive.
/// Excludes strings in ROMAN_NUMERAL_EXCLUSIONS (e.g. "VI", "XI").
fn is_roman_numeral(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if ROMAN_NUMERAL_EXCLUSIONS
        .iter()
        .any(|x| s.eq_ignore_ascii_case(x))
    {
        return false;
    }
    s.chars().all(|c| {
        matches!(c, 'I' | 'i' | 'V' | 'v' | 'X' | 'x' | 'L' | 'l' | 'C' | 'c' | 'D' | 'd' | 'M' | 'm')
    })
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U')
}

fn is_consonant(c: char) -> bool {
    c.is_ascii_alphabetic() && !is_vowel(c)
}

/// Two consonants that should stay title case (e.g. "JR" -> "Jr"), case-insensitive match.
const TWO_CONSONANT_EXCEPTIONS: &[&str] = &["jr", "sr", "dr"];

/// Mixed (vowel+consonant or other) two-letter strings that should be all caps, case-insensitive match.
const MIXED_TWO_LETTER_ALL_CAPS: &[&str] = &["aj", "oj"];

fn is_two_consonants(s: &str) -> bool {
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(a), Some(b)) if it.next().is_none() => is_consonant(a) && is_consonant(b),
        _ => false,
    }
}

/// True if a two-letter string should be all caps: in MIXED_TWO_LETTER_ALL_CAPS, or two consonants and not in TWO_CONSONANT_EXCEPTIONS.
fn is_two_letter_all_caps(s: &str) -> bool {
    if s.chars().count() != 2 {
        return false;
    }
    let s_lower = s.to_lowercase();
    if MIXED_TWO_LETTER_ALL_CAPS.iter().any(|x| x == &s_lower) {
        return true;
    }
    is_two_consonants(s) && !TWO_CONSONANT_EXCEPTIONS.iter().any(|x| s_lower.eq_ignore_ascii_case(x))
}

/// Title-case a single word (first char upper, rest lower).
fn word_to_title(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c.flat_map(|c| c.to_lowercase())).collect(),
    }
}

/// Title-case a phrase: each space-separated token is split on '-' or '.', each part is title-cased, rejoined with the same delimiter.
/// Tokens that are already quoted ("...") or parenthesized ((...)) are left unchanged so the inner content is not re-processed.
fn title_case_phrase(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let w = w.trim();
            // Protect quoted/paren tokens so we don't re-process them (e.g. "(Dr." and "OJ)" from "(Dr. OJ)")
            if (w.starts_with('"') && w.ends_with('"'))
                || (w.starts_with('(') && w.ends_with(')'))
                || w.starts_with('(')
                || w.ends_with(')')
            {
                return w.to_string();
            }
            w.split('-')
                .map(|part| {
                    part.split('.')
                        .map(|seg| {
                            if is_roman_numeral(seg) {
                                seg.to_uppercase()
                            } else if is_two_letter_all_caps(seg) {
                                seg.to_uppercase()
                            } else {
                                word_to_title(seg)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(".")
                })
                .collect::<Vec<_>>()
                .join("-")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

static QUOTED_OR_PAREN: OnceLock<Regex> = OnceLock::new();

/// Title-case a string (e.g. "HARRIS COUNTY" -> "Harris County"). Title-cases words on either side of '-'
/// (e.g. "SE-GWEN" -> "Se-Gwen") and title-cases content inside "..." or (...). Roman numerals are normalized to uppercase.
pub(crate) fn title_case(s: &str) -> String {
    // Normalize ("X") to (X) first so the quote/paren regex doesn't match (" with empty capture and break nesting.
    let s = strip_quotes_inside_parens(s);
    let re = QUOTED_OR_PAREN.get_or_init(|| Regex::new(r#"[("]([^")]*)[")]"#).unwrap());
    let s = re.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let full = caps.get(0).unwrap().as_str();
        let open = full.chars().next().unwrap();
        let close = if open == '"' { '"' } else { ')' };
        format!("{}{}{}", open, title_case_phrase(inner), close)
    });
    title_case_phrase(&s)
}

// --- name parsing ---

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoliticianName {
    pub first: String,
    pub last: Option<String>,
    pub middle: Option<String>,
    pub preferred: Option<String>,
    pub suffix: Option<String>,
}

static NAME_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static PREFERRED_REGEX: OnceLock<Regex> = OnceLock::new();
static SUFFIX_REGEX: OnceLock<Regex> = OnceLock::new();
static LAST_NAME_REGEX: OnceLock<Regex> = OnceLock::new();
/// "De La" + one word at end (e.g. "De La Cruz")
static DE_LA_LAST_REGEX: OnceLock<Regex> = OnceLock::new();
/// "De" + one word at end (e.g. "De Soto"); try after DE_LA so "De La Cruz" is not split as "De" + "La"
static DE_LAST_REGEX: OnceLock<Regex> = OnceLock::new();
/// "Di" + one word at end (e.g. "Di Nicola")
static DI_LAST_REGEX: OnceLock<Regex> = OnceLock::new();
/// "Van De" + one word at end (e.g. "Van De Bogart")
static VAN_DE_LAST_REGEX: OnceLock<Regex> = OnceLock::new();
/// "San" + one word at end (e.g. "San Miguel")
static SAN_LAST_REGEX: OnceLock<Regex> = OnceLock::new();

/// Compound first names to keep together (longest first so "Mary Anne" matches before "Mary Ann").
const COMPOUND_FIRST_NAMES: &[&str] = &[
    "Anne Marie",
    "Ann Marie",
    "Leigh Ann",
    "Mary Anne",
    "Mary Ann",
    "Le Andra",
    "Lee Ann",
    "Jo Ann",
    "Anita Jo",
];

/// When both parentheses and quotes are used (e.g. ("JIM")), remove the inner quotes so we have (JIM).
/// Enables preferred-name extraction to treat it as parenthesized rather than quoted.
fn strip_quotes_inside_parens(s: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r#"\(\s*"([^"]*)"\s*\)"#).unwrap());
    re.replace_all(s, "($1)").to_string()
}

/// Extracts and removes preferred name (in double quotes, single quotes, or parentheses) from the string.
/// Returns (name_without_preferred, preferred_if_found). Collapses multiple spaces after removal.
fn extract_preferred(input: &str) -> (String, Option<String>) {
    let preferred_regex = PREFERRED_REGEX.get_or_init(|| {
        // Match "X", 'X', or (X), capture X (group 1, 2, or 3)
        Regex::new(r#""([^"]+)"|'([^']+)'|\(([^)]+)\)"#).unwrap()
    });
    if let Some(captures) = preferred_regex.captures(input) {
        let preferred = captures
            .get(1)
            .or_else(|| captures.get(2))
            .or_else(|| captures.get(3))
            .map(|m| m.as_str().trim().to_string());
        if let (Some(preferred), Some(full_match)) = (preferred, captures.get(0)) {
            let before = input[..full_match.start()].trim_end();
            let after = input[full_match.end()..].trim_start();
            let name_without_preferred = [before, after].join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
            return (name_without_preferred, Some(preferred));
        }
    }
    (input.to_string(), None)
}

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
/// Handles compound last names: "De La X", "De X", "Di X", "Van De X", "San X", "Van X", "St."/"St X" (case-insensitive)
/// Returns (name_without_last, last_name_if_found)
fn extract_last_name(input: &str) -> (String, Option<String>) {
    // 1. "De La" + one word (e.g. "Maria De La Cruz" -> last "De La Cruz")
    let de_la_regex = DE_LA_LAST_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+de\s+la\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = de_la_regex.captures(input) {
        if let Some(last_part) = captures.get(1) {
            let last_name = format!("De La {}", last_part.as_str());
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 2. "Van De" + one word (e.g. "John Van De Bogart" -> last "Van De Bogart"); before De so "Van De" isn't split as "De"
    let van_de_regex = VAN_DE_LAST_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+van\s+de\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = van_de_regex.captures(input) {
        if let Some(last_part) = captures.get(1) {
            let last_name = format!("Van De {}", last_part.as_str());
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 3. "De" + one word (e.g. "Hernando De Soto" -> last "De Soto"); after De La and Van De so they aren't split
    let de_regex = DE_LAST_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+de\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = de_regex.captures(input) {
        if let Some(last_part) = captures.get(1) {
            let last_name = format!("De {}", last_part.as_str());
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 4. "Di" + one word (e.g. "Yane Di Nicola" -> last "Di Nicola")
    let di_regex = DI_LAST_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+di\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = di_regex.captures(input) {
        if let Some(last_part) = captures.get(1) {
            let last_name = format!("Di {}", last_part.as_str());
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 5. "San" + one word (e.g. "Maria San Miguel" -> last "San Miguel")
    let san_regex = SAN_LAST_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+san\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = san_regex.captures(input) {
        if let Some(last_part) = captures.get(1) {
            let last_name = format!("San {}", last_part.as_str());
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 6. Van or St./St compound last names
    let compound_regex = LAST_NAME_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)\s+(van|st\.?)\s+([\w\.'-]+)$"#).unwrap()
    });
    if let Some(captures) = compound_regex.captures(input) {
        if let (Some(prefix), Some(last_part)) = (captures.get(1), captures.get(2)) {
            let prefix_str = prefix.as_str();
            let last_part_str = last_part.as_str();
            let prefix_formatted = if prefix_str.eq_ignore_ascii_case("van") {
                prefix_str.to_string()
            } else if prefix_str.ends_with('.') {
                prefix_str.to_string()
            } else {
                format!("{}.", prefix_str)
            };
            let last_name = format!("{} {}", prefix_formatted, last_part_str);
            let match_start = captures.get(0).unwrap().start();
            let name_without_last = input[..match_start].trim().to_string();
            return (name_without_last, Some(last_name));
        }
    }

    // 7. Single last word (excluding middle initials like "H.")
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

/// Normalize whitespace: collapse runs of whitespace to single space, trim, and remove spaces around hyphens.
fn normalize_whitespace(s: &str) -> String {
    let collapsed: String = s.trim().split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .split('-')
        .map(|part| part.trim())
        .collect::<Vec<_>>()
        .join("-")
}

/// Strip accents/diacritics (e.g. é → e, ñ → n) via NFD and removing combining marks.
fn strip_accents(s: &str) -> String {
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect()
}

/// Replace '' and directional/curly double quotes with ASCII ".
fn normalize_quotes(s: &str) -> String {
    s.replace("''", "\"")
        .replace('\u{201C}', "\"")  // "
        .replace('\u{201D}', "\"")  // "
        .replace('\u{201E}', "\"")  // „
        .replace('\u{201F}', "\"")   // ‟
}

/// Strip "dr." (case-insensitive) from the string only when not inside single quotes, double quotes, or parentheses.
fn strip_dr_unless_quoted_or_parens(s: &str) -> String {
    let s = s.as_bytes();
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    let mut in_double = false;
    let mut in_single = false;
    let mut paren_depth: u32 = 0;

    while i < s.len() {
        let outside = !in_double && !in_single && paren_depth == 0;

        if outside && i + 3 <= s.len() && s[i].eq_ignore_ascii_case(&b'd') && s[i + 1].eq_ignore_ascii_case(&b'r') && s[i + 2] == b'.' {
            i += 3;
            continue;
        }

        let c = s[i];
        match c {
            b'"' => in_double = !in_double,
            b'\'' => in_single = !in_single,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        out.push(c);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_default()
}

/// Normalize a name string: strip accents, quotes ('' and directional → "), strip "dr." when not in quotes/parens, collapse whitespace, trim around hyphens.
/// Use for candidate names and before name parsing.
pub(crate) fn normalize_name(s: &str) -> String {
    let s = strip_accents(s);
    let s = normalize_quotes(&s);
    let s = strip_dr_unless_quoted_or_parens(&s);
    normalize_whitespace(&s)
}

pub fn extract_politician_name(input: &str) -> Option<PoliticianName> {
    let input = normalize_name(input);
    let input = strip_quotes_inside_parens(&input);

    // 1. First, extract and remove preferred name (in quotes or parentheses)
    let (name_without_preferred, preferred) = extract_preferred(&input);

    // 2. Then, extract and remove suffix
    let (name_without_suffix, suffix) = extract_suffix(&name_without_preferred);

    // 3. Then, extract and remove last name
    let (name_without_last, last_name) = extract_last_name(&name_without_suffix);

    // 4. Parse remainder as first + optional middle (preferred already stripped)
    let remainder = name_without_last.trim();
    let words: Vec<&str> = remainder.split_whitespace().collect();

    for compound in COMPOUND_FIRST_NAMES {
        let compound_word_count = compound.split_whitespace().count();
        if words.len() >= compound_word_count {
            let prefix: String = words[..compound_word_count].join(" ");
            if prefix.eq_ignore_ascii_case(compound) {
                let first = title_case(&prefix);
                let middle = if compound_word_count < words.len() {
                    let rest: String = words[compound_word_count..].join(" ");
                    let rest = rest.trim();
                    if rest.is_empty() {
                        None
                    } else {
                        let m = title_case(rest);
                        Some(if m.len() == 1 && !m.ends_with('.') {
                            format!("{}.", m)
                        } else {
                            m
                        })
                    }
                } else {
                    None
                };
                return Some(PoliticianName {
                    first,
                    last: last_name,
                    middle,
                    preferred,
                    suffix,
                });
            }
        }
    }

    let extractors = NAME_EXTRACTORS.get_or_init(|| {
        // First name (required) + optional middle part (one or more words). Allow hyphen for names like "KATHI-ANN".
        vec![Regex::new(r#"^(?<first>[\w\.'-]+)(?<middle_part>(?: +[\w\.'-]+)*)$"#).unwrap()]
    });

    for extractor in extractors {
        if let Some(captures) = extractor.captures(remainder) {
            if let Some(first) = captures.name("first").map(owned_capture) {
                let middle = captures
                    .name("middle_part")
                    .map(owned_capture)
                    .map(|m| m.trim().to_string())
                    .filter(|m| !m.is_empty())
                    .map(|m| {
                        // If middle is a single letter without period, add period
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
                    preferred,
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
            (
                "Maria De La Cruz",
                Some(PoliticianName {
                    first: "Maria".into(),
                    last: Some("De La Cruz".into()),
                    ..Default::default()
                }),
            ),
            (
                "Hernando De Soto",
                Some(PoliticianName {
                    first: "Hernando".into(),
                    last: Some("De Soto".into()),
                    ..Default::default()
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
