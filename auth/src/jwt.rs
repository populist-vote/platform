use crate::Error;
use db::{Role, User};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: uuid::Uuid,
    username: String,
    email: String,
    role: Role,
    exp: usize,
}

pub fn create_token_for_user(user_record: User) -> Result<String, Error> {
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
        exp: expiration as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => panic!("Something went wrong encoding a JWT: {}", e), //TODO properly handle this
    };

    Ok(token)
}

pub fn validate_token(token: &str) -> Result<TokenData<Claims>, Error> {
    let key = std::env::var("JWT_SECRET")?;
    
    let token_data = match decode::<Claims>(
        &token.to_string(),
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::default(),
    ) {
        Ok(c) => c,
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => panic!("Token is invalid"),
            ErrorKind::InvalidIssuer => panic!("Issuer is invalid"),
            _ => panic!("Something went wrong decoding a JWT"),
        },
    };

    Ok(token_data)
}
