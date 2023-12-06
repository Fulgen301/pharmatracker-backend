use axum::{extract::State, response::IntoResponse, Json};

use crate::{appstate::AppState, dto::heartbeat::Heartbeat};

pub async fn get(_state: State<AppState>) -> impl IntoResponse {
    Json(Heartbeat {
        status: "ok".to_string(),
    })
}
