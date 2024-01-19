use std::{fmt::Display, sync::Arc};

use entity::DatabaseConnection;
use jsonwebtoken::{Algorithm, Validation};
use serde::{Deserialize, Serialize};
use settings::Settings;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: Uuid,
    pub user_id: Uuid,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub roles: Vec<Role>,
}

pub enum TokenError {
    Jwt(jsonwebtoken::errors::Error),
    Uuid(uuid::Error),
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::Jwt(e) => write!(f, "Failed to parse JWT: {}", e),
            TokenError::Uuid(e) => write!(f, "Failed to parse UUID: {}", e),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Customer,
    Apothecary,
    Admin,
}

pub struct JwtService {
    #[allow(unused)]
    db: DatabaseConnection,
    settings: Arc<Settings>,
}

impl JwtService {
    pub fn new(db: DatabaseConnection, settings: Arc<Settings>) -> Self {
        Self { db, settings }
    }

    pub fn claims(&self, token: &str) -> Result<TokenClaims, TokenError> {
        let validation = Validation::new(Algorithm::RS256);

        Ok(jsonwebtoken::decode::<TokenClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_rsa_pem(
                self.settings.jwt.access_token_public_key.as_bytes(),
            )
            .unwrap(),
            &validation,
        )
        .map_err(|e| TokenError::Jwt(e))?
        .claims)
    }

    pub fn create_token(&self, user: crate::user::User) -> Result<String, TokenError> {
        let claims = TokenClaims {
            sub: user.id.to_string(),
            exp: 2000000000,
            roles: match user.user_type {
                entity::user::UserType::Admin => {
                    vec![Role::Customer, Role::Apothecary, Role::Admin]
                }
                entity::user::UserType::Customer => vec![Role::Customer],
                entity::user::UserType::Apothecary => vec![Role::Apothecary],
            },
        };

        Ok(jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(
                self.settings.jwt.access_token_private_key.as_bytes(),
            )
            .unwrap(),
        )
        .map_err(|e| TokenError::Jwt(e))?)
    }
}
