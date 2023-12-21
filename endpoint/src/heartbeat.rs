use axum::{extract::State, response::IntoResponse, Json};
use dto::heartbeat::Heartbeat;

use crate::appstate::AppState;

pub async fn get(_state: State<AppState>) -> impl IntoResponse {
    Json(Heartbeat {
        status: "ok".to_string(),
    })
}
