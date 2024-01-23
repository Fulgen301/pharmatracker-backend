use serde::{Deserialize, Serialize};
use time::{serde::format_description, Time, Weekday};

format_description!(schedule_format, Time, "[hour]:[minute]");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub weekday: Weekday,
    #[serde(with = "schedule_format")]
    pub start: Time,
    #[serde(with = "schedule_format")]
    pub end: Time,
}
