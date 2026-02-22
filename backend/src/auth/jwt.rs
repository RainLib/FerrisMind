use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::JwtConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user record id (e.g. "user:xxx")
    pub email: String,
    pub username: String,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at
}

pub fn create_token(
    config: &JwtConfig,
    user_id: &str,
    email: &str,
    username: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.expiration_hours as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        username: username.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

pub fn verify_token(
    config: &JwtConfig,
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JwtConfig;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-12345".to_string(),
            expiration_hours: 24,
        }
    }

    #[test]
    fn test_create_and_verify_token() {
        let config = test_config();
        let token = create_token(&config, "user:test123", "test@example.com", "testuser").unwrap();
        let claims = verify_token(&config, &token).unwrap();

        assert_eq!(claims.sub, "user:test123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let config = test_config();
        let result = verify_token(&config, "invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret() {
        let config = test_config();
        let token = create_token(&config, "user:test123", "test@example.com", "testuser").unwrap();

        let wrong_config = JwtConfig {
            secret: "wrong-secret".to_string(),
            expiration_hours: 24,
        };
        let result = verify_token(&wrong_config, &token);
        assert!(result.is_err());
    }
}
