use std::fmt::Display;

use dto::{
    medication::{MedicationQuantity, MedicationQuantityPackage, MedicationQuantityUnknown},
    reservation::MedicationReservationStatus,
};
use sea_orm::entity::prelude::*;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::{apothecary::ApothecaryWithSchedules, apothecary_medication::QuantityType};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(Some(1))",
    enum_name = "medication_quantity_type"
)]
pub enum ReservationStatus {
    #[sea_orm(string_value = "a")]
    Active,
    #[sea_orm(string_value = "p")]
    Pending,
    #[sea_orm(string_value = "r")]
    Rejected,
    #[sea_orm(string_value = "d")]
    Done,
}

impl Display for ReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReservationStatus::Active => write!(f, "active"),
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Rejected => write!(f, "rejected"),
            ReservationStatus::Done => write!(f, "done"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reservation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub apothecary_id: Uuid,
    pub medication_id: Uuid,
    pub user_id: Uuid,
    pub quantity_type: QuantityType,
    pub quantity: Option<i64>,
    pub price: Decimal,
    pub status: ReservationStatus,
    pub start_date_time: Option<PrimitiveDateTime>,
    pub end_date_time: Option<PrimitiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::apothecary::Entity",
        from = "Column::ApothecaryId",
        to = "super::apothecary::Column::Id"
    )]
    Apothecary,

    #[sea_orm(
        belongs_to = "super::medication::Entity",
        from = "Column::MedicationId",
        to = "super::medication::Column::Id"
    )]
    Medication,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::apothecary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Apothecary.def()
    }
}

impl Related<super::medication::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Medication.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

pub struct ReservationWithApothecaryAndMedication(
    (
        Model,
        super::apothecary::Model,
        Vec<super::schedule::Model>,
        super::medication::Model,
    ),
);

impl
    From<(
        Model,
        super::apothecary::Model,
        Vec<super::schedule::Model>,
        super::medication::Model,
    )> for ReservationWithApothecaryAndMedication
{
    fn from(
        (reservation, apothecary, schedules, medication): (
            Model,
            super::apothecary::Model,
            Vec<super::schedule::Model>,
            super::medication::Model,
        ),
    ) -> Self {
        Self((reservation, apothecary, schedules, medication))
    }
}

impl From<ReservationWithApothecaryAndMedication> for dto::reservation::MedicationReservation {
    fn from(
        reservation_with_apothecary_and_medication: ReservationWithApothecaryAndMedication,
    ) -> Self {
        let (reservation, apothecary, schedules, medication) =
            reservation_with_apothecary_and_medication.0;

        let now = OffsetDateTime::now_utc();
        let now = PrimitiveDateTime::new(now.date(), now.time());

        Self {
            id: reservation.id,
            apothecary: ApothecaryWithSchedules::from((apothecary, schedules)).into(),
            medication: medication.into(),
            quantity: match reservation.quantity_type {
                QuantityType::Package => MedicationQuantity::Package(MedicationQuantityPackage {
                    quantity: reservation.quantity.unwrap() as _,
                    price: reservation.price,
                }),
                QuantityType::Unknown => MedicationQuantity::Unknown(MedicationQuantityUnknown),
            },
            start_date_time: reservation.start_date_time,
            end_date_time: reservation.end_date_time,
            status: {
                if reservation.end_date_time < Some(now) {
                    MedicationReservationStatus::Expired
                } else {
                    match reservation.status {
                        ReservationStatus::Active => MedicationReservationStatus::Active,
                        ReservationStatus::Pending => MedicationReservationStatus::Pending,
                        ReservationStatus::Rejected => MedicationReservationStatus::Rejected,
                        ReservationStatus::Done => MedicationReservationStatus::Done,
                    }
                }
            },
        }
    }
}
