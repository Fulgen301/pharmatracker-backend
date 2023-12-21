use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Response},
    Json,
};
use service::user::UserServiceError;

use crate::appstate::AppState;
use dto::{
    error::RestError,
    user::{UserLogin, UserRegistration},
};

fn handle_user_service_error(error: UserServiceError) -> Response {
    let (status_code, message) = match error {
        UserServiceError::InvalidCredentials => {
            (axum::http::StatusCode::UNAUTHORIZED, error.to_string())
        }
        UserServiceError::UserNotFound => (axum::http::StatusCode::NOT_FOUND, error.to_string()),
        UserServiceError::UserAlreadyExists => {
            (axum::http::StatusCode::CONFLICT, error.to_string())
        }
        UserServiceError::Anyhow(e) => {
            tracing::error!("Error: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, String::new())
        }
    };

    (status_code, Json(RestError { message })).into_response()
}

pub async fn login(
    State(ref state): State<AppState>,
    Json(user_login): Json<UserLogin>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = state
        .user_service
        .login(user_login)
        .await
        .map_err(handle_user_service_error)?;

    Ok((
        StatusCode::OK,
        format!(
            "Bearer {}",
            state.jwt_service.create_token(user).map_err(|e| {
                tracing::error!("Error: {}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Body::empty())
            })?
        ),
    ))
}

pub async fn register(
    State(ref state): State<AppState>,
    Json(user_register): Json<UserRegistration>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = state
        .user_service
        .register(user_register)
        .await
        .map_err(handle_user_service_error)?;

    Ok((
        StatusCode::OK,
        format!(
            "Bearer {}",
            state.jwt_service.create_token(user).map_err(|e| {
                tracing::error!("Error: {}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Body::empty())
            })?
        ),
    ))
}
