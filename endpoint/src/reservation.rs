use axum::{
    extract::State,
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Response},
    Json,
};
use dto::{
    error::RestError,
    reservation::{MedicationReservation, MedicationReservationRequest},
};

use crate::{appstate::AppState, auth::Auth};

pub async fn post(
    State(ref state): State<AppState>,
    auth: Auth,
    Json(request): Json<MedicationReservationRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let response = (
        StatusCode::FORBIDDEN,
        Json(RestError {
            message: "Unavailable".to_string(),
        }),
    )
        .into_response();

    if false {
        Ok("hi")
    } else {
        Err(response.into())
    }
}
