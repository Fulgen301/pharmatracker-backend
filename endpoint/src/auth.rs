use std::fmt::Display;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use dto::error::RestError;
use service::jwt::Role;
use uuid::Uuid;

use crate::appstate::AppState;

pub enum AuthError {
    InvalidToken,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(RestError {
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(Clone)]
pub struct Auth {
    pub user_id: Uuid,
    pub roles: Vec<Role>,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Auth {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = state
            .jwt_service
            .claims(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(Self {
            user_id: Uuid::parse_str(&token_data.sub).map_err(|_| AuthError::InvalidToken)?,
            roles: token_data.roles,
        })
    }
}
