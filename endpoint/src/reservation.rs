use axum::{
    extract::State,
    http::StatusCode,
    response::{ErrorResponse, IntoResponse},
    Json,
};
use dto::{
    error::RestError,
    page::Page,
    reservation::{MedicationReservation, MedicationReservationRequest},
};
use service::reservation::ReservationServiceError;

use crate::{appstate::AppState, auth::Auth};

fn handle_reservation_service_error(e: ReservationServiceError) -> impl IntoResponse {
    let (status, message) = match e {
        ReservationServiceError::UserNotFound | ReservationServiceError::MedicationNotFound => {
            (StatusCode::NOT_FOUND, e.to_string())
        }
        ReservationServiceError::NotEnoughAvailable => (StatusCode::NOT_FOUND, e.to_string()),
        ReservationServiceError::Anyhow(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    (status, Json(RestError { message })).into_response()
}

pub async fn get(
    State(ref state): State<AppState>,
    auth: Auth,
) -> Result<Json<Page<MedicationReservation>>, ErrorResponse> {
    Ok(Json(
        state
            .reservation_service
            .get(auth.user_id)
            .await
            .map_err(handle_reservation_service_error)?
            .map(MedicationReservation::from)
            .into(),
    ))
}

pub async fn post(
    State(ref state): State<AppState>,
    auth: Auth,
    Json(request): Json<MedicationReservationRequest>,
) -> Result<Json<MedicationReservation>, ErrorResponse> {
    Ok(Json(
        state
            .reservation_service
            .reserve(auth.user_id, request)
            .await
            .map_err(handle_reservation_service_error)?
            .into(),
    ))
}
