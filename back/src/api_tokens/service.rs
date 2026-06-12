use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api_tokens::model::{CreateTokenResponse, TokenInfoResponse};
use crate::api_tokens::repository;
use crate::common::error::AppError;

const TOKEN_PREFIX: &str = "lc_";

fn generate_raw_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let encoded = URL_SAFE_NO_PAD.encode(&bytes);
    let full = format!("{}{}", TOKEN_PREFIX, encoded);
    let prefix = encoded[..3].to_string();
    (full, prefix)
}

fn hash_token(token: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(token.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_token(token: &str, stored_hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(stored_hash)?;
    Ok(Argon2::default()
        .verify_password(token.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn hash_for_lookup(token: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(token.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub async fn create_token(db: &PgPool, user_id: Uuid) -> Result<CreateTokenResponse, AppError> {
    repository::delete_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("failed to clear existing token: {e}")))?;

    let (raw_token, prefix) = generate_raw_token();
    let hash = hash_token(&raw_token).map_err(|e| AppError::internal(format!("hashing failed: {e}")))?;

    let id = Uuid::new_v4();
    let token = repository::insert_token(db, id, user_id, &prefix, &hash)
        .await
        .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok(CreateTokenResponse {
        id: token.id,
        token_prefix: format!("{}{}", TOKEN_PREFIX, token.token_prefix),
        token_full: raw_token,
        created_at: token.created_at,
    })
}

pub async fn get_token_info(db: &PgPool, user_id: Uuid) -> Result<Option<TokenInfoResponse>, AppError> {
    let token = repository::find_active_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("lookup failed: {e}")))?;

    Ok(token.map(|t| TokenInfoResponse {
        id: t.id,
        token_prefix: format!("{}{}", TOKEN_PREFIX, t.token_prefix),
        created_at: t.created_at,
        last_used_at: t.last_used_at,
    }))
}

pub async fn revoke_token(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    repository::delete_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("delete failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_format_and_prefix() {
        let (full, prefix) = generate_raw_token();
        assert!(full.starts_with("lc_"), "token must start with lc_");
        assert_eq!(full.len(), 46, "lc_ + 43 base64 chars = 46");
        assert_eq!(prefix.len(), 3, "prefix must be 3 chars");
        assert_eq!(&full[3..6], prefix, "prefix must match chars after lc_");
    }

    #[test]
    fn hash_verify_roundtrip() {
        let token = "lc_test_token_value_12345";
        let hash = hash_token(token).expect("hash should succeed");
        assert!(verify_token(token, &hash).expect("verify should succeed"));
        assert!(!verify_token("lc_wrong_token", &hash).expect("verify should succeed"));
    }

    #[test]
    fn verify_fails_for_malformed_hash() {
        assert!(verify_token("lc_some_token", "not_a_valid_hash").is_err());
    }

    #[test]
    fn hash_for_lookup_returns_valid_hash() {
        let hash = hash_for_lookup("lc_my_token").expect("hash_for_lookup should succeed");
        assert!(hash.starts_with("$argon2"));
    }
}
