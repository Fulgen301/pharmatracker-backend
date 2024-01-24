use std::{collections::HashMap, fmt::Display};

use anyhow::anyhow;
use dto::{
    apothecary,
    medication::{
        self, MedicationDetail, MedicationDetailWithQuantity, MedicationSearch,
        MedicationSearchCda, MedicationSearchResult, MedicationSearchResultList,
    },
    page::Pageable,
};
use quick_xml::NsReader;
use sea_orm::{
    sea_query::{
        self, extension::postgres::PgExpr, Expr, Func, IntoCondition, LikeExpr, SimpleExpr,
    },
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, Iden, JoinType, ModelTrait,
    QueryFilter, QuerySelect, RelationTrait, RuntimeErr, Values,
};

pub use entity::apothecary::Model as Apothecary;
pub use entity::schedule::Model as Schedule;
use entity::{
    apothecary::{ApothecaryWithSchedules, Entity},
    apothecary_medication,
};
use tracing::{debug, field::debug, warn};
use uuid::Uuid;

use crate::page::{Page, PageError};

pub enum ApothecaryServiceError {
    NotFound,
    InvalidSortColumn(String),
    InvalidXml,
    Anyhow(anyhow::Error),
}

impl Display for ApothecaryServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApothecaryServiceError::NotFound => write!(f, "Apothecary not found"),
            ApothecaryServiceError::InvalidSortColumn(e) => write!(f, "Invalid sort column: {}", e),
            ApothecaryServiceError::InvalidXml => write!(f, "Invalid XML"),
            ApothecaryServiceError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

impl From<DbErr> for ApothecaryServiceError {
    fn from(err: DbErr) -> Self {
        Self::Anyhow(err.into())
    }
}

impl From<PageError> for ApothecaryServiceError {
    fn from(err: PageError) -> Self {
        match err {
            PageError::InvalidColumnName(e) => Self::InvalidSortColumn(e),
            PageError::InvalidDirectionName => Self::Anyhow(anyhow::anyhow!("Invalid direction")),
            PageError::DbErr(e) => Self::Anyhow(anyhow::Error::from(e)),
        }
    }
}

impl From<quick_xml::Error> for ApothecaryServiceError {
    fn from(_: quick_xml::Error) -> Self {
        Self::InvalidXml
    }
}

pub struct ApothecaryService {
    db: DatabaseConnection,
}

#[derive(Iden)]
#[iden = "UPPER"]
struct Upper;

