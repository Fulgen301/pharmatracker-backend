use anyhow::anyhow;
use dto::{medication::MedicationQuantity, reservation::MedicationReservationRequest};
use entity::{
    apothecary_medication::QuantityType, reservation::ReservationWithApothecaryAndMedication,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    Set,
};
use std::fmt::Display;
use time::Duration;
use uuid::Uuid;

pub use entity::apothecary::Model as Apothecary;
pub use entity::medication::Model as Medication;
pub use entity::reservation::Model as Reservation;

use crate::page::{Page, PageError};

pub enum ReservationServiceError {
    UserNotFound,
    MedicationNotFound,
    NotEnoughAvailable,
    Anyhow(anyhow::Error),
}

impl From<DbErr> for ReservationServiceError {
    fn from(err: DbErr) -> Self {
        Self::Anyhow(err.into())
    }
}

impl From<anyhow::Error> for ReservationServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl From<PageError> for ReservationServiceError {
    fn from(err: PageError) -> Self {
        Self::Anyhow(anyhow!(err.to_string()))
    }
}

impl Display for ReservationServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReservationServiceError::UserNotFound => write!(f, "User not found"),
            ReservationServiceError::MedicationNotFound => write!(f, "Medication not found"),
            ReservationServiceError::NotEnoughAvailable => write!(f, "Not enough available"),
            ReservationServiceError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

pub struct ReservationService {
    db: DatabaseConnection,
}

impl ReservationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get(
        &self,
        user_id: Uuid,
    ) -> Result<Page<ReservationWithApothecaryAndMedication>, ReservationServiceError> {
        let reservations = entity::reservation::Entity::find()
            .filter(entity::reservation::Column::UserId.eq(user_id))
            .find_with_related(entity::apothecary::Entity)
            .all(&self.db)
            .await?;

        let mut ret = vec![];

        for (reservation, mut apothecary) in reservations {
            let apothecary = apothecary.pop().unwrap();

            let medication = entity::medication::Entity::find_by_id(reservation.medication_id)
                .one(&self.db)
                .await?
                .ok_or(ReservationServiceError::MedicationNotFound)?;

            let schedules = apothecary
                .find_related(entity::schedule::Entity)
                .all(&self.db)
                .await?;

            ret.push(ReservationWithApothecaryAndMedication::from((
                reservation,
                apothecary,
                schedules,
                medication,
            )));
        }

        Ok(Page::from(ret))
    }

    pub async fn reserve(
        &self,
        user_id: Uuid,
        request: MedicationReservationRequest,
    ) -> Result<ReservationWithApothecaryAndMedication, ReservationServiceError> {
        let apothecary_medicine = entity::apothecary_medication::Entity::find_by_id((
            request.apothecary_id,
            request.medication_id,
        ))
        .one(&self.db)
        .await?
        .ok_or(ReservationServiceError::MedicationNotFound)?;

        let reservation = match (
            request.quantity,
            apothecary_medicine.medication_quantity_type,
        ) {
            (MedicationQuantity::Package(package), QuantityType::Package) => {
                if package.quantity > apothecary_medicine.medication_quantity.unwrap() as u64 {
                    return Err(ReservationServiceError::NotEnoughAvailable);
                }

                entity::reservation::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    apothecary_id: Set(request.apothecary_id),
                    medication_id: Set(request.medication_id),
                    user_id: Set(user_id),
                    quantity_type: Set(apothecary_medicine.medication_quantity_type),
                    quantity: Set(Some(package.quantity as _)),
                    price: Set(apothecary_medicine.medication_price),
                    status: Set(entity::reservation::ReservationStatus::Active),
                    start_date_time: Set(Some(request.start_date_time)),
                    end_date_time: Set(Some(
                        request
                            .start_date_time
                            .checked_add(Duration::minutes(30))
                            .ok_or(anyhow!("Failed to add 30min".to_owned()))?,
                    )),
                }
            }
            (MedicationQuantity::Package(package), QuantityType::Unknown) => {
                entity::reservation::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    apothecary_id: Set(request.apothecary_id),
                    medication_id: Set(request.medication_id),
                    user_id: Set(user_id),
                    quantity_type: Set(apothecary_medicine.medication_quantity_type),
                    quantity: Set(Some(package.quantity as _)),
                    price: Set(apothecary_medicine.medication_price),
                    status: Set(entity::reservation::ReservationStatus::Pending),
                    start_date_time: Set(None),
                    end_date_time: Set(None),
                }
            }
            (_, _) => {
                return Err(ReservationServiceError::NotEnoughAvailable);
            }
        };

        let apothecary = entity::apothecary::Entity::find_by_id(request.apothecary_id)
            .one(&self.db)
            .await?
            .ok_or(ReservationServiceError::MedicationNotFound)?;

        let schedules = apothecary
            .find_related(entity::schedule::Entity)
            .all(&self.db)
            .await?;

        let medication = entity::medication::Entity::find_by_id(request.medication_id)
            .one(&self.db)
            .await?
            .ok_or(ReservationServiceError::MedicationNotFound)?;

        Ok(ReservationWithApothecaryAndMedication::from((
            reservation.insert(&self.db).await?,
            apothecary,
            schedules,
            medication,
        )))
    }
}
