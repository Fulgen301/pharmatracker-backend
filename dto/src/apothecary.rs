use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schedule::Schedule;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApothecaryDetail {
    pub id: Uuid,
    pub name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub street: String,
    pub number: String,
    pub post_code: i32,
    pub city: String,
    pub country: String,
    pub schedules: Vec<Schedule>,
}
