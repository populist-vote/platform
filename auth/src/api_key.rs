use crate::AccessTokenClaims;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use db::{ApiKey, User};
use jsonwebtoken::{Header, TokenData};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub const API_KEY_PREFIX: &str = "pop_";
const SECRET_BYTES: usize = 32;
const DISPLAY_SECRET_CHARS: usize = 8;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AuthenticationKind {
    Jwt,
    ApiKey,
}

#[derive(Debug, Clone)]
pub struct RequestAuthentication {
    pub claims: AccessTokenClaims,
    pub kind: AuthenticationKind,
}

#[derive(Debug)]
pub struct GeneratedApiKey {
    pub plaintext: String,
    pub display_prefix: String,
    pub hash: [u8; 32],
}

pub fn generate_api_key() -> GeneratedApiKey {
    let mut secret = [0_u8; SECRET_BYTES];
    OsRng.fill_bytes(&mut secret);
    let encoded = URL_SAFE_NO_PAD.encode(secret);
    let plaintext = format!("{API_KEY_PREFIX}{encoded}");
    let display_prefix = format!("{API_KEY_PREFIX}{}", &encoded[..DISPLAY_SECRET_CHARS]);
    let hash = hash_api_key(&plaintext);

    GeneratedApiKey {
        plaintext,
        display_prefix,
        hash,
    }
}

pub fn hash_api_key(api_key: &str) -> [u8; 32] {
    Sha256::digest(api_key.as_bytes()).into()
}

pub async fn authenticate_api_key(
    pool: &PgPool,
    api_key: &str,
) -> Result<Option<TokenData<AccessTokenClaims>>, db::Error> {
    if !api_key.starts_with(API_KEY_PREFIX) {
        return Ok(None);
    }

    let Some(key) = ApiKey::find_active_by_hash(pool, &hash_api_key(api_key)).await? else {
        return Ok(None);
    };
    let user = User::find_by_id(pool, key.user_id).await?;
    if user.confirmed_at.is_none() {
        return Ok(None);
    }
    let organizations = User::organization_roles(pool, user.id).await?;
    let claims = AccessTokenClaims {
        sub: user.id,
        username: user.username,
        email: user.email,
        system_role: user.system_role,
        organizations,
        // API keys do not expire here. The claim only lives for this request.
        exp: usize::MAX,
    };

    ApiKey::mark_used(pool, key.id).await?;

    Ok(Some(TokenData {
        header: Header::default(),
        claims,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_are_unique_and_safe_to_display() {
        let first = generate_api_key();
        let second = generate_api_key();

        assert!(first.plaintext.starts_with(API_KEY_PREFIX));
        assert_eq!(first.plaintext.len(), API_KEY_PREFIX.len() + 43);
        assert_eq!(
            first.display_prefix.len(),
            API_KEY_PREFIX.len() + DISPLAY_SECRET_CHARS
        );
        assert!(first.plaintext.starts_with(&first.display_prefix));
        assert_eq!(first.hash, hash_api_key(&first.plaintext));
        assert_ne!(first.plaintext, second.plaintext);
        assert_ne!(first.hash, second.hash);
    }

    #[test]
    fn hashing_is_deterministic_without_revealing_the_key() {
        let key = "pop_example";
        let hash = hash_api_key(key);

        assert_eq!(hash, hash_api_key(key));
        assert_ne!(hash.as_slice(), key.as_bytes());
    }
}
