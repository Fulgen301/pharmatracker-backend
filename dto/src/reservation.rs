use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::{
    apothecary::ApothecaryDetail,
    medication::{MedicationDetail, MedicationQuantity},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationReservationRequest {
    pub medication_id: Uuid,
    pub quantity: MedicationQuantity,
    pub start_date_time: PrimitiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct MedicationReservation {
    pub id: Uuid,
    pub apothecary: ApothecaryDetail,
    pub medication: MedicationDetail,
    pub quantity: MedicationQuantity,
    pub start_date_time: PrimitiveDateTime,
    pub end_date_time: PrimitiveDateTime,
}
