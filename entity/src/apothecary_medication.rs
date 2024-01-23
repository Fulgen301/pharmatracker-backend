use std::fmt::Display;

use dto::medication::{MedicationQuantity, MedicationQuantityPackage, MedicationQuantityUnknown};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(Some(1))",
    enum_name = "medication_quantity_type"
)]
pub enum QuantityType {
    #[sea_orm(string_value = "p")]
    Package,
    #[sea_orm(string_value = "u")]
    Unknown,
}

impl Display for QuantityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantityType::Package => write!(f, "package"),
            QuantityType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "apothecary_medication")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub apothecary_id: Uuid,
    #[sea_orm(primary_key)]
    pub medication_id: Uuid,
    pub medication_quantity_type: QuantityType,
    pub medication_quantity: Option<i64>,
    pub medication_price: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::medication::Entity",
        from = "Column::MedicationId",
        to = "super::medication::Column::Id"
    )]
    Medication,

    #[sea_orm(
        belongs_to = "super::apothecary::Entity",
        from = "Column::ApothecaryId",
        to = "super::apothecary::Column::Id"
    )]
    Apothecary,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for MedicationQuantity {
    fn from(medication: Model) -> Self {
        match medication.medication_quantity_type {
            QuantityType::Package => Self::Package(MedicationQuantityPackage {
                quantity: medication
                    .medication_quantity
                    .expect("Package quantity is missing") as _,
                price: medication.medication_price,
            }),
            QuantityType::Unknown => Self::Unknown(MedicationQuantityUnknown),
        }
    }
}
