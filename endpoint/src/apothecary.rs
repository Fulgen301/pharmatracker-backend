use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Response},
    Json,
};
use dto::{
    apothecary::ApothecaryDetail,
    error::RestError,
    medication::{
        MedicationDetailWithQuantity, MedicationSearch, MedicationSearchCda,
        MedicationSearchResultList,
    },
    page::Page,
};
use entity::apothecary::ApothecaryWithSchedules;
use service::apothecary::ApothecaryServiceError;

use crate::{appstate::AppState, auth::Auth};

fn handle_apothecary_service_error(error: ApothecaryServiceError) -> Response {
    let (status_code, message) = match error {
        ApothecaryServiceError::NotFound => (StatusCode::NOT_FOUND, error.to_string()),
        ApothecaryServiceError::InvalidSortColumn(e) => (StatusCode::BAD_REQUEST, e),
        ApothecaryServiceError::InvalidXml => (StatusCode::BAD_REQUEST, error.to_string()),
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
    let result = state
        .apothecary_service
        .get(None)
        .await
        .map_err(handle_apothecary_service_error)?
        .map(|p| ApothecaryDetail::from(ApothecaryWithSchedules::from(p)))
        .into();

    Ok(Json(result))
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

pub async fn get_own_medications(
    State(ref state): State<AppState>,
    auth: Auth,
) -> Result<Json<Vec<MedicationDetailWithQuantity>>, ErrorResponse> {
    let result = state
        .apothecary_service
        .get_own_medications(auth.user_id)
        .await
        .map_err(handle_apothecary_service_error)?;

    Ok(Json(result))
}

pub async fn get_medications_by_cda(
    State(ref state): State<AppState>,
    Query(search_dto): Query<MedicationSearchCda>,
    cda: String,
) -> Result<Json<Vec<MedicationSearchResultList>>, ErrorResponse> {
    let result = state
        .apothecary_service
        .get_medications_by_cda(cda, search_dto)
        .await
        .map_err(handle_apothecary_service_error)?;

    Ok(Json(result))
}
