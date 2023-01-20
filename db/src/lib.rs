pub mod errors;
pub mod loaders;
pub mod models;
pub mod pool;

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub use errors::Error;

pub use models::address::*;
pub use models::argument::*;
pub use models::ballot_measure::*;
pub use models::bill::*;
pub use models::election::*;
pub use models::embed::*;
pub use models::issue_tag::*;
pub use models::office::*;
pub use models::organization::*;
pub use models::organization_politician_note::*;
pub use models::politician::*;
pub use models::race::*;
pub use models::user::*;
pub use pool::*;

/// This function takes in a string and returns a ts_query safe string for postgres
/// For example "barack oba" becomes "barack%oba:*"
fn process_search_query(query: String) -> String {
    if query.is_empty() {
        "".to_string()
    } else {
        let mut formatted = format!("%{}", query)
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("%");
        formatted.push_str(":*");
        formatted
    }
}

#[test]
fn test_process_search_query() {
    assert_eq!(process_search_query("".to_string()), "");
    assert_eq!(process_search_query("foo".to_string()), "%foo:*");
    assert_eq!(process_search_query("foo bar".to_string()), "%foo%bar:*");
    assert_eq!(
        process_search_query("foo bar baz".to_string()),
        "%foo%bar%baz:*"
    );

    assert_eq!(
        process_search_query("jen mcewen".to_string()),
        "%jen%mcewen:*"
    )
}
