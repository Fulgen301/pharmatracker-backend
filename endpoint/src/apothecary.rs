use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Response},
    Json,
};
use dto::{
    apothecary::ApothecaryDetail,
    error::RestError,
    medication::{MedicationSearch, MedicationSearchResultList},
    page::Page,
};
use entity::apothecary::ApothecaryWithSchedules;
use service::apothecary::ApothecaryServiceError;

use crate::appstate::AppState;

fn handle_apothecary_service_error(error: ApothecaryServiceError) -> Response {
    let (status_code, message) = match error {
        ApothecaryServiceError::NotFound => (StatusCode::NOT_FOUND, error.to_string()),
        ApothecaryServiceError::InvalidSortColumn(e) => (StatusCode::BAD_REQUEST, e),
        ApothecaryServiceError::Anyhow(e) => {
            tracing::error!("Error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        }
    };

    (status_code, Json(RestError { message })).into_response()
}

pub async fn get(
    State(ref state): State<AppState>,
) -> Result<Json<Page<ApothecaryDetail>>, ErrorResponse> {
    Ok(Json(
        state
            .apothecary_service
            .get(None)
            .await
            .map(|apothecary| {
                apothecary.map(|p| ApothecaryDetail::from(ApothecaryWithSchedules::from(p)))
            })
            .map_err(handle_apothecary_service_error)?
            .into(),
    ))
}

pub async fn get_medications(
    State(ref state): State<AppState>,
    Query(search_dto): Query<MedicationSearch>,
) -> Result<Json<Vec<MedicationSearchResultList>>, ErrorResponse> {
    let result = state
        .apothecary_service
        .get_medications(search_dto)
        .await
        .map_err(handle_apothecary_service_error)?;

    Ok(Json(result))
}
