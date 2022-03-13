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

pub fn create_temporary_username(email: String) -> String {
    let mut rng = rand::thread_rng();

    let base = email.split('@').collect::<Vec<&str>>()[0].to_string();
    let rnd_int: i32 = rng.gen();
    format!("{}{}", base, rnd_int)
}
