use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::apothecary::ApothecaryDetail;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationDetail {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MedicationQuantity {
    Liquid(MedicationQuantityLiquid),
    Package(MedicationQuantityPackage),
    Unknown(MedicationQuantityUnknown),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationQuantityLiquid {
    pub r#type: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationQuantityPackage {
    pub r#type: String,
    pub quantity: u64,
    pub price: Decimal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationQuantityUnknown {
    pub r#type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationSearch {
    pub name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub max_distance: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationSearchResultList {
    pub medication: MedicationDetail,
    pub results: Vec<MedicationSearchResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationSearchResult {
    pub quantity: MedicationQuantity,
    pub aliases: Vec<MedicationDetail>,
    pub apothecary: ApothecaryDetail,
}
