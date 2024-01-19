use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApothecaryDetail {
    pub id: Uuid,
    pub name: String,
    pub longitude: f32,
    pub latitude: f32,
    pub street: String,
    pub number: String,
    pub post_code: i32,
    pub city: String,
    pub country: String,
}
