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

pub fn extract_politician_name(input: &str) -> Option<PoliticianName> {
    let extractors = NAME_EXTRACTORS.get_or_init(|| {
        // Regular expressions are broken into multiple lines for readability
        [
            // Reference: https://regex101.com/r/xKvi7n/3
            // Adapted from https://regex101.com/library/7zjSTN
            // FIXME - It's so close!!!
            [
                r#"^"#,
                r#"(?<first>[\w\.']+)"#,
                r#"(?: *(?<middle1>[\w\.']+)*?)"#,
                r#"(?: *"(?<preferred>[\w\.' ]+)*")?"#,
                r#"(?: *(?<middle2>[\w\.']+)*?)"#,
                r#"(?: *(?<last>[^\s,iIvVxX(?:[jJsS][rR]\.?)]+))"#,
                r#"(?:,? +(?<suffix>[iIvVxX((?:[jJsS][rR]\.?)]+))?"#,
                r#"$"#,
            ],
        ]
        .into_iter()
        .map(|r| Regex::new(&r.join("")).unwrap())
        .collect()
    });

    for extractor in extractors {
        println!("{}", extractor.as_str());
        if let Some(captures) = extractor.captures(input) {
            if let Some(first) = captures.name("first").map(owned_capture) {
                return Some(PoliticianName {
                    first,
                    last: captures.name("last").map(owned_capture),
                    middle: captures
                        .name("middle2")
                        .or(captures.name("middle1"))
                        .map(owned_capture),
                    preferred: captures.name("preferred").map(owned_capture),
                    suffix: captures.name("suffix").map(owned_capture),
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
