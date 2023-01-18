use crate::Error;
use db::{Role, User};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub organization_id: Option<uuid::Uuid>,
    pub exp: usize,
}

pub fn create_power_token() -> Result<String, Error> {
    let key = std::env::var("JWT_SECRET")?;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(120))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uuid::Uuid::new_v4(),
        username: "superadmin".to_string(),
        email: "info@populist.us".to_string(),
        role: Role::SUPERUSER,
        organization_id: None,
        exp: expiration as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => panic!("Something went wrong encoding a JWT: {}", e),
    };

    Ok(token)
}

pub fn create_access_token_for_user(user_record: User) -> Result<String, Error> {
    let key = std::env::var("JWT_SECRET")?;

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_record.id,
        username: user_record.username,
        email: user_record.email,
        role: user_record.role,
        organization_id: user_record.organization_id,
        exp: expiration as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => panic!("Something went wrong encoding a JWT: {}", e),
    };

    Ok(token)
}

pub fn validate_token(token: &str) -> Result<TokenData<Claims>, Error> {
    let key = std::env::var("JWT_SECRET")?;

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => Ok(token_data),
        Err(err) => Err(Error::JwtError(err)),
    }
}

pub fn create_random_token() -> Result<String, Error> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    Ok(rand_string)
}
