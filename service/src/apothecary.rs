use std::{collections::HashMap, fmt::Display};

use anyhow::anyhow;
use dto::{
    medication::{MedicationSearch, MedicationSearchResult, MedicationSearchResultList},
    page::Pageable,
};
use sea_orm::{
    sea_query::{Expr, IntoCondition},
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, JoinType, ModelTrait,
    QueryFilter, QuerySelect, RelationTrait, RuntimeErr,
};

pub use entity::apothecary::Model as Apothecary;
pub use entity::schedule::Model as Schedule;
use entity::{
    apothecary::{ApothecaryWithSchedules, Entity},
    apothecary_medication,
};
use uuid::Uuid;

use crate::page::{Page, PageError};

pub enum ApothecaryServiceError {
    NotFound,
    InvalidSortColumn(String),
    Anyhow(anyhow::Error),
}

impl Display for ApothecaryServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApothecaryServiceError::NotFound => write!(f, "Apothecary not found"),
            ApothecaryServiceError::InvalidSortColumn(e) => write!(f, "Invalid sort column: {}", e),
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

pub struct ApothecaryService {
    db: DatabaseConnection,
}

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
            .join(
                JoinType::InnerJoin,
                entity::apothecary_medication::Relation::Medication
                    .def()
                    .on_condition(|left, right| {
                        Expr::col((left, entity::apothecary_medication::Column::MedicationId))
                            .eq(Expr::col((right, entity::medication::Column::Id)))
                            .into_condition()
                    }),
            )
            .join(
                JoinType::InnerJoin,
                entity::apothecary_medication::Relation::Apothecary
                    .def()
                    .on_condition(|left, right| {
                        Expr::col((left, entity::apothecary_medication::Column::ApothecaryId))
                            .eq(Expr::col((right, entity::apothecary::Column::Id)))
                            .into_condition()
                    }),
            )
            .filter(
                Condition::all().add(entity::medication::Column::Name.contains(search_dto.name)),
            )
            .distinct_on([entity::apothecary_medication::Column::MedicationId])
            .all(&self.db)
            .await?;

        let mut result: Vec<MedicationSearchResultList> = vec![];

        let mut hash = HashMap::<Uuid, Vec<apothecary_medication::Model>>::new();

        for apothecary_medication in apothecary_medications {
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

                if (apothecary.longitude - search_dto.longitude).powf(2.0f32)
                    + (apothecary.latitude - search_dto.latitude).powf(2.0f32)
                    > (search_dto.max_distance.pow(2) as f32)
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
}
