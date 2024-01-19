use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub content: Vec<T>,
    pub last: bool,
    pub total_elements: u64,
    pub total_pages: u64,
    pub size: u64,
    pub number: u64,
    pub first: bool,
    pub number_of_elements: u64,
    pub empty: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pageable {
    #[serde(flatten)]
    pub options: PageableOptions,

    pub sort: Option<Sort>,

    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PageableOptions {
    OffsetAndLimit((u64, u64)),
    Page(PageQuery),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    #[serde(default = "default_per_page")]
    pub per_page: u64,
    pub index: u64,
}

const fn default_per_page() -> u64 {
    20
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sort {
    #[serde(flatten)]
    pub criteria: Vec<SortCriterion>,
}

#[derive(Clone, Debug)]
pub struct SortCriterion {
    pub field: String,
    pub direction: SortDirection,
}

impl<'de> Deserialize<'de> for SortCriterion {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        let mut split = s.split(',');

        let field = split.next().unwrap_or_default().to_string();
        let direction = split
            .next()
            .unwrap_or_default()
            .parse::<SortDirection>()
            .map_err(serde::de::Error::custom)?;

        Ok(Self { field, direction })
    }
}

impl Serialize for SortCriterion {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{},{}", &self.field, &self.direction))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "asc"),
            SortDirection::Desc => write!(f, "desc"),
        }
    }
}

impl FromStr for SortDirection {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err("Invalid sort direction"),
        }
    }
}
