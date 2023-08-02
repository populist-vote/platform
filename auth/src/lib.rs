pub mod errors;
pub mod jwt;
pub use errors::Error;
pub use jwt::*;
pub use passwords::PasswordGenerator;
use rand::Rng;

pub fn create_temporary_password() -> String {
    PasswordGenerator::new()
        .length(8)
        .numbers(true)
        .lowercase_letters(true)
        .uppercase_letters(true)
        .symbols(true)
        .spaces(true)
        .exclude_similar_characters(true)
        .strict(true)
        .generate_one()
        .unwrap()
}

/// Create a username with the email root and a random number
pub fn create_temporary_username(email: String) -> String {
    let mut rng = rand::thread_rng();
    let base = email.split('@').collect::<Vec<&str>>()[0].to_string();
    let rnd_int: i32 = rng.gen();
    let mut raw = format!("{}{}", base, rnd_int);
    // Strip all undesirable characters per the postgres constraint
    raw.retain(|c| !r#"+(),"-;:'-"#.contains(c));
    truncate(&raw, 20).to_string()
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

pub enum TokenType {
    Access,
    Refresh,
}

pub fn format_auth_cookie(token_type: TokenType, token: &str) -> String {
    let token_type_str = match token_type {
        TokenType::Access => "access_token",
        TokenType::Refresh => "refresh_token",
    };

    let expiry_duration = match token_type {
        TokenType::Access => chrono::Duration::minutes(5),
        TokenType::Refresh => chrono::Duration::days(120),
    };

    format!(
        "{}={}; HttpOnly; SameSite=None; Secure; Domain={}; Expires={};",
        token_type_str,
        token,
        config::Config::default().root_domain,
        (chrono::Utc::now() + expiry_duration).format("%a, %d %b %Y %T GMT")
    )
}

#[test]
fn test_create_temporary_username() {
    let input = "lai.henry+69@gmail.com";
    let result = create_temporary_username(input.to_string());
    assert!(regex::Regex::new(r"^{3,20}[a-zA-Z0-9._]")
        .unwrap()
        .is_match(&result));
}

#[test]
fn test_format_auth_cookie() {
    let token = "test";
    let result = format_auth_cookie(TokenType::Access, token);
    assert_eq!(
        result,
        format!(
            "access_token=test; HttpOnly; SameSite=None; Secure; Domain=localhost; Expires={};",
            (chrono::Utc::now() + chrono::Duration::days(30)).format("%a, %d %b %Y %T GMT")
        )
    );
}