impl ApothecaryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get(
        &self,
        pageable: Option<Pageable>,
    ) -> Result<Page<(Apothecary, Vec<Schedule>)>, ApothecaryServiceError> {
        Page::<(Apothecary, Vec<Schedule>)>::paginate_two_many(
            &self.db,
            Entity::find().find_with_related(entity::schedule::Entity),
            pageable,
        )
        .await
        .map_err(|e| e.into())
    }

    pub async fn get_medications(
        &self,
        search_dto: MedicationSearch,
    ) -> Result<Vec<MedicationSearchResultList>, ApothecaryServiceError> {
        let apothecary_medications = entity::apothecary_medication::Entity::find()
            .all(&self.db)
            .await?;

        let mut result: Vec<MedicationSearchResultList> = vec![];

        let mut hash = HashMap::<Uuid, Vec<apothecary_medication::Model>>::new();

        for apothecary_medication in apothecary_medications {
            debug!(
                "Apothecary medication: {:?}",
                apothecary_medication.medication_id
            );
            let medication_id = apothecary_medication.medication_id.clone();
            let mut list = hash
                .get(&apothecary_medication.medication_id)
                .unwrap_or(&vec![])
                .clone();

            list.push(apothecary_medication);

            hash.insert(medication_id, list);
        }

        for (medication_id, apothecary_medications) in hash.into_iter() {
            let medication = entity::medication::Entity::find_by_id(medication_id)
                .one(&self.db)
                .await?
                .ok_or(ApothecaryServiceError::NotFound)?;

            debug!("Medication: {:?}", medication.name);

            if !medication
                .name
                .to_uppercase()
                .contains(search_dto.name.to_uppercase().as_str())
            {
                continue;
            }

            let mut list = MedicationSearchResultList {
                medication: medication.into(),
                results: vec![],
            };

            for apothecary_medication in apothecary_medications {
                let apothecary_schedules =
                    entity::apothecary::Entity::find_by_id(apothecary_medication.apothecary_id)
                        .find_with_related(entity::schedule::Entity)
                        .all(&self.db)
                        .await?;

                if apothecary_schedules.len() != 1 {
                    return Err(ApothecaryServiceError::Anyhow(anyhow!(DbErr::Query(
                        RuntimeErr::Internal("Too many apothecaries for one ID".to_owned())
                    ))));
                }

                let apothecary = &apothecary_schedules.first().unwrap().0;

                if apothecary_distance(
                    (apothecary.latitude, apothecary.longitude),
                    (search_dto.latitude, search_dto.longitude),
                ) > search_dto.max_distance as f32
                {
                    continue;
                }

                let mut apothecary_schedules = apothecary_schedules;
                let (apothecary, schedules) = apothecary_schedules.pop().unwrap();

                let result = MedicationSearchResult {
                    quantity: apothecary_medication.into(),
                    aliases: vec![],
                    apothecary: ApothecaryWithSchedules::from((apothecary, schedules)).into(),
                };

                list.results.push(result);
            }

            if list.results.is_empty() {
                continue;
            }

            result.push(list);
        }

        Ok(result)
    }

    pub async fn get_medications_by_cda(
        &self,
        cda: String,
        search_dto: MedicationSearchCda,
    ) -> Result<Vec<MedicationSearchResultList>, ApothecaryServiceError> {
        warn!("content: {:?}", cda);
        let re = regex::Regex::new(r"<name>(.+)</name>").unwrap();

        match re.captures(&cda) {
            Some(captures) => {
                let (_, [name]) = captures.extract();
                warn!("Regex: {:?}", name);
                return self
                    .get_medications(MedicationSearch {
                        name: name.to_owned(),
                        latitude: search_dto.latitude,
                        longitude: search_dto.longitude,
                        max_distance: search_dto.max_distance,
                    })
                    .await;
            }
            None => return Ok(vec![]),
        }
    }

    pub async fn get_own_medications(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<MedicationDetailWithQuantity>, ApothecaryServiceError> {
        let apothecary = entity::user::Entity::find_by_id(user_id)
            .find_with_related(entity::apothecary::Entity)
            .all(&self.db)
            .await?
            .pop()
            .unwrap()
            .1
            .pop()
            .unwrap();

        debug("got apo");

        let medications = apothecary
            .find_related(entity::medication::Entity)
            .all(&self.db)
            .await?;

        let mut result = vec![];

        for medication in medications {
            let apothecary_medication = entity::apothecary_medication::Entity::find_by_id((
                apothecary.id.clone(),
                medication.id.clone(),
            ))
            .one(&self.db)
            .await?
            .unwrap();

            result.push(MedicationDetailWithQuantity {
                medication: medication.into(),
                quantity: apothecary_medication.into(),
            });
        }

        Ok(result)
    }
}

// https://github.com/geopy/geopy/blob/f495974c32a7a7b1eb433e7b8c87166e96375c32/geopy/distance.py#L463-L481
fn apothecary_distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    const EARTH_RADIUS: f32 = 6371.009;

    let (lat1, lng1) = (a.0.to_radians(), a.1.to_radians());
    let (lat2, lng2) = (b.0.to_radians(), b.1.to_radians());

    let (sin_lat1, cos_lat1) = (lat1.sin(), lat1.cos());
    let (sin_lat2, cos_lat2) = (lat2.sin(), lat2.cos());

    let delta_lng = lng2 - lng1;
    let (cos_delta_lng, sin_delta_lng) = (delta_lng.cos(), delta_lng.sin());

    let d = ((cos_lat2 * sin_delta_lng).powi(2)
        + (cos_lat1 * sin_lat2 - sin_lat1 * cos_lat2 * cos_delta_lng).powi(2))
    .sqrt()
    .atan2(sin_lat1 * sin_lat2 + cos_lat1 * cos_lat2 * cos_delta_lng);

    return EARTH_RADIUS * d;
}
